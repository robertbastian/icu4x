// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Traits for data providers that produce `Any` objects.

use crate::prelude::*;
use crate::response::DataPayloadInner;
use core::any::Any;
use yoke::trait_hack::YokeTraitHack;
use yoke::Yokeable;
use zerofrom::ZeroFrom;

/// A trait that allows to specify `Send + Sync` bounds that are only required when
/// the `sync` Cargo feature is enabled. Without the Cargo feature, this is an empty bound.
#[cfg(feature = "sync")]
pub trait MaybeSendSync: Send + Sync {}
#[cfg(feature = "sync")]
impl<T: Send + Sync> MaybeSendSync for T {}

#[allow(missing_docs)] // docs generated with all features
#[cfg(not(feature = "sync"))]
pub trait MaybeSendSync {}
#[cfg(not(feature = "sync"))]
impl<T> MaybeSendSync for T {}

/// Representations of the `Any` trait object.
///
/// **Important Note:** The types enclosed by `StructRef` and `PayloadRc` are NOT the same!
/// The first refers to the struct itself, whereas the second refers to a `DataPayload`.
#[derive(Debug)]
enum AnyYokeableInner {
    /// A reference to `M::Yokeable`
    StructRef(&'static dyn Any),
    /// A boxed `DataPayload<M>`.
    ///
    /// Note: This needs to be reference counted, not a `Box`, so that `AnyYokeable` is cloneable.
    /// If an `AnyYokeable` is cloned, the actual cloning of the data is delayed until
    /// `downcast()` is invoked (at which point we have the concrete type).
    PayloadRc(Box<dyn Any + Send + Sync>),
}

/// A type-erased data payload.
///
/// The only useful method on this type is [`AnyPayload::downcast()`], which transforms this into
/// a normal `DataPayload` which you can subsequently access or mutate.
///
/// As with `DataPayload`, cloning is designed to be cheap.
#[derive(Debug, Yokeable)]
pub struct AnyYokeable {
    inner: AnyYokeableInner,
    type_name: &'static str,
}

/// The [`DataMarker`] marker type for [`AnyPayload`].
#[allow(clippy::exhaustive_structs)] // marker type
#[derive(Debug)]
pub struct AnyMarker;

impl DataMarker for AnyMarker {
    type Yokeable = AnyYokeable;
}

impl<M> crate::dynutil::UpcastDataPayload<M> for AnyMarker
where
    M: DataMarker,
    M::Yokeable: MaybeSendSync,
{
    #[inline]
    fn upcast(other: DataPayload<M>) -> DataPayload<AnyMarker> {
        DataPayload::from_owned(AnyYokeable {
                inner: match other.0 {
                    DataPayloadInner::StaticRef(r) => AnyYokeableInner::StructRef(r),
                    inner => AnyYokeableInner::PayloadRc(Box::from(DataPayload(inner))),
                },
                type_name: core::any::type_name::<M>(),
            })
    }
}

impl AnyYokeable {
    /// Transforms a type-erased `AnyPayload` into a concrete `DataPayload<M>`.
    ///
    /// Because it is expected that the call site knows the identity of the AnyPayload (e.g., from
    /// the data request), this function returns a `DataError` if the generic type does not match
    /// the type stored in the `AnyPayload`.
    pub fn downcast<M>(self) -> Result<DataPayload<M>, DataError>
    where
        M: DataMarker,
        // For the StructRef case:
        M::Yokeable: ZeroFrom<'static, M::Yokeable>,
        // For the PayloadRc case:
        M::Yokeable: MaybeSendSync,
        for<'a> YokeTraitHack<<M::Yokeable as Yokeable<'a>>::Output>: Clone,
    {
        use AnyYokeableInner::*;
        let type_name = self.type_name;
        match self.inner {
            StructRef(any_ref) => {
                let down_ref: &'static M::Yokeable = any_ref
                    .downcast_ref()
                    .ok_or_else(|| DataError::for_type::<M>().with_str_context(type_name))?;
                Ok(DataPayload::from_static_ref(down_ref))
            }
            PayloadRc(any_box) => {
                let down_box = any_box
                    .downcast::<DataPayload<M>>()
                    .map_err(|_| DataError::for_type::<M>().with_str_context(type_name))?;
                Ok(*down_box)
            }
        }
    }
}

impl DataPayload<AnyMarker> {
    /// Transforms a type-erased `DataPayload<AnyMarker>` into a concrete `DataPayload<M>`.
    #[inline]
    pub fn downcast<M>(self) -> Result<DataPayload<M>, DataError>
    where
        M: DataMarker,
        for<'a> YokeTraitHack<<M::Yokeable as Yokeable<'a>>::Output>: Clone,
        M::Yokeable: ZeroFrom<'static, M::Yokeable>,
        M::Yokeable: MaybeSendSync,
    {
        self.try_unwrap_owned()?.downcast()
    }
}

impl DataResponse<AnyMarker> {
    /// Transforms a type-erased `DataResponse<AnyMarker>` into a concrete `DataResponse<M>`.
    #[inline]
    pub fn downcast<M>(self) -> Result<DataResponse<M>, DataError>
    where
        M: DataMarker,
        for<'a> YokeTraitHack<<M::Yokeable as Yokeable<'a>>::Output>: Clone,
        M::Yokeable: ZeroFrom<'static, M::Yokeable>,
        M::Yokeable: MaybeSendSync,
    {
        Ok(DataResponse {
            metadata: self.metadata,
            payload: self.payload.map(|p| p.downcast()).transpose()?,
        })
    }
}

/// An object-safe data provider that returns data structs cast to `dyn Any` trait objects.
///
/// # Examples
///
/// ```
/// use icu_provider::hello_world::*;
/// use icu_provider::prelude::*;
/// use std::borrow::Cow;
///
/// let any_provider = HelloWorldProvider;
///
/// let req = DataRequest {
///     locale: &icu_locale_core::langid!("de").into(),
///     ..Default::default()
/// };
///
/// // Downcasting manually
/// assert_eq!(
///     DynamicDataProvider::<AnyMarker>::load_data(
///         &any_provider,
///         HelloWorldV1Marker::KEY, 
///         req)
///         .expect("load should succeed")
///         .downcast::<HelloWorldV1Marker>()
///         .expect("types should match")
///         .take_payload()
///         .unwrap()
///         .get(),
///     &HelloWorldV1 {
///         message: Cow::Borrowed("Hallo Welt"),
///     },
/// );
///
/// // Downcasting automatically
/// let downcasting_provider: &dyn DataProvider<HelloWorldV1Marker> =
///     &any_provider.as_downcasting();
///
/// assert_eq!(
///     downcasting_provider
///         .load(req)
///         .expect("load should succeed")
///         .take_payload()
///         .unwrap()
///         .get(),
///     &HelloWorldV1 {
///         message: Cow::Borrowed("Hallo Welt"),
///     },
/// );
/// ```
pub trait AnyProvider: DynamicDataProvider<AnyMarker> {}

impl<P: DynamicDataProvider<AnyMarker> + ?Sized> AnyProvider for P {}

/// A wrapper over `AnyProvider` that implements `DynamicDataProvider<M>` via downcasting
#[allow(clippy::exhaustive_structs)] // newtype
#[derive(Debug)]
pub struct DowncastingAnyProvider<'a, P: ?Sized>(pub &'a P);

/// Blanket-implemented trait adding the [`Self::as_downcasting()`] function.
pub trait AsDowncastingAnyProvider {
    /// Returns an object implementing `DynamicDataProvider<M>` when called on `AnyProvider`
    fn as_downcasting(&self) -> DowncastingAnyProvider<Self>;
}

impl<P> AsDowncastingAnyProvider for P
where
    P: AnyProvider + ?Sized,
{
    #[inline]
    fn as_downcasting(&self) -> DowncastingAnyProvider<P> {
        DowncastingAnyProvider(self)
    }
}

impl<M, P> DataProvider<M> for DowncastingAnyProvider<'_, P>
where
    P: AnyProvider + ?Sized,
    M: KeyedDataMarker,
    for<'a> YokeTraitHack<<M::Yokeable as Yokeable<'a>>::Output>: Clone,
    M::Yokeable: ZeroFrom<'static, M::Yokeable>,
    M::Yokeable: MaybeSendSync,
{
    #[inline]
    fn load(&self, req: DataRequest) -> Result<DataResponse<M>, DataError> {
        self.0
            .load_data(M::KEY, req)?
            .downcast()
            .map_err(|e| e.with_req(M::KEY, req))
    }
}

impl<M, P> DynamicDataProvider<M> for DowncastingAnyProvider<'_, P>
where
    P: AnyProvider + ?Sized,
    M: DataMarker,
    for<'a> YokeTraitHack<<M::Yokeable as Yokeable<'a>>::Output>: Clone,
    M::Yokeable: ZeroFrom<'static, M::Yokeable>,
    M::Yokeable: MaybeSendSync,
{
    #[inline]
    fn load_data(&self, key: DataKey, req: DataRequest) -> Result<DataResponse<M>, DataError> {
        self.0
            .load_data(key, req)?
            .downcast()
            .map_err(|e| e.with_req(key, req))
    }
}

#[cfg(test)]
mod test {
    // use super::*;
    // use crate::hello_world::*;
    // use alloc::borrow::Cow;

    // const CONST_DATA: HelloWorldV1<'static> = HelloWorldV1 {
    //     message: Cow::Borrowed("Custom Hello World"),
    // };

    // #[test]
    // fn test_debug() {
    //     let payload: DataPayload<HelloWorldV1Marker> = DataPayload::from_owned(HelloWorldV1 {
    //         message: Cow::Borrowed("Custom Hello World"),
    //     });

    //     let any_payload = AnyYokeable {
    //             inner: match payload.0 {
    //                 DataPayloadInner::StaticRef(r) => AnyYokeableInner::StructRef(r),
    //                 inner => AnyYokeableInner::PayloadRc(SelectedRc::from(DataPayload::<M>(inner))),
    //             },
    //             type_name: core::any::type_name::<M>(),
    //         };
    //     assert_eq!(
    //         "AnyPayload { inner: PayloadRc(Any { .. }), type_name: \"icu_provider::hello_world::HelloWorldV1Marker\" }",
    //         format!("{any_payload:?}")
    //     );

    //     struct WrongMarker;

    //     impl DataMarker for WrongMarker {
    //         type Yokeable = u8;
    //     }

    //     let err = any_payload.downcast::<WrongMarker>().unwrap_err();
    //     assert_eq!(
    //         "ICU4X data error: Mismatched types: tried to downcast with icu_provider::any::test::test_debug::WrongMarker, but actual type is different: icu_provider::hello_world::HelloWorldV1Marker",
    //         format!("{err}")
    //     );
    // }

    // #[test]
    // fn test_non_owned_any_marker() {
    //     // This test demonstrates a code path that can trigger the InvalidState error kind.
    //     let payload_result: DataPayload<AnyMarker> =
    //         DataPayload::from_owned_buffer(Box::new(*b"pretend we're borrowing from here"))
    //             .map_project(|_, _| AnyYokeable::from_static_ref(&CONST_DATA));
    //     let err = payload_result.downcast::<HelloWorldV1Marker>().unwrap_err();
    //     assert!(matches!(
    //         err,
    //         DataError {
    //             kind: DataErrorKind::InvalidState,
    //             ..
    //         }
    //     ));
    // }
}
