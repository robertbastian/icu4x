// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use icu_provider::prelude::*;
use writeable::{impl_display_with_writeable, Writeable};
use zerotrie::cursor::ZeroAsciiIgnoreCaseTrieCursor;
use zerovec::vecs::{VarZeroSliceIter, ZeroSliceIter};

use crate::{
    provider::names::{
        Bcp47ToIanaMapV1, Bcp47ToIanaMapV1Marker, IanaToBcp47MapV3, IanaToBcp47MapV3Marker,
        NON_REGION_CITY_PREFIX,
    },
    TimeZone,
};

/// A mapper between IANA time zone identifiers and BCP-47 time zone identifiers.
///
/// This mapper supports two-way mapping, but it is optimized for the case of IANA to BCP-47.
/// It also supports normalizing and canonicalizing the IANA strings.
///
/// There are approximately 600 IANA identifiers and 450 BCP-47 identifiers.
///
/// BCP-47 time zone identifiers are 8 ASCII characters or less and currently
/// average 5.1 characters long. Current IANA time zone identifiers are less than
/// 40 ASCII characters and average 14.2 characters long.
///
/// These lists grow very slowly; in a typical year, 2-3 new identifiers are added.
///
/// # Normalization vs Canonicalization
///
/// Multiple IANA time zone identifiers can refer to the same BCP-47 time zone. For example, the
/// following three IANA identifiers all map to `"usind"`:
///
/// - "America/Fort_Wayne"
/// - "America/Indiana/Indianapolis"
/// - "America/Indianapolis"
/// - "US/East-Indiana"
///
/// There is only one canonical identifier, which is "America/Indiana/Indianapolis". The
/// *canonicalization* operation returns the canonical identifier. You should canonicalize if
/// you need to compare time zones for equality. Note that the canonical identifier can change
/// over time. For example, the identifier "Europe/Kiev" was renamed to the newly-added
/// identifier "Europe/Kyiv" in 2022.
///
/// The *normalization* operation, on the other hand, keeps the input identifier but normalizes
/// the casing. For example, "AMERICA/FORT_WAYNE" normalizes to "America/Fort_Wayne".
/// Normalization is a data-driven operation because there are no algorithmic casing rules that
/// work for all IANA time zone identifiers.
///
/// Normalization is a cheap operation, but canonicalization might be expensive, since it might
/// require searching over all IANA IDs to find the canonicalization. If you need
/// canonicalization that is reliably fast, use [`IanaMapperWithFastCanonicalization`].
///
/// # Examples
///
/// ```
/// use icu::timezone::TimeZone;
/// use icu::timezone::IanaMapper;
/// use tinystr::tinystr;
///
/// let mapper = IanaMapper::new();
///
/// // The IANA zone "Australia/Melbourne" is the BCP-47 zone "aumel":
/// assert_eq!(
///     mapper.get("Australia/Melbourne"),
///     TimeZone(tinystr!(8, "aumel"))
/// );
///
/// // Lookup is ASCII-case-insensitive:
/// assert_eq!(
///     mapper.get("australia/melbourne"),
///     TimeZone(tinystr!(8, "aumel"))
/// );
///
/// // The IANA zone "Australia/Victoria" is an alias:
/// assert_eq!(
///     mapper.get("Australia/Victoria"),
///     TimeZone(tinystr!(8, "aumel"))
/// );
///
/// // The IANA zone "Australia/Boing_Boing" does not exist
/// // (maybe not *yet*), so it produces the special unknown
/// // timezone in order for this operation to be infallible:
/// assert_eq!(
///     mapper.get("Australia/Boing_Boing"),
///     TimeZone::unknown()
/// );
///
/// // We can recover the canonical identifier from the mapper:
/// assert_eq!(
///     mapper.canonicalize("Australia/Victoria").unwrap().0,
///     "Australia/Melbourne"
/// );
/// ```
#[derive(Debug, Clone)]
pub struct IanaMapper {
    data: DataPayload<IanaToBcp47MapV3Marker>,
}

impl IanaMapper {
    /// Creates a new [`IanaMapper`] using compiled data.
    ///
    /// See [`IanaMapper`] for an example.
    ///
    /// ✨ *Enabled with the `compiled_data` Cargo feature.*
    ///
    /// [📚 Help choosing a constructor](icu_provider::constructors)
    #[cfg(feature = "compiled_data")]
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> IanaMapperBorrowed<'static> {
        IanaMapperBorrowed::new()
    }

    icu_provider::gen_any_buffer_data_constructors!(() -> error: DataError,
        functions: [
            new: skip,
            try_new_with_any_provider,
            try_new_with_buffer_provider,
            try_new_unstable,
            Self,
        ]
    );

    #[doc = icu_provider::gen_any_buffer_unstable_docs!(UNSTABLE, Self::new)]
    pub fn try_new_unstable<P>(provider: &P) -> Result<Self, DataError>
    where
        P: DataProvider<IanaToBcp47MapV3Marker> + ?Sized,
    {
        let data = provider.load(Default::default())?.payload;
        Ok(Self { data })
    }

    /// Returns a borrowed version of the mapper that can be queried.
    ///
    /// This avoids a small potential indirection cost when querying the mapper.
    pub fn as_borrowed(&self) -> IanaMapperBorrowed {
        IanaMapperBorrowed {
            data: self.data.get(),
        }
    }
}

impl AsRef<IanaMapper> for IanaMapper {
    #[inline]
    fn as_ref(&self) -> &IanaMapper {
        self
    }
}

/// A borrowed wrapper around the time zone ID mapper, returned by
/// [`IanaMapper::as_borrowed()`]. More efficient to query.
#[derive(Debug, Copy, Clone)]
pub struct IanaMapperBorrowed<'a> {
    data: &'a IanaToBcp47MapV3<'a>,
}

#[cfg(feature = "compiled_data")]
impl Default for IanaMapperBorrowed<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl IanaMapperBorrowed<'static> {
    /// Creates a new [`IanaMapperBorrowed`] using compiled data.
    ///
    /// See [`IanaMapperBorrowed`] for an example.
    ///
    /// ✨ *Enabled with the `compiled_data` Cargo feature.*
    ///
    /// [📚 Help choosing a constructor](icu_provider::constructors)
    #[cfg(feature = "compiled_data")]
    pub fn new() -> Self {
        Self {
            data: crate::provider::Baked::SINGLETON_IANA_TO_BCP47_MAP_V3_MARKER,
        }
    }

    /// Cheaply converts a [`IanaMapperBorrowed<'static>`] into a [`IanaMapper`].
    ///
    /// Note: Due to branching and indirection, using [`IanaMapper`] might inhibit some
    /// compile-time optimizations that are possible with [`IanaMapperBorrowed`].
    pub fn static_to_owned(&self) -> IanaMapper {
        IanaMapper {
            data: DataPayload::from_static_ref(self.data),
        }
    }
}

impl IanaMapperBorrowed<'_> {
    /// Gets the BCP-47 time zone ID from an IANA time zone ID
    /// with a case-insensitive lookup.
    ///
    /// Returns [`TimeZone::unknown()`] if the IANA ID is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu_timezone::TimeZone;
    /// use icu_timezone::IanaMapper;
    ///
    /// let mapper = IanaMapper::new();
    ///
    /// let result = mapper.get("Asia/CALCUTTA");
    ///
    /// assert_eq!(*result, "inccu");
    ///
    /// // Unknown IANA time zone ID:
    /// assert_eq!(
    ///     mapper.get("America/San_Francisco"),
    ///     TimeZone::unknown()
    /// );
    /// ```
    pub fn get(&self, iana_id: &str) -> TimeZone {
        self.iana_lookup_quick(iana_id)
            .and_then(|trie_value| self.data.bcp47_ids.get(trie_value.index()))
            .unwrap_or(TimeZone::unknown())
    }

    /// Same as [`Self::get()`] but works with potentially ill-formed UTF-8.
    pub fn get_utf8(&self, iana_id: &[u8]) -> TimeZone {
        self.iana_lookup_quick(iana_id)
            .and_then(|trie_value| self.data.bcp47_ids.get(trie_value.index()))
            .unwrap_or(TimeZone::unknown())
    }

    /// Normalizes the syntax of an IANA time zone ID.
    ///
    /// Also returns the BCP-47 time zone ID.
    ///
    /// Returns `None` if the IANA ID is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu_timezone::TimeZone;
    /// use icu_timezone::IanaMapper;
    /// use std::borrow::Cow;
    ///
    /// let mapper = IanaMapper::new();
    ///
    /// let result = mapper.normalize("Asia/CALCUTTA").unwrap();
    ///
    /// assert_eq!(result.0, "Asia/Calcutta");
    /// assert!(matches!(result.0, Cow::Owned(_)));
    /// assert_eq!(*result.1, "inccu");
    ///
    /// // Borrows when able:
    /// let result = mapper.normalize("America/Chicago").unwrap();
    /// assert_eq!(result.0, "America/Chicago");
    /// assert!(matches!(result.0, Cow::Borrowed(_)));
    ///
    /// // Unknown IANA time zone ID:
    /// assert_eq!(mapper.normalize("America/San_Francisco"), None);
    /// ```
    pub fn normalize<'s>(&self, iana_id: &'s str) -> Option<(Cow<'s, str>, TimeZone)> {
        let (trie_value, string) = self.iana_lookup_with_normalization(iana_id, |_| {})?;
        let Some(bcp47_id) = self.data.bcp47_ids.get(trie_value.index()) else {
            debug_assert!(false, "index should be in range");
            return None;
        };
        Some((string, bcp47_id))
    }

    /// Returns the canonical, normalized identifier of the given IANA time zone.
    ///
    /// Also returns the BCP-47 time zone ID.
    ///
    /// Returns `None` if the IANA ID is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu_timezone::TimeZone;
    /// use icu_timezone::IanaMapper;
    /// use std::borrow::Cow;
    ///
    /// let mapper = IanaMapper::new();
    ///
    /// let result = mapper.canonicalize("Asia/CALCUTTA").unwrap();
    ///
    /// assert_eq!(result.0, "Asia/Kolkata");
    /// assert!(matches!(result.0, Cow::Owned(_)));
    /// assert_eq!(*result.1, "inccu");
    ///
    /// // Borrows when able:
    /// let result = mapper.canonicalize("America/Chicago").unwrap();
    /// assert_eq!(result.0, "America/Chicago");
    /// assert!(matches!(result.0, Cow::Borrowed(_)));
    ///
    /// // Unknown IANA time zone ID:
    /// assert_eq!(mapper.canonicalize("America/San_Francisco"), None);
    /// ```
    pub fn canonicalize<'s>(
        &self,
        iana_id: &'s str,
    ) -> Option<(Cow<'s, str>, TimeZone)> {
        // Note: We collect the cursors into a stack so that we start probing
        // nearby the input IANA identifier. This should improve lookup time since
        // most renames share the same prefix like "Asia" or "Europe".
        let mut stack = Vec::with_capacity(iana_id.len());
        let (trie_value, mut string) = self.iana_lookup_with_normalization(iana_id, |cursor| {
            stack.push((cursor.clone(), 0, 1));
        })?;
        let Some(bcp47_id) = self.data.bcp47_ids.get(trie_value.index()) else {
            debug_assert!(false, "index should be in range");
            return None;
        };
        if trie_value.is_canonical() {
            return Some((string, bcp47_id));
        }
        // If we get here, we need to walk the trie to find the canonical IANA ID.
        let needle = trie_value.to_canonical();
        if !string.contains('/') {
            string.to_mut().insert(0, '_');
        }
        let Some(string) = self.iana_search(needle, string.into_owned(), stack) else {
            debug_assert!(false, "every time zone should have a canonical IANA ID");
            return None;
        };
        Some((Cow::Owned(string), bcp47_id))
    }

    /// Returns the canonical, normalized IANA ID of the given BCP-47 ID.
    ///
    /// This function performs a linear search over all IANA IDs. If this is problematic, consider one of the
    /// following functions instead:
    ///
    /// 1. [`IanaMapperBorrowed::canonicalize()`]
    ///    is faster if you have an IANA ID.
    /// 2. [`IanaMapperWithFastCanonicalizationBorrowed::find_canonical_iana()`]
    ///    is faster, but it requires loading additional data
    ///    (see [`IanaMapperWithFastCanonicalization`]).
    ///
    /// Returns `None` if the BCP-47 ID is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu_timezone::TimeZone;
    /// use icu_timezone::IanaMapper;
    /// use std::borrow::Cow;
    /// use tinystr::tinystr;
    ///
    /// let mapper = IanaMapper::new();
    ///
    /// let bcp47_id = TimeZone(tinystr!(8, "inccu"));
    /// let result = mapper.find_canonical_iana(bcp47_id).unwrap();
    ///
    /// assert_eq!(result, "Asia/Kolkata");
    ///
    /// // Unknown BCP-47 time zone ID:
    /// let bcp47_id = TimeZone(tinystr!(8, "ussfo"));
    /// assert_eq!(mapper.find_canonical_iana(bcp47_id), None);
    /// ```
    pub fn find_canonical_iana(&self, bcp47_id: TimeZone) -> Option<String> {
        let index = self.data.bcp47_ids.binary_search(&bcp47_id).ok()?;
        let stack = alloc::vec![(self.data.map.cursor(), 0, 0)];
        let needle = IanaTrieValue::canonical_for_index(index);
        let string = self.iana_search(needle, String::new(), stack)?;
        Some(string)
    }

    /// Queries the data for `iana_id` without recording the normalized string.
    /// This is a fast, no-alloc lookup.
    fn iana_lookup_quick(self, iana_id: impl AsRef<[u8]>) -> Option<IanaTrieValue> {
        let mut cursor = self.data.map.cursor();
        let iana_id = iana_id.as_ref();
        if !iana_id.contains(&b'/') {
            cursor.step(NON_REGION_CITY_PREFIX);
        }
        for &b in iana_id {
            cursor.step(b);
        }
        cursor.take_value().map(IanaTrieValue)
    }

    /// Queries the data for `iana_id` while keeping track of the normalized string.
    /// This is a fast lookup, but it may require allocating memory.
    fn iana_lookup_with_normalization<'l, 's>(
        &'l self,
        iana_id: &'s str,
        mut cursor_fn: impl FnMut(&ZeroAsciiIgnoreCaseTrieCursor<'l>),
    ) -> Option<(IanaTrieValue, Cow<'s, str>)> {
        let mut cursor = self.data.map.cursor();
        if !iana_id.contains('/') {
            cursor_fn(&cursor);
            cursor.step(NON_REGION_CITY_PREFIX);
        }
        let mut string = Cow::Borrowed(iana_id);
        let mut i = 0;
        let trie_value = loop {
            cursor_fn(&cursor);
            let Some(&input_byte) = string.as_bytes().get(i) else {
                break cursor.take_value().map(IanaTrieValue);
            };
            let Some(matched_byte) = cursor.step(input_byte) else {
                break None;
            };
            if matched_byte != input_byte {
                // Safety: we write to input_byte farther down after performing safety checks.
                let Some(input_byte) = unsafe { string.to_mut().as_bytes_mut() }.get_mut(i) else {
                    debug_assert!(false, "the same index was just accessed earlier");
                    break None;
                };
                if !input_byte.is_ascii() {
                    debug_assert!(false, "non-ASCII input byte: {input_byte}");
                    break None;
                }
                if !matched_byte.is_ascii() {
                    debug_assert!(false, "non-ASCII matched byte: {matched_byte}");
                    break None;
                }
                // Safety: we just checked that both input_byte and matched_byte are ASCII,
                // so the buffer remains UTF-8 when we replace one with the other.
                *input_byte = matched_byte;
            }
            i += 1;
        }?;
        Some((trie_value, string))
    }

    /// Performs a reverse lookup by walking the trie with an optional start position.
    /// This is not a fast operation since it requires a linear search.
    fn iana_search(
        self,
        needle: IanaTrieValue,
        mut string: String,
        mut stack: Vec<(ZeroAsciiIgnoreCaseTrieCursor, usize, usize)>,
    ) -> Option<String> {
        loop {
            let Some((mut cursor, index, suffix_len)) = stack.pop() else {
                // Nothing left in the trie.
                return None;
            };
            // Check to see if there is a value at the current node.
            if let Some(candidate) = cursor.take_value().map(IanaTrieValue) {
                if candidate == needle {
                    // Success! Found what we were looking for.
                    return Some(string);
                }
            }
            // Now check for children of the current node.
            let mut sub_cursor = cursor.clone();
            if let Some(probe_result) = sub_cursor.probe(index) {
                // Found a child. Add the current byte edge to the string.
                if !probe_result.byte.is_ascii() {
                    debug_assert!(false, "non-ASCII probe byte: {}", probe_result.byte);
                    return None;
                }
                // Safety: the byte being added is ASCII as guarded above
                unsafe { string.as_mut_vec().push(probe_result.byte) };
                // Add the child to the stack, and also add back the current
                // node if there are more siblings to visit.
                if index + 1 < probe_result.total_siblings as usize {
                    stack.push((cursor, index + 1, suffix_len));
                    stack.push((sub_cursor, 0, 1));
                } else {
                    stack.push((sub_cursor, 0, suffix_len + 1));
                }
            } else {
                // No more children. Pop this node's bytes from the string.
                for _ in 0..suffix_len {
                    // Safety: we check that the bytes being removed are ASCII
                    let removed_byte = unsafe { string.as_mut_vec().pop() };
                    if let Some(removed_byte) = removed_byte {
                        if !removed_byte.is_ascii() {
                            debug_assert!(false, "non-ASCII removed byte: {removed_byte}");
                            // If we get here for some reason, `string` is not in a valid state,
                            // so to be extra safe, we can clear it.
                            string.clear();
                            return None;
                        }
                    } else {
                        debug_assert!(false, "could not remove another byte");
                        return None;
                    }
                }
            }
        }
    }

    /// Returns an iterator over BCP-47 time zone identifiers in alphabetical order.
    ///
    /// To iterate over canonical IANA time zone IDs, use
    /// [`IanaMapperWithFastCanonicalizationBorrowed::iter_canonical_iana()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::timezone::IanaMapper;
    ///
    /// let ids = IanaMapper::new()
    ///     .iter()
    ///     .skip(30)
    ///     .take(5)
    ///     .map(|id| id.to_string())
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(
    ///     ids,
    ///     &[
    ///         "arush",
    ///         "asppg",
    ///         "atvie",
    ///         "auadl",
    ///         "aubhq",
    ///     ]
    /// );
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = TimeZone> + '_ {
        TimeZoneIter {
            inner: self.data.bcp47_ids.iter(),
        }
    }
}

/// An iterator over BCP-47 time zone identifiers.
///
/// See [`IanaMapperBorrowed::iter()`]
#[derive(Debug)]
struct TimeZoneIter<'a> {
    inner: ZeroSliceIter<'a, TimeZone>,
}

impl Iterator for TimeZoneIter<'_> {
    type Item = TimeZone;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// A mapper that supplements [`IanaMapper`] with about 8 KB of additional data to
/// improve the performance of canonical IANA ID lookup.
///
/// The data in [`IanaMapper`] is optimized for IANA to BCP-47 lookup; the reverse
/// requires a linear walk over all ~600 IANA identifiers. The data added here allows for
/// constant-time mapping from BCP-47 to IANA.
#[derive(Debug, Clone)]
pub struct IanaMapperWithFastCanonicalization<I> {
    inner: I,
    data: DataPayload<Bcp47ToIanaMapV1Marker>,
}

impl IanaMapperWithFastCanonicalization<IanaMapper> {
    /// Creates a new [`IanaMapperWithFastCanonicalization`] using compiled data.
    ///
    /// See [`IanaMapperWithFastCanonicalization`] for an example.
    ///
    /// ✨ *Enabled with the `compiled_data` Cargo feature.*
    ///
    /// [📚 Help choosing a constructor](icu_provider::constructors)
    #[cfg(feature = "compiled_data")]
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> IanaMapperWithFastCanonicalizationBorrowed<'static> {
        IanaMapperWithFastCanonicalizationBorrowed::new()
    }

    icu_provider::gen_any_buffer_data_constructors!(() -> error: DataError,
        functions: [
            new: skip,
            try_new_with_any_provider,
            try_new_with_buffer_provider,
            try_new_unstable,
            Self,
        ]
    );

    #[doc = icu_provider::gen_any_buffer_unstable_docs!(UNSTABLE, Self::new)]
    pub fn try_new_unstable<P>(provider: &P) -> Result<Self, DataError>
    where
        P: DataProvider<IanaToBcp47MapV3Marker> + DataProvider<Bcp47ToIanaMapV1Marker> + ?Sized,
    {
        let mapper = IanaMapper::try_new_unstable(provider)?;
        Self::try_new_with_mapper_unstable(provider, mapper)
    }
}

impl<I> IanaMapperWithFastCanonicalization<I>
where
    I: AsRef<IanaMapper>,
{
    /// Creates a new [`IanaMapperWithFastCanonicalization`] using compiled data
    /// and a pre-existing [`IanaMapper`], which can be borrowed.
    ///
    /// See [`IanaMapperWithFastCanonicalization`] for an example.
    ///
    /// ✨ *Enabled with the `compiled_data` Cargo feature.*
    ///
    /// [📚 Help choosing a constructor](icu_provider::constructors)
    #[cfg(feature = "compiled_data")]
    pub fn try_new_with_mapper(mapper: I) -> Result<Self, DataError> {
        Self {
            inner: mapper,
            data: DataPayload::from_static_ref(
                crate::provider::Baked::SINGLETON_BCP47_TO_IANA_MAP_V1_MARKER,
            ),
        }
        .validated()
    }

    icu_provider::gen_any_buffer_data_constructors!((mapper: I) -> error: DataError,
        functions: [
            try_new_with_mapper: skip,
            try_new_with_mapper_with_any_provider,
            try_new_with_mapper_with_buffer_provider,
            try_new_with_mapper_unstable,
            Self,
        ]
    );

    #[doc = icu_provider::gen_any_buffer_unstable_docs!(UNSTABLE, Self::new)]
    pub fn try_new_with_mapper_unstable<P>(provider: &P, mapper: I) -> Result<Self, DataError>
    where
        P: DataProvider<IanaToBcp47MapV3Marker> + DataProvider<Bcp47ToIanaMapV1Marker> + ?Sized,
    {
        let data = provider.load(Default::default())?.payload;
        Self {
            inner: mapper,
            data,
        }
        .validated()
    }

    fn validated(self) -> Result<Self, DataError> {
        if self.inner.as_ref().data.get().bcp47_ids_checksum != self.data.get().bcp47_ids_checksum {
            return Err(
                DataErrorKind::InconsistentData(IanaToBcp47MapV3Marker::INFO)
                    .with_marker(Bcp47ToIanaMapV1Marker::INFO),
            );
        }
        Ok(self)
    }

    /// Gets the inner [`IanaMapper`] for performing queries.
    pub fn inner(&self) -> &IanaMapper {
        self.inner.as_ref()
    }

    /// Returns a borrowed version of the mapper that can be queried.
    ///
    /// This avoids a small potential indirection cost when querying the mapper.
    pub fn as_borrowed(&self) -> IanaMapperWithFastCanonicalizationBorrowed {
        IanaMapperWithFastCanonicalizationBorrowed {
            inner: self.inner.as_ref().as_borrowed(),
            data: self.data.get(),
        }
    }
}

/// A borrowed wrapper around the time zone ID mapper, returned by
/// [`IanaMapperWithFastCanonicalization::as_borrowed()`]. More efficient to query.
#[derive(Debug, Copy, Clone)]
pub struct IanaMapperWithFastCanonicalizationBorrowed<'a> {
    inner: IanaMapperBorrowed<'a>,
    data: &'a Bcp47ToIanaMapV1<'a>,
}

#[cfg(feature = "compiled_data")]
impl Default for IanaMapperWithFastCanonicalizationBorrowed<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl IanaMapperWithFastCanonicalizationBorrowed<'static> {
    /// Creates a new [`IanaMapperWithFastCanonicalizationBorrowed`] using compiled data.
    ///
    /// See [`IanaMapperWithFastCanonicalizationBorrowed`] for an example.
    ///
    /// ✨ *Enabled with the `compiled_data` Cargo feature.*
    ///
    /// [📚 Help choosing a constructor](icu_provider::constructors)
    #[cfg(feature = "compiled_data")]
    pub fn new() -> Self {
        const _: () = assert!(
            crate::provider::Baked::SINGLETON_IANA_TO_BCP47_MAP_V3_MARKER.bcp47_ids_checksum
                == crate::provider::Baked::SINGLETON_BCP47_TO_IANA_MAP_V1_MARKER.bcp47_ids_checksum,
        );
        Self {
            inner: IanaMapperBorrowed::new(),
            data: crate::provider::Baked::SINGLETON_BCP47_TO_IANA_MAP_V1_MARKER,
        }
    }

    /// Cheaply converts a [`IanaMapperWithFastCanonicalizationBorrowed<'static>`] into a [`IanaMapperWithFastCanonicalization`].
    ///
    /// Note: Due to branching and indirection, using [`IanaMapperWithFastCanonicalization`] might inhibit some
    /// compile-time optimizations that are possible with [`IanaMapperWithFastCanonicalizationBorrowed`].
    pub fn static_to_owned(&self) -> IanaMapperWithFastCanonicalization<IanaMapper> {
        IanaMapperWithFastCanonicalization {
            inner: self.inner.static_to_owned(),
            data: DataPayload::from_static_ref(self.data),
        }
    }
}

impl<'a> IanaMapperWithFastCanonicalizationBorrowed<'a> {
    /// Gets the inner [`IanaMapperBorrowed`] for performing queries.
    pub fn inner(&self) -> IanaMapperBorrowed<'a> {
        self.inner
    }

    /// Returns the canonical, normalized identifier of the given IANA time zone.
    ///
    /// Also returns the BCP-47 time zone ID.
    ///
    /// This is a faster version of [`IanaMapperBorrowed::canonicalize()`]
    /// and it always returns borrowed IANA strings, but it requires loading additional data
    /// (see [`IanaMapperWithFastCanonicalization`]).
    ///
    /// Returns `None` if the IANA ID is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu_timezone::TimeZone;
    /// use icu_timezone::IanaMapperWithFastCanonicalization;
    /// use std::borrow::Cow;
    ///
    /// let mapper = IanaMapperWithFastCanonicalization::new();
    ///
    /// let result = mapper.canonicalize("Asia/CALCUTTA").unwrap();
    ///
    /// // The Cow is always returned borrowed:
    /// assert_eq!(result.0, "Asia/Kolkata");
    /// assert_eq!(*result.1, "inccu");
    ///
    /// // Unknown IANA time zone ID:
    /// assert_eq!(mapper.canonicalize("America/San_Francisco"), None);
    /// ```
    pub fn canonicalize(
        &self,
        iana_id: &str,
    ) -> Option<(IanaIdBorrowed, TimeZone)> {
        let trie_value = self.inner.iana_lookup_quick(iana_id)?;
        let Some(bcp47_id) = self.inner.data.bcp47_ids.get(trie_value.index()) else {
            debug_assert!(false, "index should be in range");
            return None;
        };
        let Some(string) = self.data.canonical_iana_ids.get(trie_value.index()) else {
            debug_assert!(false, "index should be in range");
            return None;
        };
        Some((IanaIdBorrowed(string), bcp47_id))
    }

    /// Returns the canonical, normalized IANA ID of the given BCP-47 ID.
    ///
    /// This is a faster version of [`IanaMapperBorrowed::find_canonical_iana()`]
    /// and it always returns borrowed IANA strings, but it requires loading additional data
    /// (see [`IanaMapperWithFastCanonicalization`]).
    ///
    /// Returns `None` if the BCP-47 ID is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu_timezone::TimeZone;
    /// use icu_timezone::IanaMapperWithFastCanonicalization;
    /// use std::borrow::Cow;
    /// use tinystr::tinystr;
    ///
    /// let mapper = IanaMapperWithFastCanonicalization::new();
    ///
    /// let bcp47_id = TimeZone(tinystr!(8, "inccu"));
    /// let result = mapper.find_canonical_iana(bcp47_id).unwrap();
    ///
    /// // The Cow is always returned borrowed:
    /// assert_eq!(result, "Asia/Kolkata");
    ///
    /// // Unknown BCP-47 time zone ID:
    /// let bcp47_id = TimeZone(tinystr!(8, "ussfo"));
    /// assert_eq!(mapper.find_canonical_iana(bcp47_id), None);
    /// ```
    pub fn find_canonical_iana(
        &self,
        bcp47_id: TimeZone,
    ) -> Option<IanaIdBorrowed> {
        let index = self.inner.data.bcp47_ids.binary_search(&bcp47_id).ok()?;
        let Some(string) = self.data.canonical_iana_ids.get(index) else {
            debug_assert!(false, "index should be in range");
            return None;
        };
        Some(IanaIdBorrowed(string))
    }

    /// Returns an iterator over all canonical IANA time zone identifiers in an arbitrary, unstable order.
    ///
    /// To iterate over BCP-47 IDs, use [`IanaMapperBorrowed::iter()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::timezone::IanaMapperWithFastCanonicalization;
    ///
    /// let ids = IanaMapperWithFastCanonicalization::new()
    ///     .iter_canonical_iana()
    ///     .skip(30)
    ///     .take(5)
    ///     .map(|id| id.to_string())
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(
    ///     ids,
    ///     &[
    ///         "America/Argentina/Ushuaia",
    ///         "Pacific/Pago_Pago",
    ///         "Europe/Vienna",
    ///         "Australia/Adelaide",
    ///         "Australia/Broken_Hill",
    ///     ]
    /// );
    /// ```
    pub fn iter_canonical_iana(&self) -> impl Iterator<Item = IanaIdBorrowed> {
        CanonicalIanaIter {
            inner: self.data.canonical_iana_ids.iter(),
        }
    }
}

/// An iterator over canonical IANA time zone identifiers.
///
/// See [`IanaMapperWithFastCanonicalizationBorrowed::iter_canonical_iana()`]
#[derive(Debug)]
struct CanonicalIanaIter<'a> {
    inner: VarZeroSliceIter<'a, str>,
}

impl<'a> Iterator for CanonicalIanaIter<'a> {
    type Item = IanaIdBorrowed<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(IanaIdBorrowed)
    }
}

/// A time zone IANA ID.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IanaIdBorrowed<'a>(&'a str);

impl Writeable for IanaIdBorrowed<'_> {
    #[inline]
    fn write_to<W: fmt::Write + ?Sized>(&self, sink: &mut W) -> fmt::Result {
        self.0.write_to(sink)
    }
    #[inline]
    fn write_to_string(&self) -> Cow<str> {
        Cow::Borrowed(self.0)
    }
}

impl_display_with_writeable!(IanaIdBorrowed<'_>);

impl PartialEq<&str> for IanaIdBorrowed<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
struct IanaTrieValue(usize);

impl IanaTrieValue {
    #[inline]
    pub(crate) fn to_canonical(self) -> Self {
        Self(self.0 | 1)
    }
    #[inline]
    pub(crate) fn canonical_for_index(index: usize) -> Self {
        Self(index << 1).to_canonical()
    }
    #[inline]
    pub(crate) fn index(self) -> usize {
        self.0 >> 1
    }
    #[inline]
    pub(crate) fn is_canonical(self) -> bool {
        (self.0 & 0x1) != 0
    }
}
