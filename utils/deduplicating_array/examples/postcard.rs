// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

// This example demonstrates how to use deduplicating_array

#![no_main] // https://github.com/unicode-org/icu4x/issues/395

use deduplicating_array;

#[derive(serde::Serialize, serde::Deserialize)]
struct DataStruct {
    #[serde(with = "deduplicating_array")]
    coordinates: [(f64, f64); 5],
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DuplicatingDataStruct {
    coordinates: [(f64, f64); 5],
}

const COORDINATES: [(f64, f64); 5] = [
    (0.32847593438459383483949834, 0.57349389409237894504879231),
    (0.75935920345829340598345782, 0.48925083496234859234852085),
    (0.32847593438459383483949834, 0.57349389409237894504879231),
    (0.32847593438459383483949834, 0.57349389409237894504879231),
    (0.82934057230459813590813902, 0.72034572031957482035492034),
];

#[no_mangle]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let bytes = postcard::to_stdvec(&DataStruct {
        coordinates: COORDINATES,
    })
    .expect("Serialization should be successful");

    let duplicating_bytes = postcard::to_stdvec(&DuplicatingDataStruct {
        coordinates: COORDINATES,
    })
    .expect("Serialization should be successful");

    assert_eq!(bytes.len(), 53);
    assert_eq!(duplicating_bytes.len(), 80);

    let data: DataStruct = postcard::from_bytes(&bytes).expect("Valid bytes");

    let also_data: DuplicatingDataStruct =
        postcard::from_bytes(&duplicating_bytes).expect("Valid bytes");

    assert_eq!(data.coordinates, COORDINATES);
    assert_eq!(also_data.coordinates, COORDINATES);

    0
}
