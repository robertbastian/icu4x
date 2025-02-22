// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use std::collections::HashSet;

use crate::SourceDataProvider;
use calendrical_calculations::islamic::{
    IslamicBasedMarker, ObservationalIslamicMarker, SaudiIslamicMarker,
};
use calendrical_calculations::iso;
use icu::calendar::provider::islamic::*;
use icu_provider::prelude::*;

const YEARS: i32 = 250;
const ISO_START: i32 = 1900;

fn load<IB: IslamicBasedMarker>() -> IslamicCache<'static> {
    let extended_start = IB::approximate_islamic_from_fixed(iso::fixed_from_iso(ISO_START, 1, 1));
    let extended_end = extended_start + YEARS;
    IslamicCache::compute_for::<IB>(extended_start..extended_end)
}

impl DataProvider<CalendarIslamicObservationalV1> for SourceDataProvider {
    fn load(
        &self,
        req: DataRequest,
    ) -> Result<DataResponse<CalendarIslamicObservationalV1>, DataError> {
        self.check_req::<CalendarIslamicObservationalV1>(req)?;
        let cache = load::<ObservationalIslamicMarker>();
        Ok(DataResponse {
            metadata: Default::default(),
            payload: DataPayload::from_owned(cache),
        })
    }
}

impl crate::IterableDataProviderCached<CalendarIslamicObservationalV1> for SourceDataProvider {
    fn iter_ids_cached(&self) -> Result<HashSet<DataIdentifierCow<'static>>, DataError> {
        Ok(HashSet::from_iter([Default::default()]))
    }
}

impl DataProvider<CalendarIslamicUmmalquraV1> for crate::SourceDataProvider {
    fn load(
        &self,
        req: DataRequest,
    ) -> Result<DataResponse<CalendarIslamicUmmalquraV1>, DataError> {
        self.check_req::<CalendarIslamicUmmalquraV1>(req)?;
        let cache = load::<SaudiIslamicMarker>();
        Ok(DataResponse {
            metadata: Default::default(),
            payload: DataPayload::from_owned(cache),
        })
    }
}

impl crate::IterableDataProviderCached<CalendarIslamicUmmalquraV1> for SourceDataProvider {
    fn iter_ids_cached(&self) -> Result<HashSet<DataIdentifierCow<'static>>, DataError> {
        Ok(HashSet::from_iter([Default::default()]))
    }
}
