// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! This module contains the core transformer code from CLDR JSON to ICU4X Data Provider.
//!
//! Every ICU4X component should have its own private submodule and then export the types from here.

mod calendar;
mod datetime;
mod decimal;
#[cfg(feature = "icu_list")]
mod list;
mod locale_canonicalizer;
mod plurals;
mod time_zones;

use crate::CldrPaths;
use icu_provider::iter::IterableProvider;
use icu_provider::prelude::*;
use icu_provider::serde::SerializeMarker;

macro_rules! cldr_json_data_provider {
    ($($ident: ident: $type: ty,)+) => {
        #[derive(Debug)]
        pub struct CldrJsonDataProvider {
            $(
                $ident: $type,
            )+
        }

        impl CldrJsonDataProvider {
            pub fn try_new(cldr_paths: &CldrPaths) -> Result<Self, $crate::error::Error> {
                use std::convert::TryFrom;
                Ok(CldrJsonDataProvider {
                    $(
                        $ident: <$type>::try_from(cldr_paths)?,
                    )+
                })
            }
        }

        impl DynProvider<SerializeMarker> for CldrJsonDataProvider {
            fn load_payload(
                &self,
                key: ResourceKey,
                req: &DataRequest,
            ) -> Result<DataResponse<SerializeMarker>, DataError> {
                $(
                    match DynProvider::load_payload(&self.$ident, key, req) {
                        Err(DataError { kind: DataErrorKind::MissingResourceKey, ..}) => {}
                        r => return r,
                    }
                )+
                Err(DataErrorKind::MissingResourceKey.with_req(key, req))
            }
        }

        impl IterableProvider<SerializeMarker> for CldrJsonDataProvider {
            fn supported_options_for_key(
                &self,
                key: &ResourceKey,
            ) -> Result<Box<dyn Iterator<Item = ResourceOptions> + '_>, DataError> {
                $(
                    match IterableProvider::supported_options_for_key(&self.$ident, key) {
                        Err(DataError { kind: DataErrorKind::MissingResourceKey, ..}) => {}
                        r => return r,
                    }
                )+
                Err(DataErrorKind::MissingResourceKey.with_key(*key))
            }
        }
    };
}

#[cfg(feature = "icu_list")]
cldr_json_data_provider!(
    aliases: locale_canonicalizer::aliases::AliasesProvider,
    date_symbols: datetime::symbols::DateSymbolsProvider,
    date_skeletons: datetime::skeletons::DateSkeletonPatternsProvider,
    date_patterns: datetime::patterns::DatePatternsProvider,
    japanese: calendar::japanese::JapaneseErasProvider,
    likelysubtags: locale_canonicalizer::likely_subtags::LikelySubtagsProvider,
    numbers: decimal::NumbersProvider,
    plurals: plurals::PluralsProvider,
    time_zones: time_zones::TimeZonesProvider,
    list: list::ListProvider,
);

#[cfg(not(feature = "icu_list"))]
cldr_json_data_provider!(
    aliases: locale_canonicalizer::aliases::AliasesProvider,
    date_symbols: datetime::symbols::DateSymbolsProvider,
    date_skeletons: datetime::skeletons::DateSkeletonPatternsProvider,
    date_patterns: datetime::patterns::DatePatternsProvider,
    japanese: calendar::japanese::JapaneseErasProvider,
    likelysubtags: locale_canonicalizer::likely_subtags::LikelySubtagsProvider,
    numbers: decimal::NumbersProvider,
    plurals: plurals::PluralsProvider,
    time_zones: time_zones::TimeZonesProvider,
);

pub const ALL_KEYS: [ResourceKey; if cfg!(feature = "icu_list") { 18 } else { 15 }] = [
    icu_calendar::provider::JapaneseErasV1Marker::KEY,
    icu_datetime::provider::calendar::DatePatternsV1Marker::KEY,
    icu_datetime::provider::calendar::DateSkeletonPatternsV1Marker::KEY,
    icu_datetime::provider::calendar::DateSymbolsV1Marker::KEY,
    icu_datetime::provider::time_zones::TimeZoneFormatsV1Marker::KEY,
    icu_datetime::provider::time_zones::ExemplarCitiesV1Marker::KEY,
    icu_datetime::provider::time_zones::MetaZoneGenericNamesLongV1Marker::KEY,
    icu_datetime::provider::time_zones::MetaZoneGenericNamesShortV1Marker::KEY,
    icu_datetime::provider::time_zones::MetaZoneSpecificNamesLongV1Marker::KEY,
    icu_datetime::provider::time_zones::MetaZoneSpecificNamesShortV1Marker::KEY,
    icu_decimal::provider::DecimalSymbolsV1Marker::KEY,
    #[cfg(feature = "icu_list")]
    icu_list::provider::AndListV1Marker::KEY,
    #[cfg(feature = "icu_list")]
    icu_list::provider::OrListV1Marker::KEY,
    #[cfg(feature = "icu_list")]
    icu_list::provider::UnitListV1Marker::KEY,
    icu_locale_canonicalizer::provider::AliasesV1Marker::KEY,
    icu_locale_canonicalizer::provider::LikelySubtagsV1Marker::KEY,
    icu_plurals::provider::CardinalV1Marker::KEY,
    icu_plurals::provider::OrdinalV1Marker::KEY,
];
