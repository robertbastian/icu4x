// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::FormattedStringError;
use alloc::borrow::ToOwned;
use alloc::collections::vec_deque::VecDeque;
use core::str;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LocationInPart {
    Begin,
    Extend,
}

/// A string with L levels of annotations of type F. For N = 0, this is
/// implemented for `&str`, for higher N see LayeredFormattedString.
pub trait FormattedStringLike<'a, F: Copy, const L: usize> {
    fn fields_at(&self, pos: usize) -> [F; L] {
        self.annotation_at(pos).map(|(_, field)| field)
    }

    fn is_field_start(&self, pos: usize, level: usize) -> bool {
        assert!(level < L);
        let (location, _) = self.annotation_at(pos)[level];
        location == LocationInPart::Begin
    }

    // These members are not part of the public API.
    #[doc(hidden)]
    fn annotation_at(&self, pos: usize) -> &[(LocationInPart, F); L];
    #[doc(hidden)]
    type BytesIter: DoubleEndedIterator<Item = u8> + 'a;
    #[doc(hidden)]
    fn bytes(&'a self) -> Self::BytesIter;
    #[doc(hidden)]
    fn len(&self) -> usize;
}

impl<'a, F: Copy> FormattedStringLike<'a, F, 0> for &'a str {
    fn annotation_at(&self, _pos: usize) -> &[(LocationInPart, F); 0] {
        // Yay we can return dangling references for singleton types!
        &[]
    }

    type BytesIter = core::str::Bytes<'a>;
    fn bytes(&'a self) -> Self::BytesIter {
        (self as &str).bytes()
    }

    fn len(&self) -> usize {
        (self as &str).len()
    }
}

/// A string with L levels of formatting annotations.
#[derive(Debug, PartialEq)]
pub struct LayeredFormattedString<F: Copy, const L: usize> {
    // bytes is always valid UTF-8, so from_utf8_unchecked is safe
    bytes: VecDeque<u8>,
    // The vector dimension corresponds to the bytes, the array dimension are the L levels of annotations
    annotations: VecDeque<[(LocationInPart, F); L]>,
}

pub type FormattedString<F> = LayeredFormattedString<F, 1>;

impl<'a, F: Copy, const L: usize> FormattedStringLike<'a, F, L> for LayeredFormattedString<F, L> {
    fn annotation_at(&self, pos: usize) -> &[(LocationInPart, F); L] {
        &self.annotations[pos]
    }

    type BytesIter =
        core::iter::Copied<core::iter::Chain<core::slice::Iter<'a, u8>, core::slice::Iter<'a, u8>>>;
    fn bytes(&'a self) -> Self::BytesIter {
        let (front, back) = self.bytes.as_slices();
        front.iter().chain(back.iter()).copied()
    }

    fn len(&self) -> usize {
        self.bytes.len()
    }
}

trait InsertIter<T> {
    fn insert_iter<I>(&mut self, pos: usize, values: I)
    where
        I: DoubleEndedIterator<Item = T>;
}

impl<T> InsertIter<T> for VecDeque<T> {
    fn insert_iter<I>(&mut self, pos: usize, values: I)
    where
        I: DoubleEndedIterator<Item = T>,
    {
        if pos == 0 {
            for value in values.rev() {
                self.push_front(value)
            }
        } else if pos == self.len() {
            for value in values {
                self.push_back(value)
            }
        } else {
            for (i, value) in values.enumerate() {
                self.insert(pos + i, value)
            }
        }
    }
}

impl<F: Copy, const L: usize> LayeredFormattedString<F, L> {
    pub fn new() -> Self {
        Self::with_capacity(40)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        // A LayeredFormattedString with 0 annotations doesn't make sense.
        assert!(L > 0);
        Self {
            bytes: VecDeque::with_capacity(capacity),
            annotations: VecDeque::with_capacity(capacity),
        }
    }

    pub fn capacity(&self) -> usize {
        assert_eq!(self.bytes.capacity(), self.annotations.capacity());
        self.bytes.capacity()
    }

    pub fn len(&self) -> usize {
        assert_eq!(self.bytes.len(), self.annotations.len());
        self.bytes.len()
    }

    pub fn append<'a, S, const L1: usize>(&mut self, string: &'a S, field: F) -> &mut Self
    where
        S: FormattedStringLike<'a, F, L1>,
    {
        assert_eq!(L - 1, L1);
        // len() is always a char boundary
        self.insert_internal(self.bytes.len(), string, field)
    }

    pub fn prepend<'a, S, const L1: usize>(&mut self, string: &'a S, field: F) -> &mut Self
    where
        S: FormattedStringLike<'a, F, L1>,
    {
        assert_eq!(L - 1, L1);
        // 0 is always a char boundary
        self.insert_internal(0, string, field)
    }

    pub fn insert<'a, S, const L1: usize>(
        &mut self,
        pos: usize,
        string: &'a S,
        field: F,
    ) -> Result<&mut Self, FormattedStringError>
    where
        S: FormattedStringLike<'a, F, L1>,
    {
        assert_eq!(L - 1, L1);
        if pos > self.bytes.len() {
            return Err(FormattedStringError::IndexOutOfBounds(pos));
        }
        // This is bit magic equivalent to: b >= 128 && b < 192, i.e. b is
        // not a UTF-8 character boundary. Lifted from str::is_char_boundary
        if (self.bytes[pos] as i8) < -0x40 {
            Err(FormattedStringError::PositionNotCharBoundary(
                pos,
                self.as_str().to_owned(),
            ))
        } else {
            Ok(self.insert_internal(pos, string, field))
        }
    }

    // Precondition here is that pos is a char boundary and < bytes.len().
    fn insert_internal<'a, S, const L1: usize>(
        &mut self,
        pos: usize,
        string: &'a S,
        field: F,
    ) -> &mut Self
    where
        S: FormattedStringLike<'a, F, L1>,
    {
        assert_eq!(L - 1, L1);
        self.bytes.insert_iter(pos, string.bytes());
        self.annotations.insert_iter(
            pos,
            (0..string.len()).map(|i| {
                let top_level = (
                    if i == 0 {
                        LocationInPart::Begin
                    } else {
                        LocationInPart::Extend
                    },
                    field,
                );
                let mut all_levels = [top_level; L];
                all_levels[1..L].copy_from_slice(string.annotation_at(i));
                all_levels
            }),
        );
        self
    }

    pub fn field_at(&self, pos: usize) -> F {
        self.fields_at(pos)[0]
    }

    pub fn as_str(&mut self) -> &str {
        unsafe { &str::from_utf8_unchecked(self.bytes.make_contiguous()) }
    }
}

impl<F: Copy, const L: usize> Default for LayeredFormattedString<F, L> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{fmt::Debug, panic};

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Field {
        Word,
        Space,
        Greeting,
    }

    #[test]
    fn test_basic() {
        let mut x = FormattedString::<Field>::new();
        x.append(&"world", Field::Word)
            .prepend(&" ", Field::Space)
            .prepend(&"hello", Field::Word);

        assert_eq!(x.as_str(), "hello world");

        for i in 0.."hello".len() {
            assert_eq!(x.field_at(i), Field::Word);
        }
        assert_eq!(x.field_at(5), Field::Space);
        for i in 0.."world".len() {
            assert_eq!(x.field_at(6 + i), Field::Word);
        }
        assert_panics(|| x.field_at(11));
    }

    #[test]
    fn test_multi_level() {
        let mut x = FormattedString::<Field>::new();
        x.append(&"world", Field::Word)
            .prepend(&" ", Field::Space)
            .prepend(&"hello", Field::Word);

        let mut y = LayeredFormattedString::<Field, 2>::new();
        y.append(&x, Field::Greeting);

        assert_eq!(y.as_str(), "hello world");
        assert_eq!(y.fields_at(0), [Field::Greeting, Field::Word]);
    }

    #[test]
    fn test_multi_byte() {
        let mut x = FormattedString::<Field>::new();
        x.append(&"π", Field::Word);
        assert_eq!(
            x.insert(1, &"pi/2", Field::Word).unwrap_err().to_string(),
            "index 1 is not a character boundary in \"π\"",
        );

        assert_eq!(x.as_str(), "π");
        assert_eq!(x.field_at(0), Field::Word);
        assert_eq!(x.field_at(1), Field::Word);
        assert_panics(|| x.field_at(2));
    }

    fn assert_panics<F: FnOnce() -> R + panic::UnwindSafe, R>(f: F) {
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        let result = panic::catch_unwind(f);
        panic::set_hook(prev_hook);
        assert!(result.is_err());
    }
}
