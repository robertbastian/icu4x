// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! [`VarULE`] impls for tuples.
//!
//! This module exports [`Tuple2VarULE`], [`Tuple3VarULE`], ..., the corresponding [`VarULE`] types
//! of tuples containing purely [`VarULE`] types.
//!
//! This can be paired with [`VarTupleULE`] to make arbitrary combinations of [`ULE`] and [`VarULE`] types.
//!
//! [`VarTupleULE`]: crate::ule::vartuple::VarTupleULE

use super::*;
use alloc::borrow::ToOwned;
use core::fmt;
use core::marker::PhantomData;
use core::mem;
use zerofrom::ZeroFrom;

macro_rules! tuple_varule {
    // Invocation: Should be called like `tuple_ule!(Tuple2VarULE, 2, [ A a AX 0, B b BX 1 ])`
    //
    // $T is a generic name, $t is a lowercase version of it, $T_alt is an "alternate" name to use when we need two types referring
    // to the same input field, $i is an index.
    //
    // $name is the name of the type, $len MUST be the total number of fields, and then $i must be an integer going from 0 to (n - 1) in sequence
    // (This macro code can rely on $i < $len)
    ($name:ident, $len:literal, [ $($T:ident $t:ident $T_alt: ident $i:tt),+ ]) => {
        #[doc = concat!("VarULE type for tuples with ", $len, " elements. See module docs for more information")]
        #[repr(transparent)]
        #[allow(clippy::exhaustive_structs)] // stable
        pub struct $name<$($T: ?Sized),+> {
            $($t: PhantomData<$T>,)+
            // Safety invariant: Each "field" $i of the MultiFieldsULE is a valid instance of $t
            //
            // In other words, calling `.get_field::<$T>($i)` is always safe.
            //
            // This invariant is upheld when this type is constructed during VarULE parsing/validation
            multi: MultiFieldsULE<$len>
        }

        impl<$($T: VarULE + ?Sized),+> $name<$($T),+> {
            $(
                #[doc = concat!("Get field ", $i, "of this tuple")]
                pub fn $t(&self) -> &$T {
                     // Safety: See invariant of `multi`.
                    unsafe {
                        self.multi.get_field::<$T>($i)
                    }
                }


            )+
        }

        // # Safety
        //
        // ## Checklist
        //
        // Safety checklist for `VarULE`:
        //
        // 1. align(1): repr(transparent) around an align(1) VarULE type: MultiFieldsULE
        // 2. No padding: see previous point
        // 3. `validate_byte_slice` validates that this type is a valid MultiFieldsULE, and that each field is the correct type from the tuple.
        // 4. `validate_byte_slice` checks length by deferring to the inner ULEs
        // 5. `from_byte_slice_unchecked` returns a fat pointer to the bytes.
        // 6. All other methods are left at their default impl.
        // 7. The inner ULEs have byte equality, so this composition has byte equality.
        unsafe impl<$($T: VarULE + ?Sized),+> VarULE for $name<$($T),+>
        {
            fn validate_byte_slice(bytes: &[u8]) -> Result<(), UleError> {
                let multi = <MultiFieldsULE<$len> as VarULE>::parse_byte_slice(bytes)?;
                $(
                    // Safety invariant: $i < $len, from the macro invocation
                    unsafe {
                        multi.validate_field::<$T>($i)?;
                    }
                )+
                Ok(())
            }

            unsafe fn from_byte_slice_unchecked(bytes: &[u8]) -> &Self {
                let multi = <MultiFieldsULE<$len> as VarULE>::from_byte_slice_unchecked(bytes);

                // This type is repr(transparent) over MultiFieldsULE<$len>, so its slices can be transmuted
                // Field invariant upheld here: validate_byte_slice above validates every field for being the right type
                mem::transmute::<&MultiFieldsULE<$len>, &Self>(multi)
            }
        }

        impl<$($T: fmt::Debug + VarULE + ?Sized),+> fmt::Debug for $name<$($T),+> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                ($(self.$t(),)+).fmt(f)
            }
        }

        // We need manual impls since `#[derive()]` is disallowed on packed types
        impl<$($T: PartialEq + VarULE + ?Sized),+> PartialEq for $name<$($T),+> {
            fn eq(&self, other: &Self) -> bool {

                ($(self.$t(),)+).eq(&($(other.$t(),)+))
            }
        }

        impl<$($T: Eq + VarULE + ?Sized),+> Eq for $name<$($T),+> {}

        impl<$($T: PartialOrd + VarULE + ?Sized),+> PartialOrd for $name<$($T),+> {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                ($(self.$t(),)+).partial_cmp(&($(other.$t(),)+))
            }
        }

        impl<$($T: Ord + VarULE + ?Sized),+> Ord for $name<$($T),+> {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                ($(self.$t(),)+).cmp(&($(other.$t(),)+))
            }
        }

        // # Safety
        //
        // encode_var_ule_len: returns the length of the individual VarULEs together.
        //
        // encode_var_ule_write: writes bytes by deferring to the inner VarULE impls.
        unsafe impl<$($T,)+ $($T_alt),+> EncodeAsVarULE<$name<$($T),+>> for ( $($T_alt),+ )
        where
            $($T: VarULE + ?Sized,)+
            $($T_alt: EncodeAsVarULE<$T>,)+
        {
            fn encode_var_ule_as_slices<R>(&self, _: impl FnOnce(&[&[u8]]) -> R) -> R {
                // unnecessary if the other two are implemented
                unreachable!()
            }

            #[inline]
            fn encode_var_ule_len(&self) -> usize {
                MultiFieldsULE::<$len>::compute_encoded_len_for([$(self.$i.encode_var_ule_len()),+])
            }

            #[inline]
            fn encode_var_ule_write(&self, dst: &mut [u8]) {
                let lengths = [$(self.$i.encode_var_ule_len()),+];
                let multi = MultiFieldsULE::<$len>::new_from_lengths_partially_initialized(lengths, dst);
                $(
                    // Safety: $i < $len, from the macro invocation, and field $i is supposed to be of type $T
                    unsafe {
                        multi.set_field_at::<$T, $T_alt>($i, &self.$i);
                    }
                )+
            }
        }

        impl<$($T: VarULE + ?Sized),+> ToOwned for $name<$($T),+> {
            type Owned = Box<Self>;
            fn to_owned(&self) -> Self::Owned {
                encode_varule_to_box(self)
            }
        }

        impl<'a, $($T,)+ $($T_alt),+> ZeroFrom <'a, $name<$($T,)+>> for ($($T_alt),+)
        where
                    $($T: VarULE + ?Sized,)+
                    $($T_alt: ZeroFrom<'a, $T>,)+ {
            fn zero_from(other: &'a $name<$($T,)+>) -> Self {
                (
                    $($T_alt::zero_from(other.$t()),)+
                )
            }
        }
    };
}

tuple_varule!(Tuple2VarULE, 2, [ A a AE 0, B b BE 1 ]);
tuple_varule!(Tuple3VarULE, 3, [ A a AE 0, B b BE 1, C c CE 2 ]);
tuple_varule!(Tuple4VarULE, 4, [ A a AE 0, B b BE 1, C c CE 2, D d DE 3 ]);
tuple_varule!(Tuple5VarULE, 5, [ A a AE 0, B b BE 1, C c CE 2, D d DE 3, E e EE 4 ]);
tuple_varule!(Tuple6VarULE, 6, [ A a AE 0, B b BE 1, C c CE 2, D d DE 3, E e EE 4, F f FE 5 ]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VarZeroSlice;
    use crate::VarZeroVec;
    #[test]
    fn test_pairvarule_validate() {
        let vec: Vec<(&str, &[u8])> = vec![("a", b"b"), ("foo", b"bar"), ("lorem", b"ipsum\xFF")];
        let zerovec: VarZeroVec<Tuple2VarULE<str, [u8]>> = (&vec).into();
        let bytes = zerovec.as_bytes();
        let zerovec2 = VarZeroVec::parse_byte_slice(bytes).unwrap();
        assert_eq!(zerovec, zerovec2);

        // Test failed validation with a correctly sized but differently constrained tuple
        // Note: ipsum\xFF is not a valid str
        let zerovec3 = VarZeroVec::<Tuple2VarULE<str, str>>::parse_byte_slice(bytes);
        assert!(zerovec3.is_err());
    }
    #[test]
    fn test_tripleule_validate() {
        let vec: Vec<(&str, &[u8], VarZeroVec<str>)> = vec![
            ("a", b"b", (&vec!["a", "b", "c"]).into()),
            ("foo", b"bar", (&vec!["baz", "quux"]).into()),
            (
                "lorem",
                b"ipsum\xFF",
                (&vec!["dolor", "sit", "amet"]).into(),
            ),
        ];
        let zerovec: VarZeroVec<Tuple3VarULE<str, [u8], VarZeroSlice<str>>> = (&vec).into();
        let bytes = zerovec.as_bytes();
        let zerovec2 = VarZeroVec::parse_byte_slice(bytes).unwrap();
        assert_eq!(zerovec, zerovec2);

        // Test failed validation with a correctly sized but differently constrained tuple
        // Note: the str is unlikely to be a valid varzerovec
        let zerovec3 = VarZeroVec::<Tuple3VarULE<VarZeroSlice<str>, [u8], VarZeroSlice<str>>>::parse_byte_slice(bytes);
        assert!(zerovec3.is_err());
    }
}
