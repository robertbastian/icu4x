// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::error::{DataError, DataErrorKind};

use crate::fallback::LocaleFallbackConfig;
use alloc::borrow::Cow;
use core::fmt;
use core::fmt::Write;
use core::ops::Deref;
use writeable::{LengthHint, Writeable};
use zerovec::ule::*;

/// 128-bit magic number
pub const LEADING_TAG: [u8; 16] = *b"icu4x_marker_tag";

/// A compact hash of a [`DataMarkerInfo`]. Useful for markers in maps.
///
/// The hash will be stable over time within major releases.
#[derive(Debug, Eq, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct DataMarkerPathHash([u8; 20]);

impl DataMarkerPathHash {
    const fn new(path: &str) -> Self {
        Self::tagged(fxhash_32(path.as_bytes()).to_le_bytes())
    }

    const fn tagged(hash: [u8; 4]) -> Self {
        Self([
            LEADING_TAG[0],
            LEADING_TAG[1],
            LEADING_TAG[2],
            LEADING_TAG[3],
            LEADING_TAG[4],
            LEADING_TAG[5],
            LEADING_TAG[6],
            LEADING_TAG[7],
            LEADING_TAG[8],
            LEADING_TAG[9],
            LEADING_TAG[10],
            LEADING_TAG[11],
            LEADING_TAG[12],
            LEADING_TAG[13],
            LEADING_TAG[14],
            LEADING_TAG[15],
            hash[0],
            hash[1],
            hash[2],
            hash[3],
        ])
    }

    /// Gets the hash value as a byte array.
    pub const fn to_bytes(self) -> [u8; 4] {
        [self.0[16], self.0[17], self.0[18], self.0[19]]
    }
}

impl PartialEq for DataMarkerPathHash {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl PartialOrd for DataMarkerPathHash {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.to_bytes().cmp(&other.to_bytes()))
    }
}

impl Ord for DataMarkerPathHash {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.to_bytes().cmp(&other.to_bytes())
    }
}

impl core::hash::Hash for DataMarkerPathHash {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.to_bytes().hash(state)
    }
}

/// Const function to compute the FxHash of a byte array.
///
/// FxHash is a speedy hash algorithm used within rustc. The algorithm is satisfactory for our
/// use case since the strings being hashed originate from a trusted source (the ICU4X
/// components), and the hashes are computed at compile time, so we can check for collisions.
///
/// We could have considered a SHA or other cryptographic hash function. However, we are using
/// FxHash because:
///
/// 1. There is precedent for this algorithm in Rust
/// 2. The algorithm is easy to implement as a const function
/// 3. The amount of code is small enough that we can reasonably keep the algorithm in-tree
/// 4. FxHash is designed to output 32-bit or 64-bit values, whereas SHA outputs more bits,
///    such that truncation would be required in order to fit into a u32, partially reducing
///    the benefit of a cryptographically secure algorithm
// The indexing operations in this function have been reviewed in detail and won't panic.
#[allow(clippy::indexing_slicing)]
const fn fxhash_32(bytes: &[u8]) -> u32 {
    // This code is adapted from https://github.com/rust-lang/rustc-hash,
    // whose license text is reproduced below.
    //
    // Copyright 2015 The Rust Project Developers. See the COPYRIGHT
    // file at the top-level directory of this distribution and at
    // http://rust-lang.org/COPYRIGHT.
    //
    // Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
    // http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
    // <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
    // option. This file may not be copied, modified, or distributed
    // except according to those terms.

    #[inline]
    const fn hash_word_32(mut hash: u32, word: u32) -> u32 {
        const ROTATE: u32 = 5;
        const SEED32: u32 = 0x9e_37_79_b9;
        hash = hash.rotate_left(ROTATE);
        hash ^= word;
        hash = hash.wrapping_mul(SEED32);
        hash
    }

    let mut cursor = 0;
    let mut hash = 0;

    while bytes.len() - cursor >= 4 {
        let word = u32::from_le_bytes([
            bytes[cursor],
            bytes[cursor + 1],
            bytes[cursor + 2],
            bytes[cursor + 3],
        ]);
        hash = hash_word_32(hash, word);
        cursor += 4;
    }

    if bytes.len() - cursor >= 2 {
        let word = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
        hash = hash_word_32(hash, word as u32);
        cursor += 2;
    }

    if bytes.len() - cursor >= 1 {
        hash = hash_word_32(hash, bytes[cursor] as u32);
    }

    hash
}

impl<'a> zerovec::maps::ZeroMapKV<'a> for DataMarkerPathHash {
    type Container = zerovec::ZeroVec<'a, DataMarkerPathHash>;
    type Slice = zerovec::ZeroSlice<DataMarkerPathHash>;
    type GetType = <DataMarkerPathHash as AsULE>::ULE;
    type OwnedType = DataMarkerPathHash;
}

impl AsULE for DataMarkerPathHash {
    type ULE = RawBytesULE<4>;
    #[inline]
    fn to_unaligned(self) -> Self::ULE {
        RawBytesULE(self.to_bytes())
    }
    #[inline]
    fn from_unaligned(unaligned: Self::ULE) -> Self {
        Self::tagged(unaligned.0)
    }
}

// Safe since the ULE type is `self`.
unsafe impl EqULE for DataMarkerPathHash {}

/// The string path of a data marker. For example, "foo@1"
///
/// ```
/// # use icu_provider::DataMarkerPath;
/// const K: DataMarkerPath = icu_provider::data_marker_path!("foo/bar@1");
/// ```
///
/// The human-readable path string ends with `@` followed by one or more digits (the version
/// number). Paths do not contain characters other than ASCII letters and digits, `_`, `/`.
///
/// Invalid paths are compile-time errors (as [`data_marker_path!`](crate::data_marker_path) uses `const`).
///
/// ```compile_fail,E0080
/// # use icu_provider::DataMarkerPath;
/// const K: DataMarkerPath = icu_provider::data_marker_path!("foo/../bar@1");
/// ```
#[derive(Debug, Copy, Clone, Eq)]
pub struct DataMarkerPath {
    path: &'static str,
    hash: DataMarkerPathHash,
}

impl PartialEq for DataMarkerPath {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.path == other.path
    }
}

impl Ord for DataMarkerPath {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.path.cmp(other.path)
    }
}

impl PartialOrd for DataMarkerPath {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.path.cmp(other.path))
    }
}

impl core::hash::Hash for DataMarkerPath {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state)
    }
}

impl DataMarkerPath {
    #[doc(hidden)]
    // macro use
    // Error is a str of the expected character class and the index where it wasn't encountered
    // The indexing operations in this function have been reviewed in detail and won't panic.
    pub const fn construct_internal(path: &'static str) -> Result<Self, (&'static str, usize)> {
        // Regex: [a-zA-Z0-9_][a-zA-Z0-9_/]*@[0-9]+
        enum State {
            Empty,
            Body,
            At,
            Version,
        }
        use State::*;
        let mut i = 0;
        let mut state = Empty;
        loop {
            let byte = if i < path.len() {
                #[allow(clippy::indexing_slicing)] // protected by debug assertion
                Some(path.as_bytes()[i])
            } else {
                None
            };
            state = match (state, byte) {
                (Empty | Body, Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')) => Body,
                (Body, Some(b'/')) => Body,
                (Body, Some(b'@')) => At,
                (At | Version, Some(b'0'..=b'9')) => Version,
                // One of these cases will be hit at the latest when i == end, so the loop converges.
                (Version, None) => break,

                (Empty, _) => return Err(("[a-zA-Z0-9_]", i)),
                (Body, _) => return Err(("[a-zA-z0-9_/@]", i)),
                (At, _) => return Err(("[0-9]", i)),
                (Version, _) => return Err(("[0-9]", i)),
            };
            i += 1;
        }

        let hash = DataMarkerPathHash::new(path);

        Ok(Self { path, hash })
    }

    /// Gets a platform-independent hash of a [`DataMarkerPath`].
    ///
    /// The hash is 4 bytes and allows for fast comparison.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_provider::DataMarkerPath;
    /// use icu_provider::DataMarkerPathHash;
    ///
    /// const PATH: DataMarkerPath = icu_provider::data_marker_path!("foo@1");
    /// const PATH_HASH: DataMarkerPathHash = PATH.hashed();
    ///
    /// assert_eq!(PATH_HASH.to_bytes(), [0xe2, 0xb6, 0x17, 0x71]);
    /// ```
    #[inline]
    pub const fn hashed(self) -> DataMarkerPathHash {
        self.hash
    }
}

impl Deref for DataMarkerPath {
    type Target = str;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.path
    }
}

/// Used for loading data from a dynamic ICU4X data provider.
///
/// A data marker is tightly coupled with the code that uses it to load data at runtime.
/// Executables can be searched for `DataMarkerInfo` instances to produce optimized data files.
/// Therefore, users should not generally create DataMarkerInfo instances; they should instead use
/// the ones exported by a component.
#[derive(Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DataMarkerInfo {
    /// The human-readable path string ends with `@` followed by one or more digits (the version
    /// number). Paths do not contain characters other than ASCII letters and digits, `_`, `/`.
    ///
    /// Useful for reading and writing data to a file system.
    pub path: DataMarkerPath,
    /// TODO
    pub is_singleton: bool,
    /// TODO
    pub fallback_config: LocaleFallbackConfig,
}

impl PartialOrd for DataMarkerInfo {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.path.cmp(&other.path))
    }
}

impl Ord for DataMarkerInfo {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

impl core::hash::Hash for DataMarkerInfo {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state)
    }
}

impl DataMarkerInfo {
    /// See [`Default::default`]
    pub const fn from_path(path: DataMarkerPath) -> Self {
        Self {
            path,
            is_singleton: false,
            fallback_config: LocaleFallbackConfig::const_default(),
        }
    }

    /// Returns [`Ok`] if this data marker matches the argument, or the appropriate error.
    ///
    /// Convenience method for data providers that support a single [`DataMarkerInfo`].
    ///
    /// # Examples
    ///
    /// ```
    /// use icu_provider::prelude::*;
    /// use icu_provider::hello_world::*;
    /// # struct DummyMarker;
    /// # impl DynamicDataMarker for DummyMarker {
    /// #     type Yokeable = <HelloWorldV1Marker as DynamicDataMarker>::Yokeable;
    /// # }
    /// # impl DataMarker for DummyMarker {
    /// #     const INFO: DataMarkerInfo = DataMarkerInfo::from_path(icu_provider::data_marker_path!("dummy@1"));
    /// # }
    ///
    /// assert!(matches!(HelloWorldV1Marker::INFO.match_marker(HelloWorldV1Marker::INFO), Ok(())));
    /// assert!(matches!(
    ///     HelloWorldV1Marker::INFO.match_marker(DummyMarker::INFO),
    ///     Err(DataError {
    ///         kind: DataErrorKind::MissingDataMarker,
    ///         ..
    ///     })
    /// ));
    ///
    /// // The error context contains the argument:
    /// assert_eq!(HelloWorldV1Marker::INFO.match_marker(DummyMarker::INFO).unwrap_err().marker, Some(DummyMarker::INFO.path));
    /// ```
    pub fn match_marker(self, marker: Self) -> Result<(), DataError> {
        if self == marker {
            Ok(())
        } else {
            Err(DataErrorKind::MissingDataMarker.with_marker(marker))
        }
    }
}

/// See [`DataMarkerInfo`].
#[macro_export]
macro_rules! data_marker_path {
    ($path:expr) => {{
        // Force the DataMarkerInfo into a const context
        const X: $crate::DataMarkerPath = match $crate::DataMarkerPath::construct_internal($path) {
            Ok(path) => path,
            #[allow(clippy::panic)] // Const context
            Err(_) => panic!(concat!("Invalid path: ", $path)),
            // TODO Once formatting is const:
            // Err((expected, index)) => panic!(
            //     "Invalid data marker path {:?}: expected {:?}, found {:?} ",
            //     $path,
            //     expected,
            //     $path.get(index..))
            // );
        };
        X
    }};
}

impl fmt::Debug for DataMarkerInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("DataMarkerInfo{")?;
        fmt::Display::fmt(self, f)?;
        f.write_char('}')?;
        Ok(())
    }
}

impl Writeable for DataMarkerInfo {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        self.path.write_to(sink)
    }

    fn writeable_length_hint(&self) -> LengthHint {
        self.path.writeable_length_hint()
    }

    fn write_to_string(&self) -> Cow<str> {
        Cow::Borrowed(&*self.path)
    }
}

writeable::impl_display_with_writeable!(DataMarkerInfo);

#[test]
fn test_path_syntax() {
    // Valid paths:
    DataMarkerPath::construct_internal("hello/world@1").unwrap();
    DataMarkerPath::construct_internal("hello/world/foo@1").unwrap();
    DataMarkerPath::construct_internal("hello/world@999").unwrap();
    DataMarkerPath::construct_internal("hello_world/foo@1").unwrap();
    DataMarkerPath::construct_internal("hello_458/world@1").unwrap();
    DataMarkerPath::construct_internal("hello_world@1").unwrap();

    // No version:
    assert_eq!(
        DataMarkerPath::construct_internal("hello/world"),
        Err(("[a-zA-z0-9_/@]", "hello/world".len()))
    );

    assert_eq!(
        DataMarkerPath::construct_internal("hello/world@"),
        Err(("[0-9]", "hello/world@".len()))
    );
    assert_eq!(
        DataMarkerPath::construct_internal("hello/world@foo"),
        Err(("[0-9]", "hello/world@".len()))
    );
    assert_eq!(
        DataMarkerPath::construct_internal("hello/world@1foo"),
        Err(("[0-9]", "hello/world@1".len()))
    );

    // Meta no longer accepted:
    assert_eq!(
        DataMarkerPath::construct_internal("foo@1[R]"),
        Err(("[0-9]", "foo@1".len()))
    );
    assert_eq!(
        DataMarkerPath::construct_internal("foo@1[u-ca]"),
        Err(("[0-9]", "foo@1".len()))
    );
    assert_eq!(
        DataMarkerPath::construct_internal("foo@1[R][u-ca]"),
        Err(("[0-9]", "foo@1".len()))
    );

    // Invalid meta:
    assert_eq!(
        DataMarkerPath::construct_internal("foo@1[U]"),
        Err(("[0-9]", "foo@1".len()))
    );
    assert_eq!(
        DataMarkerPath::construct_internal("foo@1[uca]"),
        Err(("[0-9]", "foo@1".len()))
    );
    assert_eq!(
        DataMarkerPath::construct_internal("foo@1[u-"),
        Err(("[0-9]", "foo@1".len()))
    );
    assert_eq!(
        DataMarkerPath::construct_internal("foo@1[u-caa]"),
        Err(("[0-9]", "foo@1".len()))
    );
    assert_eq!(
        DataMarkerPath::construct_internal("foo@1[R"),
        Err(("[0-9]", "foo@1".len()))
    );

    // Invalid characters:
    assert_eq!(
        DataMarkerPath::construct_internal("你好/世界@1"),
        Err(("[a-zA-Z0-9_]", 0))
    );
}

#[test]
fn test_path_to_string() {
    struct TestCase {
        pub path: DataMarkerPath,
        pub expected: &'static str,
    }

    for cas in [
        TestCase {
            path: data_marker_path!("core/cardinal@1"),
            expected: "core/cardinal@1",
        },
        TestCase {
            path: data_marker_path!("core/maxlengthsubcatg@1"),
            expected: "core/maxlengthsubcatg@1",
        },
        TestCase {
            path: data_marker_path!("core/cardinal@65535"),
            expected: "core/cardinal@65535",
        },
    ] {
        assert_eq!(cas.expected, &*cas.path);
    }
}

#[test]
fn test_hash_word_32() {
    assert_eq!(0, fxhash_32(b"", 0, 0));
    assert_eq!(0, fxhash_32(b"a", 1, 0));
    assert_eq!(0, fxhash_32(b"a", 0, 1));
    assert_eq!(0, fxhash_32(b"a", 0, 10));
    assert_eq!(0, fxhash_32(b"a", 10, 0));
    assert_eq!(0, fxhash_32(b"a", 1, 1));
    assert_eq!(0xF3051F19, fxhash_32(b"a", 0, 0));
    assert_eq!(0x2F9DF119, fxhash_32(b"ab", 0, 0));
    assert_eq!(0xCB1D9396, fxhash_32(b"abc", 0, 0));
    assert_eq!(0x8628F119, fxhash_32(b"abcd", 0, 0));
    assert_eq!(0xBEBDB56D, fxhash_32(b"abcde", 0, 0));
    assert_eq!(0x1CE8476D, fxhash_32(b"abcdef", 0, 0));
    assert_eq!(0xC0F176A4, fxhash_32(b"abcdefg", 0, 0));
    assert_eq!(0x09AB476D, fxhash_32(b"abcdefgh", 0, 0));
    assert_eq!(0xB72F5D88, fxhash_32(b"abcdefghi", 0, 0));
}

#[test]
fn test_path_hash() {
    struct TestCase {
        pub path: DataMarkerPath,
        pub hash: DataMarkerPathHash,
    }

    for cas in [
        TestCase {
            path: data_marker_path!("core/cardinal@1"),
            hash: DataMarkerPathHash::tagged([172, 207, 42, 236]),
        },
        TestCase {
            path: data_marker_path!("core/maxlengthsubcatg@1"),
            hash: DataMarkerPathHash::tagged([193, 6, 79, 61]),
        },
        TestCase {
            path: data_marker_path!("core/cardinal@65535"),
            hash: DataMarkerPathHash::tagged([176, 131, 182, 223]),
        },
    ] {
        assert_eq!(cas.hash, cas.path.hashed(), "{}", &cas.path as &str);
    }
}
