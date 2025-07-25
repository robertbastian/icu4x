// generated by diplomat-tool
// dart format off

part of 'lib.g.dart';

/// See the [Rust documentation for `RegionDisplayNames`](https://docs.rs/icu/2.0.0/icu/experimental/displaynames/struct.RegionDisplayNames.html) for more information.
final class RegionDisplayNames implements ffi.Finalizable {
  final ffi.Pointer<ffi.Opaque> _ffi;

  // These are "used" in the sense that they keep dependencies alive
  // ignore: unused_field
  final core.List<Object> _selfEdge;

  // This takes in a list of lifetime edges (including for &self borrows)
  // corresponding to data this may borrow from. These should be flat arrays containing
  // references to objects, and this object will hold on to them to keep them alive and
  // maintain borrow validity.
  RegionDisplayNames._fromFfi(this._ffi, this._selfEdge) {
    if (_selfEdge.isEmpty) {
      _finalizer.attach(this, _ffi.cast());
    }
  }

  @_DiplomatFfiUse('icu4x_RegionDisplayNames_destroy_mv1')
  static final _finalizer = ffi.NativeFinalizer(ffi.Native.addressOf(_icu4x_RegionDisplayNames_destroy_mv1));

  /// Creates a new `RegionDisplayNames` from locale data and an options bag using compiled data.
  ///
  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/experimental/displaynames/struct.RegionDisplayNames.html#method.try_new) for more information.
  ///
  /// Throws [DataError] on failure.
  factory RegionDisplayNames(Locale locale, DisplayNamesOptions options) {
    final temp = _FinalizedArena();
    final result = _icu4x_RegionDisplayNames_create_v1_mv1(locale._ffi, options._toFfi(temp.arena));
    if (!result.isOk) {
      throw DataError.values[result.union.err];
    }
    return RegionDisplayNames._fromFfi(result.union.ok, []);
  }

  /// Creates a new `RegionDisplayNames` from locale data and an options bag using a particular data source.
  ///
  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/experimental/displaynames/struct.RegionDisplayNames.html#method.try_new) for more information.
  ///
  /// Throws [DataError] on failure.
  factory RegionDisplayNames.createWithProvider(DataProvider provider, Locale locale, DisplayNamesOptions options) {
    final temp = _FinalizedArena();
    final result = _icu4x_RegionDisplayNames_create_v1_with_provider_mv1(provider._ffi, locale._ffi, options._toFfi(temp.arena));
    if (!result.isOk) {
      throw DataError.values[result.union.err];
    }
    return RegionDisplayNames._fromFfi(result.union.ok, []);
  }

  /// Returns the locale specific display name of a region.
  /// Note that the function returns an empty string in case the display name for a given
  /// region code is not found.
  ///
  /// See the [Rust documentation for `of`](https://docs.rs/icu/2.0.0/icu/experimental/displaynames/struct.RegionDisplayNames.html#method.of) for more information.
  ///
  /// Throws [LocaleParseError] on failure.
  String of(String region) {
    final temp = _FinalizedArena();
    final write = _Write();
    final result = _icu4x_RegionDisplayNames_of_mv1(_ffi, region._utf8AllocIn(temp.arena), write._ffi);
    if (!result.isOk) {
      throw LocaleParseError.values[result.union.err];
    }
    return write.finalize();
  }

}

@_DiplomatFfiUse('icu4x_RegionDisplayNames_destroy_mv1')
@ffi.Native<ffi.Void Function(ffi.Pointer<ffi.Void>)>(isLeaf: true, symbol: 'icu4x_RegionDisplayNames_destroy_mv1')
// ignore: non_constant_identifier_names
external void _icu4x_RegionDisplayNames_destroy_mv1(ffi.Pointer<ffi.Void> self);

@_DiplomatFfiUse('icu4x_RegionDisplayNames_create_v1_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, _DisplayNamesOptionsFfi)>(isLeaf: true, symbol: 'icu4x_RegionDisplayNames_create_v1_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_RegionDisplayNames_create_v1_mv1(ffi.Pointer<ffi.Opaque> locale, _DisplayNamesOptionsFfi options);

@_DiplomatFfiUse('icu4x_RegionDisplayNames_create_v1_with_provider_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>, _DisplayNamesOptionsFfi)>(isLeaf: true, symbol: 'icu4x_RegionDisplayNames_create_v1_with_provider_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_RegionDisplayNames_create_v1_with_provider_mv1(ffi.Pointer<ffi.Opaque> provider, ffi.Pointer<ffi.Opaque> locale, _DisplayNamesOptionsFfi options);

@_DiplomatFfiUse('icu4x_RegionDisplayNames_of_mv1')
@ffi.Native<_ResultVoidInt32 Function(ffi.Pointer<ffi.Opaque>, _SliceUtf8, ffi.Pointer<ffi.Opaque>)>(isLeaf: true, symbol: 'icu4x_RegionDisplayNames_of_mv1')
// ignore: non_constant_identifier_names
external _ResultVoidInt32 _icu4x_RegionDisplayNames_of_mv1(ffi.Pointer<ffi.Opaque> self, _SliceUtf8 region, ffi.Pointer<ffi.Opaque> write);

// dart format on
