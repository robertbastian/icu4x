// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

#![cfg_attr(not(any(test, doc)), no_std)]

//! Time and timezone functionality.
//!
//! This module is published as its own crate ([`icu_time`](https://docs.rs/icu_time/latest/icu_time/))
//! and as part of the [`icu`](https://docs.rs/icu/latest/icu/) crate. See the latter for more details on the ICU4X project.

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod provider;
pub mod scaffold;

#[cfg(feature = "ixdtf")]
mod ixdtf;
#[cfg(feature = "ixdtf")]
pub use ixdtf::ParseError;

pub mod zone;
#[doc(no_inline)]
pub use zone::{TimeZone, TimeZoneInfo};

mod types;
pub use types::{DateTime, Hour, Minute, Nanosecond, Second, Time, ZonedDateTime};
