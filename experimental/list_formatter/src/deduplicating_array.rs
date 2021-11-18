use core::{mem, ops::Index, ptr};
use icu_provider::yoke::*;

#[derive(Debug)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Deserialize, serde::Serialize)
)]
// TODO: Make this generic over the 12. Serde currently can't derive
// if const generics are present: https://github.com/serde-rs/serde/issues/1937
pub struct DeduplicatingArray<T>([Dedupe<T>; 12]);

#[derive(Debug, PartialEq)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Deserialize, serde::Serialize)
)]
#[serde(untagged)]
enum Dedupe<T> {
    Value(T),
    Fallback(usize),
}

impl<T: PartialEq> DeduplicatingArray<T> {
    pub fn new(raw: [T; 12]) -> Self {
        let mut deduped = raw.map(Dedupe::Value);
        for i in 0..12 {
            if let Dedupe::Value(_) = &deduped[i] {
                // This is the first time we're seeing this value, so we don't
                // have to look below i
                for j in (i + 1)..12 {
                    if deduped[j] == deduped[i] {
                        deduped[j] = Dedupe::Fallback(i);
                    }
                }
            }
        }
        Self(deduped)
    }
}

impl<T> Index<usize> for DeduplicatingArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        match &self.0[index] {
            Dedupe::Value(t) => t,
            Dedupe::Fallback(fallback_index) => match &self.0[*fallback_index] {
                Dedupe::Value(t) => t,
                _ => panic!("Bad fallback"),
            },
        }
    }
}

impl<C, T: ZeroCopyFrom<C>> ZeroCopyFrom<DeduplicatingArray<C>> for DeduplicatingArray<T>
where
    DeduplicatingArray<C>: Index<usize, Output = C>,
{
    fn zero_copy_from(
        this: &DeduplicatingArray<C>,
    ) -> DeduplicatingArray<<T as Yokeable<'_>>::Output> {
        DeduplicatingArray([
            // While we're zcf'ing we might as well resolve fallbacks to save
            // an indirection later.
            Dedupe::Value(<T as ZeroCopyFrom<C>>::zero_copy_from(&this[0])),
            Dedupe::Value(<T as ZeroCopyFrom<C>>::zero_copy_from(&this[1])),
            Dedupe::Value(<T as ZeroCopyFrom<C>>::zero_copy_from(&this[2])),
            Dedupe::Value(<T as ZeroCopyFrom<C>>::zero_copy_from(&this[3])),
            Dedupe::Value(<T as ZeroCopyFrom<C>>::zero_copy_from(&this[4])),
            Dedupe::Value(<T as ZeroCopyFrom<C>>::zero_copy_from(&this[5])),
            Dedupe::Value(<T as ZeroCopyFrom<C>>::zero_copy_from(&this[6])),
            Dedupe::Value(<T as ZeroCopyFrom<C>>::zero_copy_from(&this[7])),
            Dedupe::Value(<T as ZeroCopyFrom<C>>::zero_copy_from(&this[8])),
            Dedupe::Value(<T as ZeroCopyFrom<C>>::zero_copy_from(&this[9])),
            Dedupe::Value(<T as ZeroCopyFrom<C>>::zero_copy_from(&this[10])),
            Dedupe::Value(<T as ZeroCopyFrom<C>>::zero_copy_from(&this[11])),
        ])
    }
}

unsafe impl<'a, T: 'static + for<'b> Yokeable<'b>> Yokeable<'a> for DeduplicatingArray<T> {
    type Output = DeduplicatingArray<<T as Yokeable<'a>>::Output>;

    fn transform(&'a self) -> &'a Self::Output {
        unsafe { mem::transmute(self) }
    }

    fn transform_owned(self) -> Self::Output {
        debug_assert!(mem::size_of::<Self::Output>() == mem::size_of::<Self>());
        unsafe {
            let ptr: *const Self::Output = (&self as *const Self).cast();
            mem::forget(self);
            ptr::read(ptr)
        }
    }

    unsafe fn make(from: Self::Output) -> Self {
        debug_assert!(mem::size_of::<Self::Output>() == mem::size_of::<Self>());
        let ptr: *const Self = (&from as *const Self::Output).cast();
        mem::forget(from);
        ptr::read(ptr)
    }

    fn transform_mut<F>(&'a mut self, f: F)
    where
        F: 'static + for<'b> FnOnce(&'b mut Self::Output),
    {
        // Cast away the lifetime of Self
        unsafe { f(mem::transmute::<&'a mut Self, &'a mut Self::Output>(self)) }
    }
}
