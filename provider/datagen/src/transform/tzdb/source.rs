// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::source::AbstractFs;
use icu_provider::DataError;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use tzif::data::tzif::TzifData;

#[derive(Debug)]
pub(crate) struct TzifPaths(AbstractFs);

impl TzifPaths {
    pub(crate) fn new<T: AsRef<Path>>(root: T) -> Result<Self, DataError> {
        AbstractFs::new(root).map(Self)
    }

    pub(crate) fn read_and_parse(&self) -> Result<HashMap<String, TzifData>, DataError> {
        self.0
            .list(".", true)?
            .map(|path| -> Result<_, DataError> {
                let buf = self.0.read_to_buf(&format!("{path}"))?;
                let data = tzif::parse_tzif(&buf).map_err(|e| {
                    DataError::custom("TZif parse")
                        .with_display_context(&e)
                        .with_path_context(&path)
                })?;
                Ok((path, data))
            })
            .collect()
    }
}