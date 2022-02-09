// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::error::Error;
use std::path::PathBuf;

/// A collection of filesystem paths to CLDR JSON resource directories.
///
/// # Examples
///
/// ```
/// use icu_provider_cldr::CldrPaths;
/// use icu_provider_cldr::CldrJsonDataProvider;
/// use std::path::PathBuf;
///
/// let paths = CldrPaths {
///     cldr_json_root: PathBuf::from("/path/to/cldr-json"),
///     locale_subset: "full".to_string(),
///     uprops_root: Some(PathBuf::from("path/to/uprops")),
/// };
///
/// let data_provider = CldrJsonDataProvider::try_new(&paths);
/// ```
#[derive(Debug, PartialEq)]
pub struct CldrPaths {
    /// Path to the CLDR JSON root directory
    pub cldr_json_root: PathBuf,
    /// CLDR JSON directory suffix: probably either "modern" or "full"
    pub locale_subset: String,
    /// Path to uprops TOML root directory. Required by some CLDR transformers
    pub uprops_root: Option<PathBuf>,
}

impl CldrPaths {
    /// Path to checkout of cldr-core:
    /// <https://github.com/unicode-cldr/cldr-core>
    pub fn cldr_core(&self) -> Result<PathBuf, Error> {
        Ok(self.cldr_json_root.clone().join("cldr-core"))
    }

    /// Path to checkout of cldr-dates:
    /// <https://github.com/unicode-cldr/cldr-dates-full>
    pub fn cldr_dates_gregorian(&self) -> Result<PathBuf, Error> {
        Ok(self
            .cldr_json_root
            .clone()
            .join(format!("cldr-dates-{}", self.locale_subset)))
    }

    /// Path to checkout of cldr-cal-buddhist:
    /// <https://github.com/unicode-cldr/cldr-cal-buddhist-full>
    pub fn cldr_dates_buddhist(&self) -> Result<PathBuf, Error> {
        Ok(self
            .cldr_json_root
            .clone()
            .join(format!("cldr-cal-buddhist-{}", self.locale_subset)))
    }

    /// Path to checkout of cldr-cal-japanese:
    /// <https://github.com/unicode-cldr/cldr-cal-japanese-full>
    pub fn cldr_dates_japanese(&self) -> Result<PathBuf, Error> {
        Ok(self
            .cldr_json_root
            .clone()
            .join(format!("cldr-cal-japanese-{}", self.locale_subset)))
    }

    /// Path to checkout of cldr-numbers:
    /// <https://github.com/unicode-cldr/cldr-numbers-full>
    pub fn cldr_numbers(&self) -> Result<PathBuf, Error> {
        Ok(self
            .cldr_json_root
            .clone()
            .join(format!("cldr-numbers-{}", self.locale_subset)))
    }

    /// Path to checkout of cldr-misc
    /// <https://github.com/unicode-cldr/cldr-misc-full>
    pub fn cldr_misc(&self) -> Result<PathBuf, Error> {
        Ok(self
            .cldr_json_root
            .clone()
            .join(format!("cldr-misc-{}", self.locale_subset)))
    }

    /// Returns a list of (CLDR name, BCP name, path) for each supported calendar
    pub fn cldr_dates_all(&self) -> Vec<(&'static str, &'static str, PathBuf)> {
        let mut vec = Vec::new();
        if let Ok(greg) = self.cldr_dates_gregorian() {
            vec.push(("gregorian", "gregory", greg));
        }
        if let Ok(bud) = self.cldr_dates_buddhist() {
            vec.push(("buddhist", "buddhist", bud));
        }
        if let Ok(jp) = self.cldr_dates_japanese() {
            vec.push(("japanese", "japanese", jp));
        }
        // TODO Japanese is not yet fully supported (#1116)
        // more calendars here
        vec
    }

    /// Path to uprops TOML data, which is required by some CLDR transformers
    pub fn uprops(&self) -> Result<PathBuf, Error> {
        self.uprops_root
            .clone()
            .ok_or_else(|| Error::Custom("The uprops root has not been set".to_owned(), None))
    }
}

#[cfg(test)]
pub(crate) fn for_test() -> CldrPaths {
    CldrPaths {
        cldr_json_root: icu_testdata::paths::cldr_json_root(),
        locale_subset: "full".to_string(),
        uprops_root: Some(icu_testdata::paths::uprops_toml_root()),
    }
}
