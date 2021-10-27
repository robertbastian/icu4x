// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::error::Error;
use crate::reader::{get_langid_subdirectories, open_reader};
use crate::CldrPaths;
use icu_list::options::{Type, Width};
use icu_list::provider::*;
use icu_locid::LanguageIdentifier;
use icu_provider::iter::{IterableDataProviderCore, KeyedDataProvider};
use icu_provider::prelude::*;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::marker::PhantomData;

mod cldr_serde;

/// All keys that this module is able to produce.
pub const ALL_KEYS: [ResourceKey; 1] = [key::LIST_FORMAT_V1];

/// A data provider reading from CLDR JSON list rule files.
#[derive(PartialEq, Debug)]
pub struct ListProvider<'data> {
    data: Vec<(
        LanguageIdentifier,
        cldr_serde::list_patterns_json::LangListPatterns,
    )>,
    _phantom: PhantomData<&'data ()>, // placeholder for when we need the lifetime param
}

impl<'data> TryFrom<&dyn CldrPaths> for ListProvider<'data> {
    type Error = Error;
    fn try_from(cldr_paths: &dyn CldrPaths) -> Result<Self, Self::Error> {
        let mut data = vec![];
        for dir in get_langid_subdirectories(&cldr_paths.cldr_misc()?.join("main"))? {
            let path = dir.join("listPatterns.json");
            let mut resource: cldr_serde::list_patterns_json::Resource =
                serde_json::from_reader(open_reader(&path)?).map_err(|e| (e, path))?;
            data.append(&mut resource.main.0);
        }
        Ok(Self {
            data,
            _phantom: PhantomData,
        })
    }
}

impl<'data> KeyedDataProvider for ListProvider<'data> {
    fn supports_key(resc_key: &ResourceKey) -> Result<(), DataError> {
        key::LIST_FORMAT_V1.match_key(*resc_key)
    }
}

fn variants() -> Vec<Cow<'static, str>> {
    vec![
        (Type::And, Width::Wide),
        (Type::And, Width::Short),
        (Type::And, Width::Narrow),
        (Type::Or, Width::Wide),
        (Type::Or, Width::Short),
        (Type::Or, Width::Narrow),
        (Type::Unit, Width::Wide),
        (Type::Unit, Width::Short),
        (Type::Unit, Width::Narrow),
    ]
    .iter()
    .map(|(type_, width)| Cow::Owned(format!("{:?}-{:?}", type_, width)))
    .collect()
}

impl<'data> DataProvider<'data, ListFormatterPatternsV1Marker> for ListProvider<'data> {
    fn load_payload(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<'data, ListFormatterPatternsV1Marker>, DataError> {
        Self::supports_key(&req.resource_path.key)?;
        let langid = req.try_langid()?;
        let raw_patterns = match self.data.binary_search_by_key(&langid, |(lid, _)| lid) {
            Ok(idx) => &self.data[idx].1.list_patterns,
            Err(_) => return Err(DataError::MissingResourceOptions(req.clone())),
        };

        let variants = variants();
        let variant = req
            .resource_path
            .options
            .variant
            .as_ref()
            .unwrap_or(&variants[0]);

        let patterns = ListFormatterPattern::try_from(if variant == &variants[0] {
            &raw_patterns.standard
        } else if variant == &variants[1] {
            &raw_patterns.standard_short
        } else if variant == &variants[2] {
            &raw_patterns.standard_narrow
        } else if variant == &variants[3] {
            &raw_patterns.or
        } else if variant == &variants[4] {
            &raw_patterns.or_short
        } else if variant == &variants[5] {
            &raw_patterns.or_narrow
        } else if variant == &variants[6] {
            &raw_patterns.unit
        } else if variant == &variants[7] {
            &raw_patterns.unit_short
        } else if variant == &variants[8] {
            &raw_patterns.unit_narrow
        } else {
            return Err(DataError::MissingResourceOptions(req.clone()));
        })
        .unwrap();

        Ok(DataResponse {
            metadata: DataResponseMetadata {
                data_langid: req.resource_path.options.langid.clone(),
            },
            payload: Some(DataPayload::from_owned(ListFormatterPatternsV1 {
                patterns,
            })),
        })
    }
}

icu_provider::impl_dyn_provider!(ListProvider<'data>, {
    _ => ListFormatterPatternsV1Marker,
}, SERDE_SE, 'data);

impl<'data> IterableDataProviderCore for ListProvider<'data> {
    #[allow(clippy::needless_collect)] // https://github.com/rust-lang/rust-clippy/issues/7526
    fn supported_options_for_key(
        &self,
        _resc_key: &ResourceKey,
    ) -> Result<Box<dyn Iterator<Item = ResourceOptions>>, DataError> {
        let list: Vec<ResourceOptions> = self
            .data
            .iter()
            .map(|(l, _)| {
                let x: Vec<ResourceOptions> = variants()
                    .iter()
                    .map(|tag| ResourceOptions {
                        variant: Some(tag.clone()),
                        langid: Some(l.clone()),
                    })
                    .collect();
                x
            })
            .flatten()
            .collect();
        Ok(Box::new(list.into_iter()))
    }
}

impl<'a> TryFrom<&cldr_serde::list_patterns_json::ListPattern> for ListFormatterPattern<'a> {
    type Error = icu_list::error::Error;

    fn try_from(other: &cldr_serde::list_patterns_json::ListPattern) -> Result<Self, Self::Error> {
        Ok(Self::new(
            other.start.parse()?,
            other.middle.parse()?,
            other.end.parse()?,
            other.pair.parse()?,
        ))
    }
}

#[test]
fn test_basic() {
    use icu_locid_macros::langid;

    let cldr_paths = crate::cldr_paths::for_test();
    let provider = ListProvider::try_from(&cldr_paths as &dyn CldrPaths).unwrap();

    let fr_list: DataPayload<ListFormatterPatternsV1Marker> = provider
        .load_payload(&DataRequest {
            resource_path: ResourcePath {
                key: key::LIST_FORMAT_V1,
                options: ResourceOptions {
                    variant: Some(std::borrow::Cow::Borrowed("Or-Wide")),
                    langid: Some(langid!("fr")),
                },
            },
        })
        .unwrap()
        .take_payload()
        .unwrap();

    assert_eq!(
        fr_list.get().patterns.pair(),
        &"{0} ou {1}".parse().unwrap()
    );
}
