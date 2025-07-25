// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! A data exporter that bakes the data into Rust code.
//!
//! This module can be used as a target for the `icu_provider_export` crate.
//!
//! See our [datagen tutorial](https://github.com/unicode-org/icu4x/blob/main/tutorials/data-management.md) for more information about different data providers.
//!
//! # Examples
//!
//! ```
//! use icu_provider_export::baked_exporter::*;
//! use icu_provider_export::prelude::*;
//!
//! let demo_path = std::env::temp_dir().join("icu4x_baked_demo");
//! # let _ = std::fs::remove_dir_all(&demo_path);
//!
//! // Set up the exporter
//! let mut exporter =
//!     BakedExporter::new(demo_path.clone(), Default::default()).unwrap();
//!
//! // Export something. Make sure to use the same fallback data at runtime!
//! ExportDriver::new(
//!     [DataLocaleFamily::FULL],
//!     DeduplicationStrategy::Maximal.into(),
//!     LocaleFallbacker::new().static_to_owned(),
//! )
//! .export(&icu_provider::hello_world::HelloWorldProvider, exporter)
//! .unwrap();
//! #
//! # let _ = std::fs::remove_dir_all(&demo_path);
//! ```
//!
//! There are two ways to use baked data: you can build custom data providers for use with
//! [`_unstable` constructors](icu_provider::constructors), or you can use it with the
//! `compiled_data` Cargo feature and constructors.
//!
//! ## Custom `DataProvider`
//!
//! This allows you to use baked data in custom data pipelines, such as including some baked
//! data and lazily loading more data from the network.
//!
//! ```
//! # use icu_provider::prelude::icu_locale_core;
//! use icu_locale_core::locale;
//! use icu_provider::hello_world::*;
//!
//! # macro_rules! include {
//! #   ($path:literal) => {}
//! # }
//! # macro_rules! impl_data_provider {
//! #   ($p:ty) => {
//! #     use icu_provider::prelude::*;
//! #     use icu_provider::hello_world::*;
//! #     impl DataProvider<HelloWorldV1> for $p {
//! #       fn load(&self, req: DataRequest) -> Result<DataResponse<HelloWorldV1>, DataError> {
//! #         HelloWorldProvider.load(req)
//! #       }
//! #     }
//! #   }
//! # }
//! include!("/tmp/icu4x_baked_demo/mod.rs");
//!
//! pub struct MyDataProvider;
//! impl_data_provider!(MyDataProvider);
//!
//! # fn main() {
//! let formatter = HelloWorldFormatter::try_new_unstable(&MyDataProvider, locale!("en").into()).unwrap();
//!
//! assert_eq!(formatter.format_to_string(), "Hello World");
//! # }
//! ```
//!
//! ## `compiled_data`
//!
//! You can use baked data to overwrite the compiled data that's included in ICU4X.
//! To do this, build your binary with the `ICU4X_DATA_DIR` environment variable:
//!
//! ```console
//! ICU4X_DATA_DIR=/tmp/icu4x_baked_demo cargo build <...>
//! ```
//!
//! ```
//! # use icu_provider::prelude::icu_locale_core;
//! use icu_locale_core::locale;
//! use icu_provider::hello_world::*;
//!
//! let formatter = HelloWorldFormatter::try_new(locale!("en").into()).unwrap();
//!
//! assert_eq!(formatter.format_to_string(), "Hello World");
//! ```

use databake::*;
use heck::ToShoutySnakeCase;
use heck::ToSnakeCase;
use icu_provider::export::*;
use icu_provider::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

// TokenStream isn't Send/Sync
type SyncTokenStream = String;

// Produces an MSRV clippy annotation if the `CARGO_PKG_RUST_VERSION` is set.
fn maybe_msrv() -> TokenStream {
    std::option_env!("CARGO_PKG_RUST_VERSION")
        .map(|msrv| {
            quote! {
                #[clippy::msrv = #msrv]
            }
        })
        .unwrap_or_default()
}

/// Options for configuring the output of [`BakedExporter`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// By default, baked providers perform fallback internally. This field can be used to
    /// disable this behavior.
    pub use_internal_fallback: bool,
    /// Whether to run `rustfmt` on the generated files.
    pub pretty: bool,
    /// Whether to use separate crates to name types instead of the `icu` metacrate.
    ///
    /// By default, types will be named through the `icu` crate, like `icu::list::provider::ListJoinerPattern`.
    /// With this enabled, the alternative name from the component crates will be used: `icu_list::provider::ListJoinerPattern`.
    /// This is required when you are not using the `icu` crate, *and* you're building custom data providers;
    /// data for `compiled_data` constructors uses `icu` names.
    pub use_separate_crates: bool,
    /// Whether to overwrite existing data. By default, errors if it is present.
    pub overwrite: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            use_internal_fallback: true,
            pretty: false,
            use_separate_crates: false,
            overwrite: false,
        }
    }
}

#[expect(clippy::type_complexity)]
/// See the module-level documentation for details.
pub struct BakedExporter {
    // Input arguments
    mod_directory: PathBuf,
    pretty: bool,
    use_separate_crates: bool,
    use_internal_fallback: bool,
    // Temporary storage for put_payload: marker -> (payload -> {data id})
    data: Mutex<
        HashMap<
            DataMarkerInfo,
            HashMap<DataPayload<ExportMarker>, BTreeSet<DataIdentifierCow<'static>>>,
        >,
    >,
    /// file names, required crates, and statistics to be consumed by `close`.
    impl_data:
        Mutex<BTreeMap<DataMarkerInfo, (SyncTokenStream, BTreeSet<&'static str>, Statistics)>>,
}

#[derive(Default)]
pub struct Statistics {
    pub structs_total_size: usize,
    pub structs_count: usize,
    pub lookup_struct_size: usize,
    pub identifiers_count: usize,
}

impl std::fmt::Debug for BakedExporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BakedExporter")
            .field("mod_directory", &self.mod_directory)
            .field("pretty", &self.pretty)
            .field("use_separate_crates", &self.use_separate_crates)
            // skip formatting intermediate data
            .finish()
    }
}

impl BakedExporter {
    /// Constructs a new [`BakedExporter`] with the given output directory and options.
    pub fn new(mod_directory: PathBuf, options: Options) -> Result<Self, DataError> {
        let Options {
            use_internal_fallback,
            pretty,
            use_separate_crates,
            overwrite,
        } = options;

        if mod_directory.exists() {
            if overwrite {
                std::fs::remove_dir_all(&mod_directory)
            } else {
                std::fs::remove_dir(&mod_directory)
            }
            .map_err(|e| DataError::from(e).with_path_context(&mod_directory))?;
        }

        Ok(Self {
            mod_directory,
            pretty,
            use_internal_fallback,
            use_separate_crates,
            data: Default::default(),
            impl_data: Default::default(),
        })
    }

    fn write_to_file(&self, relative_path: &Path, data: TokenStream) -> Result<(), DataError> {
        let path = self.mod_directory.join(relative_path);

        let mut formatted = if self.pretty {
            use std::process::{Command, Stdio};
            let mut rustfmt = Command::new("rustfmt")
                .arg("--config")
                .arg("newline_style=unix")
                .arg("--config")
                .arg("normalize_doc_attributes=true")
                .arg("--config")
                .arg("max_width=5000000") // better to format wide than to not format
                // currently unnecessary, may become necessary for format_macro_bodies
                // in the future
                .arg("--config")
                .arg("unstable_features=true")
                .arg("--config")
                .arg("format_macro_bodies=true")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;
            let mut rustfmt_stdin = rustfmt.stdin.take().unwrap();
            write!(rustfmt_stdin, "{data}")?;

            drop(rustfmt_stdin); // EOF

            let output = rustfmt.wait_with_output()?;
            if !output.status.success() {
                let stderr = String::from_utf8(output.stderr)
                    .map_err(|_| DataError::custom("rustfmt output not utf-8"))?;
                return Err(DataError::custom("rustfmt failed").with_display_context(&stderr));
            }
            String::from_utf8(output.stdout)
                .map_err(|_| DataError::custom("rustfmt output not utf-8"))?
        } else {
            data.to_string()
        };

        if !self.use_separate_crates {
            // Don't search the whole file, there should be a macro in the first 1000 bytes
            if formatted[..core::cmp::min(formatted.len(), 1000)].contains("macro_rules!")
                || formatted[..core::cmp::min(formatted.len(), 1000)].contains("include!")
            {
                // Formatted, otherwise it'd be `macro_rules !`
                formatted = formatted
                    .replace("icu_", "icu::")
                    .replace("icu::provider", "icu_provider")
                    .replace("icu::locale_core", "icu_locale_core")
                    .replace("icu::pattern", "icu_pattern");
            } else {
                // Unformatted
                formatted = formatted
                    .replace("icu_", "icu :: ")
                    .replace("icu :: provider", "icu_provider")
                    .replace("icu :: locale_core", "icu_locale_core")
                    .replace("icu :: pattern", "icu_pattern");
            }
        }

        std::fs::create_dir_all(path.parent().unwrap())?;
        let mut file = crlify::BufWriterWithLineEndingFix::new(
            File::create(&path).map_err(|e| DataError::from(e).with_path_context(&path))?,
        );
        write!(file, "// @generated\n{formatted}")
            .map_err(|e| DataError::from(e).with_path_context(&path))
    }

    fn write_impl_macros(
        &self,
        marker: DataMarkerInfo,
        dependencies: CrateEnv,
        stats: Statistics,
        body: TokenStream,
        dry_body: Option<TokenStream>,
        iterable_body: TokenStream,
    ) -> Result<(), DataError> {
        let marker_unqualified = bake_marker(marker).into_iter().last().unwrap().to_string();

        let mut required_crates = dependencies.into_iter().collect::<BTreeSet<_>>();
        if !self.use_separate_crates {
            required_crates.retain(|&krate| {
                !krate.starts_with("icu_")
                    || krate.starts_with("icu_provider")
                    || krate == "icu_locale_core"
                    || krate == "icu_pattern"
            });
            required_crates.insert("icu");
        }

        let &Statistics {
            structs_total_size,
            structs_count,
            lookup_struct_size,
            identifiers_count,
        } = &stats;

        let mut doc = format!(
            " Implement `DataProvider<{marker_unqualified}>` on the given struct using the data\n \
            hardcoded in this file. This allows the struct to be used with\n \
            `icu`'s `_unstable` constructors."
        );

        if structs_count > 0 {
            let _infallible = write!(&mut doc, "\n\n Using this implementation will embed the following data in the binary's data segment:\n ");

            if marker.is_singleton {
                let _infallible = write!(
                    &mut doc,
                    "* {structs_total_size}B[^1] for the singleton data struct\n "
                );
            } else {
                let _infallible = write!(&mut doc, "* {lookup_struct_size}B for the lookup data structure ({identifiers_count} data identifiers)\n ");
                let _infallible = write!(&mut doc, "* {structs_total_size}B[^1] for the actual data ({structs_count} unique structs)\n ");
            };
            let _infallible = write!(
                &mut doc,
                "\n [^1]: these numbers can be smaller in practice due to linker deduplication"
            );
        }

        let _infallible = writeln!(&mut doc, "\n\n This macro requires the following crates:");
        for required_crate in &required_crates {
            let _infallible = writeln!(&mut doc, " * `{required_crate}`");
        }

        let ident = marker_unqualified.to_snake_case();

        let macro_ident = format!("impl_{ident}",).parse::<TokenStream>().unwrap();

        // We prefix all macros with `__`, as these will be automatically exported at the crate root, which is annoying
        // for crates that include the data but don't want it to be public. We then reexport them as items that use
        // normal scoping that clients can control.
        let prefixed_macro_ident = format!("__{macro_ident}").parse::<TokenStream>().unwrap();

        let maybe_msrv = maybe_msrv();

        let dry = if let Some(dry_body) = dry_body {
            quote! {
                ($provider:ty, DRY) => {
                    #prefixed_macro_ident!($provider);
                    #dry_body
                };
                ($provider:ty, DRY, ITER) => {
                    #prefixed_macro_ident!($provider);
                    #dry_body
                    #iterable_body
                };
            }
        } else {
            quote! {
                ($provider:ty, DRY) => {
                };
                ($provider:ty, DRY, ITER) => {
                    #prefixed_macro_ident!($provider, ITER);
                };
            }
        };

        self.write_to_file(
            Path::new(&format!("{ident}.rs.data")),
            quote! {
                #[doc = #doc]
                #[doc(hidden)] // macro
                #[macro_export]
                macro_rules! #prefixed_macro_ident {
                    ($provider:ty) => {
                        #maybe_msrv
                        const _: () = <$provider>::MUST_USE_MAKE_PROVIDER_MACRO;
                        #body
                    };
                    ($provider:ty, ITER) => {
                        #prefixed_macro_ident!($provider);
                        #iterable_body
                    };
                    #dry
                }
                #[doc(inline)]
                pub use #prefixed_macro_ident as #macro_ident;
            },
        )?;

        self.impl_data
            .lock()
            .expect("poison")
            .insert(marker, (ident, required_crates, stats));
        Ok(())
    }
}

impl DataExporter for BakedExporter {
    fn put_payload(
        &self,
        marker: DataMarkerInfo,
        id: DataIdentifierBorrowed,
        payload: &DataPayload<ExportMarker>,
    ) -> Result<(), DataError> {
        self.data
            .lock()
            .expect("poison")
            .entry(marker)
            .or_default()
            .entry(payload.clone())
            .or_default()
            .insert(id.into_owned());
        Ok(())
    }

    fn flush_singleton(
        &self,
        marker: DataMarkerInfo,
        payload: &DataPayload<ExportMarker>,
        metadata: FlushMetadata,
    ) -> Result<(), DataError> {
        let maybe_msrv = maybe_msrv();

        let marker_bake = bake_marker(marker);

        let singleton_ident = format!(
            "SINGLETON_{}",
            marker_bake
                .clone()
                .into_iter()
                .last()
                .unwrap()
                .to_string()
                .to_shouty_snake_case()
        )
        .parse::<TokenStream>()
        .unwrap();

        let (checksum_decl, metadata_bake) = if let Some(checksum) = metadata.checksum {
            let singleton_checksum_ident = format!("{singleton_ident}_CHECKSUM")
                .parse::<TokenStream>()
                .unwrap();
            (
                quote! {
                    #[doc(hidden)] // singletons might be used cross-crate
                    pub const #singleton_checksum_ident: u64 = #checksum;
                },
                quote! {
                    icu_provider::DataResponseMetadata::default().with_checksum(Self::#singleton_checksum_ident)
                },
            )
        } else {
            (
                quote!(),
                quote! {
                    icu_provider::DataResponseMetadata::default()
                },
            )
        };

        let dependencies = CrateEnv::default();
        dependencies.insert("icu_provider");

        let bake = payload.tokenize(&dependencies);

        let stats = Statistics {
            structs_total_size: payload.baked_size(),
            structs_count: 1,
            identifiers_count: 1,
            lookup_struct_size: 0,
        };

        self.write_impl_macros(marker, dependencies, stats, quote! {
            #maybe_msrv
            impl $provider {
                // Exposing singleton structs as consts allows us to get rid of fallibility
                #[doc(hidden)] // singletons might be used cross-crate
                pub const #singleton_ident: &'static <#marker_bake as icu_provider::DynamicDataMarker>::DataStruct = &#bake;
                #checksum_decl
            }

            #maybe_msrv
            impl icu_provider::DataProvider<#marker_bake> for $provider {
                fn load(
                    &self,
                    req: icu_provider::DataRequest,
                ) -> Result<icu_provider::DataResponse<#marker_bake>, icu_provider::DataError> {
                    if req.id.locale.is_unknown() {
                        Ok(icu_provider::DataResponse {
                            payload: icu_provider::DataPayload::from_static_ref(Self::#singleton_ident),
                            metadata: #metadata_bake,
                        })
                    } else {
                        Err(icu_provider::DataErrorKind::InvalidRequest.with_req(<#marker_bake as icu_provider::DataMarker>::INFO, req))
                    }
                }
            }
        },
        Some(quote! {
            #maybe_msrv
            impl icu_provider::DryDataProvider<#marker_bake> for $provider {
                fn dry_load(&self, req: icu_provider::DataRequest) -> Result<icu_provider::DataResponseMetadata, icu_provider::DataError> {
                    if req.id.locale.is_unknown() {
                        Ok(#metadata_bake)
                    } else {
                        Err(icu_provider::DataErrorKind::InvalidRequest.with_req(<#marker_bake as icu_provider::DataMarker>::INFO, req))
                    }
                }
            }
        }),
        quote! {
            #maybe_msrv
            impl icu_provider::IterableDataProvider<#marker_bake> for $provider {
                fn iter_ids(&self) -> Result<std::collections::BtreeSet<icu_provider::DataIdentifierCow<'static>>, icu_provider::DataError> {
                    Ok([Default::default()].into_iter().collect())
                }
            }
        })
    }

    fn flush(&self, marker: DataMarkerInfo, metadata: FlushMetadata) -> Result<(), DataError> {
        let maybe_msrv = maybe_msrv();

        let marker_bake = bake_marker(marker);
        let marker_bake_shouty = marker_bake
            .clone()
            .into_iter()
            .last()
            .unwrap()
            .to_string()
            .to_shouty_snake_case();

        let deduplicated_values = self
            .data
            .lock()
            .expect("poison")
            .remove(&marker)
            .unwrap_or_default();

        let dependencies = CrateEnv::default();
        dependencies.insert("icu_provider");

        if deduplicated_values.is_empty() {
            self.write_impl_macros(
                marker,
                dependencies,
                Default::default(),
                quote! {
                    #maybe_msrv
                    impl icu_provider::DataProvider<#marker_bake> for $provider {
                        fn load(
                            &self,
                            req: icu_provider::DataRequest,
                        ) -> Result<icu_provider::DataResponse<#marker_bake>, icu_provider::DataError> {
                            Err(icu_provider::DataErrorKind::IdentifierNotFound.with_req(<#marker_bake as icu_provider::DataMarker>::INFO, req))
                        }
                    }
                },
                Some(quote! {
                    #maybe_msrv
                    impl icu_provider::DryDataProvider<#marker_bake> for $provider {
                        fn dry_load(&self, req: icu_provider::DataRequest) -> Result<icu_provider::DataResponseMetadata, icu_provider::DataError> {
                            Err(icu_provider::DataErrorKind::IdentifierNotFound.with_req(<#marker_bake as icu_provider::DataMarker>::INFO, req))
                        }
                    }
                }),
                quote! {
                    #maybe_msrv
                    impl icu_provider::IterableDataProvider<#marker_bake> for $provider {
                        fn iter_ids(&self) -> Result<std::collections::BTreeSet<icu_provider::DataIdentifierCow<'static>>, icu_provider::DataError> {
                            Ok(Default::default())
                        }
                    }
                },
            )
        } else {
            let mut stats = Statistics::default();

            let mut values = deduplicated_values
                .iter()
                .map(|(payload, ids)| {
                    // TODO(#5230): Update these size calculations for EncodedStruct storage
                    stats.structs_count += 1;
                    stats.identifiers_count += ids.len();
                    stats.structs_total_size += payload.baked_size();
                    (payload, ids)
                })
                .collect::<Vec<_>>();

            // Stability
            values.sort_by(|a, b| a.1.first().cmp(&b.1.first()));

            let values = values.iter().enumerate();

            // Safety invariant upheld: the only values being added to the trie are `index`
            // values, which come from enumerating `values`
            let trie = icu_provider::baked::zerotrie::ZeroTrieSimpleAscii::from_iter(
                values.clone().flat_map(|(index, (_payload, ids))| {
                    ids.iter().map(move |id| {
                        let mut encoded = id.locale.to_string().into_bytes();
                        if !id.marker_attributes.is_empty() {
                            encoded.push(icu_provider::baked::zerotrie::ID_SEPARATOR);
                            encoded.extend_from_slice(id.marker_attributes.as_bytes());
                        }
                        (encoded, index)
                    })
                }),
            );

            stats.lookup_struct_size = core::mem::size_of::<
                icu_provider::baked::zerotrie::Data<icu_provider::hello_world::HelloWorldV1>,
            >() + trie.as_borrowed_slice().borrows_size();

            let mut consts = vec![];
            let baked_trie = trie.as_borrowed_slice().bake(&Default::default());
            let data_ident = format!("DATA_{marker_bake_shouty}")
                .parse::<TokenStream>()
                .unwrap();

            if let Some(vzv_tokens) = DataPayload::tokenize_encoded_seq(
                &values
                    .clone()
                    .map(|(_index, (payload, _ids))| *payload)
                    .collect::<Vec<_>>(),
                &dependencies,
            ) {
                consts.push(quote! {
                    // Safety invariant upheld: see above
                    const #data_ident: icu_provider::baked::zerotrie::DataForVarULEs<#marker_bake> = {
                        const TRIE: icu_provider::baked::zerotrie::ZeroTrieSimpleAscii<&'static [u8]> = icu_provider::baked:: #baked_trie;
                        const VALUES: &'static zerovec::VarZeroSlice<<<#marker_bake as icu_provider::baked::zerotrie::DynamicDataMarker>::DataStruct as icu_provider::ule::MaybeAsVarULE>::EncodedStruct> = #vzv_tokens;
                        unsafe {
                            icu_provider::baked::zerotrie::DataForVarULEs::from_trie_and_values_unchecked(TRIE, VALUES)
                        }
                    };
                });
            } else if marker.expose_baked_consts {
                let bakes = values.clone().map(|(_index, (payload, ids))| {
                    let ident = format!(
                        "{marker_bake_shouty}_{}",
                        ids.first().unwrap().to_string().to_shouty_snake_case()
                    )
                    .parse::<TokenStream>()
                    .unwrap();
                    let bake = payload.tokenize(&dependencies);

                    consts.push(quote! {
                        pub const #ident: &<#marker_bake as icu_provider::baked::zerotrie::DynamicDataMarker>::DataStruct = &#bake;
                    });

                    for deduped in ids.iter().skip(1) {
                        let deduped_ident = format!(
                            "{marker_bake_shouty}_{}",
                            deduped.to_string().to_shouty_snake_case()
                        )
                        .parse::<TokenStream>()
                        .unwrap();
                        consts.push(quote! {
                            pub const #deduped_ident: &<#marker_bake as icu_provider::baked::zerotrie::DynamicDataMarker>::DataStruct = Self::#ident;
                        });
                    }

                    quote!(Self::#ident)
                });
                let data = quote! {
                    // Safety invariant upheld: see above
                    const #data_ident: icu_provider::baked::zerotrie::DataRef<#marker_bake> = unsafe {
                        icu_provider::baked::zerotrie::DataRef::from_trie_and_refs_unchecked(icu_provider::baked:: #baked_trie, &[#(#bakes,)*])
                    };
                };
                consts.push(data);
            } else {
                let bakes = values
                    .clone()
                    .map(|(_index, (payload, _ids))| payload.tokenize(&dependencies));
                consts.push(quote! {
                    // Safety invariant upheld: see above
                    const #data_ident: icu_provider::baked::zerotrie::Data<#marker_bake> = {
                        const TRIE: icu_provider::baked::zerotrie::ZeroTrieSimpleAscii<&'static [u8]> = icu_provider::baked:: #baked_trie;
                        const VALUES: &'static [<#marker_bake as icu_provider::baked::zerotrie::DynamicDataMarker>::DataStruct] = &[#(#bakes,)*];
                        unsafe {
                            icu_provider::baked::zerotrie::Data::from_trie_and_values_unchecked(TRIE, VALUES)
                        }
                    };
                });
            };

            let metadata_bake = if let Some(checksum) = metadata.checksum {
                quote! {
                    icu_provider::DataResponseMetadata::default().with_checksum(#checksum)
                }
            } else {
                quote! {
                    icu_provider::DataResponseMetadata::default()
                }
            };

            dependencies.insert("icu_provider/baked");

            let search = if !self.use_internal_fallback {
                quote! {
                    let metadata = #metadata_bake;
                    let Some(payload) = icu_provider::baked::DataStore::get(&Self::#data_ident, req.id, req.metadata.attributes_prefix_match) else {
                        return Err(icu_provider::DataErrorKind::IdentifierNotFound.with_req(<#marker_bake as icu_provider::DataMarker>::INFO, req))
                    };
                }
            } else if deduplicated_values
                .iter()
                .all(|(_, ids)| ids.iter().all(|id| id.locale.is_unknown()))
            {
                quote! {
                    // we need to use fallback, but all values are for root locale, so we just go there directly
                    let mut req = req;
                    req.id.locale = Default::default();
                    let metadata = #metadata_bake;
                    let Some(payload) = icu_provider::baked::DataStore::get(&Self::#data_ident, req.id, req.metadata.attributes_prefix_match) else {
                        return Err(icu_provider::DataErrorKind::IdentifierNotFound.with_req(<#marker_bake as icu_provider::DataMarker>::INFO, req))
                    };
                }
            } else {
                dependencies.insert("icu_locale/compiled_data");
                quote! {
                    let mut metadata = #metadata_bake;

                    let payload =  if let Some(payload) = icu_provider::baked::DataStore::get(&Self::#data_ident, req.id, req.metadata.attributes_prefix_match) {
                        payload
                    } else {
                        const FALLBACKER: icu_locale::fallback::LocaleFallbackerWithConfig<'static> =
                            icu_locale::fallback::LocaleFallbacker::new()
                                .for_config(<#marker_bake as icu_provider::DataMarker>::INFO.fallback_config);
                        let mut fallback_iterator = FALLBACKER.fallback_for(req.id.locale.clone());
                        loop {
                            if let Some(payload) = icu_provider::baked::DataStore::get(&Self::#data_ident, icu_provider::DataIdentifierBorrowed::for_marker_attributes_and_locale(req.id.marker_attributes, fallback_iterator.get()), req.metadata.attributes_prefix_match) {
                                metadata.locale = Some(fallback_iterator.take());
                                break payload;
                            }
                            if fallback_iterator.get().is_unknown() {
                                return Err(icu_provider::DataErrorKind::IdentifierNotFound.with_req(<#marker_bake as icu_provider::DataMarker>::INFO, req));
                            }
                            fallback_iterator.step();
                        }
                    };
                }
            };

            self.write_impl_macros(
                marker,
                dependencies,
                stats,
                quote! {
                    #maybe_msrv
                    impl $provider {
                        #(#consts)*
                    }

                    #maybe_msrv
                    impl icu_provider::DataProvider<#marker_bake> for $provider {
                        fn load(
                            &self,
                            req: icu_provider::DataRequest,
                        ) -> Result<icu_provider::DataResponse<#marker_bake>, icu_provider::DataError> {
                            #search

                            Ok(icu_provider::DataResponse {
                                payload,
                                metadata
                            })
                        }
                    }
                },
                if metadata.supports_dry_provider {
                    Some(quote! {
                        #maybe_msrv
                        impl icu_provider::DryDataProvider<#marker_bake> for $provider {
                            fn dry_load(&self, req: icu_provider::DataRequest) -> Result<icu_provider::DataResponseMetadata, icu_provider::DataError> {
                                icu_provider::DataProvider::<#marker_bake>::load(self, req).map(|r| r.metadata)
                            }
                        }
                    })
                } else {
                    None
                },
                quote! {
                    #maybe_msrv
                    impl icu_provider::IterableDataProvider<#marker_bake> for $provider {
                        fn iter_ids(&self) -> Result<std::collections::BTreeSet<icu_provider::DataIdentifierCow<'static>>, icu_provider::DataError> {
                            Ok(icu_provider::baked::DataStore::iter(&Self::#data_ident).collect())
                        }
                    }
                },
            )
        }
    }

    fn close(&mut self) -> Result<ExporterCloseMetadata, DataError> {
        log::info!("Writing macros module...");

        let data = core::mem::take(&mut self.impl_data)
            .into_inner()
            .expect("poison");

        let maybe_msrv = maybe_msrv();

        let file_paths = data.values().map(|(i, _, _)| format!("{i}.rs.data"));

        let macro_idents = data
            .values()
            .map(|(i, _, _)| format!("impl_{i}").parse::<TokenStream>().unwrap());

        let required_crates = data
            .values()
            .flat_map(|(_, deps, _)| deps.iter().copied())
            .collect::<BTreeSet<_>>();

        let required_crates_list = required_crates.iter().map(|c| format!(" * `{c}`"));

        // mod.rs is the interface for built-in data. It exposes one macro per marker.
        self.write_to_file(
            Path::new("mod.rs"),
            quote! {
                #(
                    include!(#file_paths);
                )*

                /// Marks a type as a data provider. You can then use macros like
                /// `impl_core_helloworld_v1` to add implementations.
                ///
                /// ```ignore
                /// struct MyProvider;
                /// const _: () = {
                ///     include!("path/to/generated/macros.rs");
                ///     make_provider!(MyProvider);
                ///     impl_core_helloworld_v1!(MyProvider);
                /// }
                /// ```
                #[doc(hidden)] // macro
                #[macro_export]
                macro_rules! __make_provider {
                    ($name:ty) => {
                        #maybe_msrv
                        impl $name {
                            #[allow(dead_code)]
                            pub(crate) const MUST_USE_MAKE_PROVIDER_MACRO: () = ();
                        }
                        icu_provider::marker::impl_data_provider_never_marker!($name);
                    };
                }
                #[doc(inline)]
                pub use __make_provider as make_provider;

                // Not public as it will only work locally due to needing access to the other macros.
                /// This macro requires the following crates:
                #(
                    #[doc = #required_crates_list]
                )*
                #[allow(unused_macros)]
                macro_rules! impl_data_provider {
                    ($provider:ty) => {
                        make_provider!($provider);
                        #(
                            #macro_idents ! ($provider);
                        )*
                    };
                }
            },
        )?;

        let statistics = data
            .into_iter()
            .map(|(marker, (_, _, stats))| (marker, stats))
            .collect();

        Ok(ExporterCloseMetadata(Some(Box::new(
            BakedExporterCloseMetadata {
                statistics,
                required_crates,
            },
        ))))
    }
}

/// Metadata of a bake export
pub struct BakedExporterCloseMetadata {
    /// Per-marker size heuristics
    pub statistics: BTreeMap<DataMarkerInfo, Statistics>,
    /// List of crates required to compile the output
    pub required_crates: BTreeSet<&'static str>,
}

macro_rules! cb {
    ($($marker_ty:ty:$marker:ident,)+ #[experimental] $($emarker_ty:ty:$emarker:ident,)+) => {
        fn bake_marker(marker: DataMarkerInfo) -> databake::TokenStream {
            if marker.id == icu_provider::hello_world::HelloWorldV1::INFO.id {
                return databake::quote!(icu_provider::hello_world::HelloWorldV1);
            }

            $(
                if marker.id.name() == stringify!($marker) {
                    return stringify!($marker_ty)
                        .replace("icu :: ", "icu_")
                        .parse()
                        .unwrap();
                }
            )+

            $(
                if marker.id.name() == stringify!($emarker) {
                    return stringify!($emarker_ty)
                        .replace("icu :: ", "icu_")
                        .parse()
                        .unwrap();
                }
            )+

            unreachable!("unregistered marker {marker:?}")
        }
    }
}
icu_provider_registry::registry!(cb);
