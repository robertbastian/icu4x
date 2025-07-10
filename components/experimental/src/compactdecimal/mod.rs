// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Compact decimal

mod error;
mod format;
mod formatter;
mod options;
pub mod provider;

pub use error::ExponentError;
pub use formatter::CompactDecimalFormatter;
pub use formatter::CompactDecimalFormatterPreferences;
pub use options::CompactDecimalFormatterOptions;

/// Locale preferences used by this crate
pub mod preferences {
    #[doc(inline)]
    /// **This is a reexport of a type in [`icu::locale`](icu_locale_core::preferences::extensions::unicode::keywords)**.
    #[doc = "\n"] // prevent autoformatting
    pub use icu_locale_core::preferences::extensions::unicode::keywords::NumberingSystem;
}
