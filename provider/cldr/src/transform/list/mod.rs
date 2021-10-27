// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::error::Error;
use crate::reader::{get_langid_subdirectories, open_reader};
use crate::CldrPaths;
use icu_list::provider::*;
use icu_locid::LanguageIdentifier;
use icu_locid_macros::langid;
use icu_provider::iter::{IterableDataProviderCore, KeyedDataProvider};
use icu_provider::prelude::*;
use std::convert::TryFrom;
use std::marker::PhantomData;

mod cldr_serde;

/// All keys that this module is able to produce.
pub const ALL_KEYS: [ResourceKey; 3] = [
    key::LIST_FORMAT_AND_V1,
    key::LIST_FORMAT_OR_V1,
    key::LIST_FORMAT_UNIT_V1,
];

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
        key::LIST_FORMAT_AND_V1
            .match_key(*resc_key)
            .or_else(|_| key::LIST_FORMAT_OR_V1.match_key(*resc_key))
            .or_else(|_| key::LIST_FORMAT_UNIT_V1.match_key(*resc_key))
    }
}

impl<'data> DataProvider<'data, ListFormatterPatternsV1Marker> for ListProvider<'data> {
    fn load_payload(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<'data, ListFormatterPatternsV1Marker>, DataError> {
        Self::supports_key(&req.resource_path.key)?;
        let langid = req.try_langid()?;
        let data = match self.data.binary_search_by_key(&langid, |(lid, _)| lid) {
            Ok(idx) => &self.data[idx].1.list_patterns,
            Err(_) => return Err(DataError::MissingResourceOptions(req.clone())),
        };

        let mut patterns = if key::LIST_FORMAT_AND_V1
            .match_key(req.resource_path.key)
            .is_ok()
        {
            parse_and_patterns(data).unwrap()
        } else if key::LIST_FORMAT_OR_V1
            .match_key(req.resource_path.key)
            .is_ok()
        {
            parse_or_patterns(data).unwrap()
        } else if key::LIST_FORMAT_UNIT_V1
            .match_key(req.resource_path.key)
            .is_ok()
        {
            parse_unit_patterns(data).unwrap()
        } else {
            panic!("Cannot happen due to check in supports_key")
        };

        if langid.language == langid!("es").language {
            replace_es_special_cases(&mut patterns);
        }

        if langid.language == langid!("he").language {
            replace_he_special_cases(&mut patterns);
        }

        Ok(DataResponse {
            metadata: DataResponseMetadata {
                data_langid: req.resource_path.options.langid.clone(),
            },
            payload: Some(DataPayload::from_owned(patterns)),
        })
    }
}

fn replace_es_special_cases(pattern: &mut ListFormatterPatternsV1) {
    let simple_y = &"{0} y {1}".parse().unwrap();
    let e_before_y = &ConditionalListJoinerPattern::from_regex_and_strs(
        // Starts with i or (hi but not hia/hie)
        "i|hi([^ae]|$)",
        "{0} e {1}",
        "{0} y {1}",
    )
    .unwrap();

    // Replace all simple_ys with e_before_y.
    pattern.replace_patterns(simple_y, e_before_y);

    let simple_o = &"{0} o {1}".parse().unwrap();
    let u_before_o = &ConditionalListJoinerPattern::from_regex_and_strs(
        // Starts with o, ho, 8 (including 80, 800, ...), or 11 either alone or followed
        // by thousand groups and/or decimals (excluding e.g. 110, 1100, ...)
        r"o|ho|8|(11(\.?\d\d\d)*(,\d*)?([^\.,\d]|$))",
        "{0} u {1}",
        "{0} o {1}",
    )
    .unwrap();

    // Replace all simple_os with u_before_o.
    pattern.replace_patterns(simple_o, u_before_o);
}

fn replace_he_special_cases(pattern: &mut ListFormatterPatternsV1) {
    let simple_vav = &"{0} \u{05D5}{1}".parse().unwrap();
    let vav_dash_after_non_he = &ConditionalListJoinerPattern::from_regex_and_strs(
        // Starts with a non-Hebrew letter
        r"[^\p{IsHebrew}]",
        "{0} ו-{1}",
        "{0} ו{1}",
    )
    .unwrap();

    // Replace all simple_vavs with vav_dash_after_non_he.
    pattern.replace_patterns(simple_vav, vav_dash_after_non_he);
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
            // ur-IN has a buggy pattern ("{1}, {0}") which violates
            // our invariant that {0} is at index 0 (and rotates the output)
            .filter(|(l, _)| l != &icu_locid_macros::langid!("ur-IN"))
            .map(|(l, _)| ResourceOptions {
                variant: None,
                langid: Some(l.clone()),
            })
            .collect();
        Ok(Box::new(list.into_iter()))
    }
}

fn parse_and_patterns<'a>(
    raw: &cldr_serde::list_patterns_json::ListPatterns,
) -> Result<ListFormatterPatternsV1<'a>, icu_list::error::Error> {
    Ok(ListFormatterPatternsV1::new(
        raw.standard.start.parse()?,
        raw.standard.middle.parse()?,
        raw.standard.end.parse()?,
        raw.standard.pair.parse()?,
        raw.standard_short.start.parse()?,
        raw.standard_short.middle.parse()?,
        raw.standard_short.end.parse()?,
        raw.standard_short.pair.parse()?,
        raw.standard_narrow.start.parse()?,
        raw.standard_narrow.middle.parse()?,
        raw.standard_narrow.end.parse()?,
        raw.standard_narrow.pair.parse()?,
    ))
}

fn parse_or_patterns<'a>(
    raw: &cldr_serde::list_patterns_json::ListPatterns,
) -> Result<ListFormatterPatternsV1<'a>, icu_list::error::Error> {
    Ok(ListFormatterPatternsV1::new(
        raw.or.start.parse()?,
        raw.or.middle.parse()?,
        raw.or.end.parse()?,
        raw.or.pair.parse()?,
        raw.or_short.start.parse()?,
        raw.or_short.middle.parse()?,
        raw.or_short.end.parse()?,
        raw.or_short.pair.parse()?,
        raw.or_narrow.start.parse()?,
        raw.or_narrow.middle.parse()?,
        raw.or_narrow.end.parse()?,
        raw.or_narrow.pair.parse()?,
    ))
}

fn parse_unit_patterns<'a>(
    raw: &cldr_serde::list_patterns_json::ListPatterns,
) -> Result<ListFormatterPatternsV1<'a>, icu_list::error::Error> {
    Ok(ListFormatterPatternsV1::new(
        raw.unit.start.parse()?,
        raw.unit.middle.parse()?,
        raw.unit.end.parse()?,
        raw.unit.pair.parse()?,
        raw.unit_short.start.parse()?,
        raw.unit_short.middle.parse()?,
        raw.unit_short.end.parse()?,
        raw.unit_short.pair.parse()?,
        raw.unit_narrow.start.parse()?,
        raw.unit_narrow.middle.parse()?,
        raw.unit_narrow.end.parse()?,
        raw.unit_narrow.pair.parse()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use icu_list::options::Width;
    use icu_locid::LanguageIdentifier;
    use icu_locid_macros::langid;

    fn provide<'data>(
        lang: LanguageIdentifier,
        key: ResourceKey,
    ) -> DataPayload<'data, ListFormatterPatternsV1Marker> {
        let cldr_paths = crate::cldr_paths::for_test();
        let provider = ListProvider::try_from(&cldr_paths as &dyn CldrPaths).unwrap();
        provider
            .load_payload(&DataRequest {
                resource_path: ResourcePath {
                    key,
                    options: ResourceOptions {
                        variant: None,
                        langid: Some(lang),
                    },
                },
            })
            .unwrap()
            .take_payload()
            .unwrap()
    }

    #[test]
    fn test_basic() {
        assert_eq!(
            provide(langid!("fr"), key::LIST_FORMAT_OR_V1)
                .get()
                .end(Width::Wide),
            &"{0} ou {1}".parse().unwrap()
        );
    }

    #[test]
    fn test_spanish() {
        let y_parts = (" y ", "");
        let e_parts = (" e ", "");
        let o_parts = (" o ", "");
        let u_parts = (" u ", "");

        let payload_and = provide(langid!("es"), key::LIST_FORMAT_AND_V1);
        let and = &payload_and.get().end(Width::Wide);
        let payload_or = provide(langid!("es"), key::LIST_FORMAT_OR_V1);
        let or = &payload_or.get().end(Width::Wide);

        // ... y Mallorca
        assert_eq!(and.parts("Mallorca"), y_parts);
        // ... e Ibiza
        assert_eq!(and.parts("Ibiza"), e_parts);
        // ... e Hidalgo
        assert_eq!(and.parts("Hidalgo"), e_parts);
        // ... y Hierva
        assert_eq!(and.parts("Hierva"), y_parts);

        // ... o Ibiza
        assert_eq!(or.parts("Ibiza"), o_parts);
        // ... u Okinawa
        assert_eq!(or.parts("Okinawa"), u_parts);
        // ... u 8 más
        assert_eq!(or.parts("8 más"), u_parts);
        // ... u 8
        assert_eq!(or.parts("8"), u_parts);
        // ... u 87 más
        assert_eq!(or.parts("87 más"), u_parts);
        // ... u 87
        assert_eq!(or.parts("87"), u_parts);
        // ... u 11 más
        assert_eq!(or.parts("11 más"), u_parts);
        // ... u 11
        assert_eq!(or.parts("11"), u_parts);
        // ... o 110 más
        assert_eq!(or.parts("110 más"), o_parts);
        // ... o 110
        assert_eq!(or.parts("110"), o_parts);
        // ... o 11.000 más
        assert_eq!(or.parts("11.000 más"), u_parts);
        // ... o 11.000
        assert_eq!(or.parts("11.000"), u_parts);
        // ... o 11.000,92 más
        assert_eq!(or.parts("11.000,92 más"), u_parts);
        // ... o 11.000,92
        assert_eq!(or.parts("11.000,92"), u_parts);

        // Works for all es-* locales
        assert_eq!(
            provide(langid!("es-AR"), key::LIST_FORMAT_AND_V1)
                .get()
                .end(Width::Wide)
                .parts("Ibiza"),
            e_parts
        );
    }

    #[test]
    fn test_hebrew() {
        let vav_parts = (" ו", "");
        let vav_dash_parts = (" ו-", "");

        assert_eq!(
            provide(langid!("he"), key::LIST_FORMAT_AND_V1)
                .get()
                .end(Width::Wide)
                .parts("יפו"),
            vav_parts
        );

        assert_eq!(
            provide(langid!("he"), key::LIST_FORMAT_AND_V1)
                .get()
                .end(Width::Wide)
                .parts("Ibiza"),
            vav_dash_parts
        );
    }
}
