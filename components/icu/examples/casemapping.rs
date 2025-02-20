// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, missing_docs)] // https://github.com/rust-lang/rust-clippy/issues/13981

#![no_main] // https://github.com/unicode-org/icu4x/issues/395
icu_benchmark_macros::instrument!();
use icu_benchmark_macros::println;

use icu::casemap::CaseMapper;
use icu::locale::langid;

fn main() {
    let cm = CaseMapper::new();

    println!(
        r#"The uppercase of "hello world" is "{}""#,
        cm.uppercase_to_string("hello world", &langid!("und"))
    );
    println!(
        r#"The lowercase of "Γειά σου Κόσμε" is "{}""#,
        cm.lowercase_to_string("Γειά σου Κόσμε", &langid!("und"))
    );
}
