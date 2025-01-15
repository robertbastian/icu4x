// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

#[diplomat::bridge]
#[diplomat::abi_rename = "icu4x_{0}_mv1"]
#[diplomat::attr(auto, namespace = "icu4x")]
pub mod ffi {
    use alloc::boxed::Box;
    use writeable::Writeable;

    #[cfg(feature = "buffer_provider")]
    use crate::{errors::ffi::DataError, provider::ffi::DataProvider};

    use tinystr::TinyAsciiStr;

    /// A mapper between IANA time zone identifiers and BCP-47 time zone identifiers.
    ///
    /// This mapper supports two-way mapping, but it is optimized for the case of IANA to BCP-47.
    /// It also supports normalizing and canonicalizing the IANA strings.
    #[diplomat::opaque]
    #[diplomat::rust_link(icu::timezone::IanaMapper, Struct)]
    #[diplomat::rust_link(icu::timezone::IanaMapper::as_borrowed, FnInStruct, hidden)]
    #[diplomat::rust_link(icu::timezone::IanaMapperBorrowed, Struct, hidden)]
    #[diplomat::rust_link(icu::timezone::IanaMapperBorrowed::new, FnInStruct, hidden)]
    #[diplomat::rust_link(icu::timezone::NormalizedIana, Struct, hidden)]
    pub struct IanaMapper(pub icu_timezone::IanaMapper);

    impl IanaMapper {
        /// Create a new [`IanaMapper`] using compiled data
        #[diplomat::rust_link(icu::timezone::IanaMapper::new, FnInStruct)]
        #[diplomat::attr(auto, constructor)]
        #[cfg(feature = "compiled_data")]
        pub fn create() -> Box<IanaMapper> {
            Box::new(IanaMapper(
                icu_timezone::IanaMapper::new().static_to_owned(),
            ))
        }

        /// Create a new [`IanaMapper`] using a particular data source
        #[diplomat::rust_link(icu::timezone::IanaMapper::new, FnInStruct)]
        #[diplomat::attr(all(supports = fallible_constructors, supports = named_constructors), named_constructor = "with_provider")]
        #[cfg(feature = "buffer_provider")]
        pub fn create_with_provider(
            provider: &DataProvider,
        ) -> Result<Box<IanaMapper>, DataError> {
            Ok(Box::new(IanaMapper(
                icu_timezone::IanaMapper::try_new_with_buffer_provider(provider.get()?)?,
            )))
        }

        #[diplomat::rust_link(icu::timezone::IanaMapperBorrowed::get, FnInStruct)]
        #[diplomat::rust_link(
            icu::timezone::IanaMapperBorrowed::get_utf8,
            FnInStruct,
            hidden
        )]
        pub fn get(
            &self,
            value: &DiplomatStr,
            write: &mut diplomat_runtime::DiplomatWrite,
        ) {
            let handle = self.0.as_borrowed();
            let bcp47 = handle.get_utf8(value);
            let _infallible = bcp47.0.write_to(write);
        }

        #[diplomat::rust_link(icu::timezone::IanaMapperBorrowed::normalize, FnInStruct)]
        pub fn normalize(
            &self,
            value: &str,
            write: &mut diplomat_runtime::DiplomatWrite,
        ) -> Option<()> {
            let handle = self.0.as_borrowed();
            let iana = handle.normalize(value)?;
            let _infallible = iana.0.write_to(write);
            Some(())
        }

        #[diplomat::rust_link(
            icu::timezone::IanaMapperBorrowed::canonicalize,
            FnInStruct
        )]
        pub fn canonicalize(
            &self,
            value: &str,
            write: &mut diplomat_runtime::DiplomatWrite,
        ) -> Option<()> {
            let handle = self.0.as_borrowed();
            let iana = handle.canonicalize(value)?;
            let _infallible = iana.0.write_to(write);
            Some(())
        }

        #[diplomat::rust_link(
            icu::timezone::IanaMapperBorrowed::find_canonical_iana,
            FnInStruct
        )]
        pub fn find_canonical_iana(
            &self,
            value: &DiplomatStr,
            write: &mut diplomat_runtime::DiplomatWrite,
        ) -> Option<()> {
            let handle = self.0.as_borrowed();
            let iana = TinyAsciiStr::try_from_utf8(value).ok().and_then(|s| {
                handle.find_canonical_iana(icu_timezone::TimeZone(s))
            })?;
            let _infallible = iana.write_to(write);
            Some(())
        }
    }

    /// A mapper between IANA time zone identifiers and BCP-47 time zone identifiers.
    ///
    /// This mapper supports two-way mapping, but it is optimized for the case of IANA to BCP-47.
    /// It also supports normalizing and canonicalizing the IANA strings.
    #[diplomat::opaque]
    #[diplomat::rust_link(icu::timezone::IanaMapperWithFastCanonicalization, Struct)]
    #[diplomat::rust_link(
        icu::timezone::IanaMapperWithFastCanonicalization::as_borrowed,
        FnInStruct,
        hidden
    )]
    #[diplomat::rust_link(
        icu::timezone::IanaMapperWithFastCanonicalization::inner,
        FnInStruct,
        hidden
    )]
    #[diplomat::rust_link(
        icu::timezone::IanaMapperWithFastCanonicalizationBorrowed,
        Struct,
        hidden
    )]
    #[diplomat::rust_link(
        icu::timezone::IanaMapperWithFastCanonicalizationBorrowed::inner,
        FnInStruct,
        hidden
    )]
    pub struct IanaMapperWithFastCanonicalization(
        pub icu_timezone::IanaMapperWithFastCanonicalization<icu_timezone::IanaMapper>,
    );

    impl IanaMapperWithFastCanonicalization {
        /// Create a new [`IanaMapperWithFastCanonicalization`] using compiled data
        #[diplomat::rust_link(
            icu::timezone::IanaMapperWithFastCanonicalization::new,
            FnInStruct
        )]
        #[diplomat::rust_link(
            icu::timezone::IanaMapperWithFastCanonicalizationBorrowed::new,
            FnInStruct
        )]
        #[diplomat::attr(auto, constructor)]
        #[cfg(feature = "compiled_data")]
        pub fn create() -> Box<IanaMapperWithFastCanonicalization> {
            Box::new(IanaMapperWithFastCanonicalization(
                icu_timezone::IanaMapperWithFastCanonicalization::new().static_to_owned(),
            ))
        }
        /// Create a new [`IanaMapperWithFastCanonicalization`] using a particular data source
        #[diplomat::rust_link(
            icu::timezone::IanaMapperWithFastCanonicalization::new,
            FnInStruct
        )]
        #[diplomat::rust_link(
            icu::timezone::IanaMapperWithFastCanonicalizationBorrowed::new,
            FnInStruct
        )]
        #[diplomat::attr(all(supports = fallible_constructors, supports = named_constructors), named_constructor = "with_provider")]
        #[cfg(feature = "buffer_provider")]
        pub fn create_with_provider(
            provider: &DataProvider,
        ) -> Result<Box<IanaMapperWithFastCanonicalization>, DataError> {
            Ok(Box::new(IanaMapperWithFastCanonicalization(
                icu_timezone::IanaMapperWithFastCanonicalization::try_new_with_buffer_provider(provider.get()?)?,
            )))
        }

        #[diplomat::rust_link(
            icu::timezone::IanaMapperWithFastCanonicalizationBorrowed::canonicalize,
            FnInStruct
        )]
        pub fn canonicalize(
            &self,
            value: &str,
            write: &mut diplomat_runtime::DiplomatWrite,
        ) -> Option<()> {
            let handle = self.0.as_borrowed();
            let iana = handle.canonicalize(value)?;
            let _infallible = iana.0.write_to(write);
            Some(())
        }

        #[diplomat::rust_link(
            icu::timezone::IanaMapperWithFastCanonicalizationBorrowed::find_canonical_iana,
            FnInStruct
        )]
        pub fn find_canonical_iana(
            &self,
            value: &DiplomatStr,
            write: &mut diplomat_runtime::DiplomatWrite,
        ) -> Option<()> {
            let handle = self.0.as_borrowed();
            let iana = TinyAsciiStr::try_from_utf8(value)
                .ok()
                .map(icu_timezone::TimeZone)
                .and_then(|t| handle.find_canonical_iana(t))?;
            let _infallible = iana.write_to(write);
            Some(())
        }
    }
}
