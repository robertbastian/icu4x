// generated by diplomat-tool
// dart format off

part of 'lib.g.dart';

/// See the [Rust documentation for `DateTimeFormatter`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html) for more information.
final class DateTimeFormatter implements ffi.Finalizable {
  final ffi.Pointer<ffi.Opaque> _ffi;

  // These are "used" in the sense that they keep dependencies alive
  // ignore: unused_field
  final core.List<Object> _selfEdge;

  // This takes in a list of lifetime edges (including for &self borrows)
  // corresponding to data this may borrow from. These should be flat arrays containing
  // references to objects, and this object will hold on to them to keep them alive and
  // maintain borrow validity.
  DateTimeFormatter._fromFfi(this._ffi, this._selfEdge) {
    if (_selfEdge.isEmpty) {
      _finalizer.attach(this, _ffi.cast());
    }
  }

  @_DiplomatFfiUse('icu4x_DateTimeFormatter_destroy_mv1')
  static final _finalizer = ffi.NativeFinalizer(ffi.Native.addressOf(_icu4x_DateTimeFormatter_destroy_mv1));

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `DT`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DT.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DT.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DT.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DT.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.dt(Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment}) {
    final result = _icu4x_DateTimeFormatter_create_dt_mv1(locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `DT`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DT.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DT.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DT.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DT.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.dtWithProvider(DataProvider provider, Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment}) {
    final result = _icu4x_DateTimeFormatter_create_dt_with_provider_mv1(provider._ffi, locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `MDT`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDT.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDT.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDT.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDT.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.mdt(Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment}) {
    final result = _icu4x_DateTimeFormatter_create_mdt_mv1(locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `MDT`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDT.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDT.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDT.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDT.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.mdtWithProvider(DataProvider provider, Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment}) {
    final result = _icu4x_DateTimeFormatter_create_mdt_with_provider_mv1(provider._ffi, locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `YMDT`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDT.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDT.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDT.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDT.html#method.with_year_style), [4](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDT.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.ymdt(Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment, YearStyle? yearStyle}) {
    final result = _icu4x_DateTimeFormatter_create_ymdt_mv1(locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err(), yearStyle != null ? _ResultInt32Void.ok(yearStyle.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `YMDT`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDT.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDT.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDT.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDT.html#method.with_year_style), [4](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDT.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.ymdtWithProvider(DataProvider provider, Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment, YearStyle? yearStyle}) {
    final result = _icu4x_DateTimeFormatter_create_ymdt_with_provider_mv1(provider._ffi, locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err(), yearStyle != null ? _ResultInt32Void.ok(yearStyle.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `DET`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DET.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DET.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DET.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DET.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.det(Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment}) {
    final result = _icu4x_DateTimeFormatter_create_det_mv1(locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `DET`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DET.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DET.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DET.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.DET.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.detWithProvider(DataProvider provider, Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment}) {
    final result = _icu4x_DateTimeFormatter_create_det_with_provider_mv1(provider._ffi, locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `MDET`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDET.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDET.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDET.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDET.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.mdet(Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment}) {
    final result = _icu4x_DateTimeFormatter_create_mdet_mv1(locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `MDET`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDET.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDET.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDET.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.MDET.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.mdetWithProvider(DataProvider provider, Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment}) {
    final result = _icu4x_DateTimeFormatter_create_mdet_with_provider_mv1(provider._ffi, locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `YMDET`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDET.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDET.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDET.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDET.html#method.with_year_style), [4](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDET.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.ymdet(Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment, YearStyle? yearStyle}) {
    final result = _icu4x_DateTimeFormatter_create_ymdet_mv1(locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err(), yearStyle != null ? _ResultInt32Void.ok(yearStyle.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `YMDET`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDET.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDET.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDET.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDET.html#method.with_year_style), [4](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.YMDET.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.ymdetWithProvider(DataProvider provider, Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment, YearStyle? yearStyle}) {
    final result = _icu4x_DateTimeFormatter_create_ymdet_with_provider_mv1(provider._ffi, locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err(), yearStyle != null ? _ResultInt32Void.ok(yearStyle.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `ET`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.ET.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.ET.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.ET.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.ET.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.et(Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment}) {
    final result = _icu4x_DateTimeFormatter_create_et_mv1(locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `try_new`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.try_new) for more information.
  ///
  /// See the [Rust documentation for `ET`](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.ET.html) for more information.
  ///
  /// Additional information: [1](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.ET.html#method.with_time_precision), [2](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.ET.html#method.with_alignment), [3](https://docs.rs/icu/2.0.0/icu/datetime/fieldsets/struct.ET.html#method.for_length)
  ///
  /// Throws [DateTimeFormatterLoadError] on failure.
  factory DateTimeFormatter.etWithProvider(DataProvider provider, Locale locale, {DateTimeLength? length, TimePrecision? timePrecision, DateTimeAlignment? alignment}) {
    final result = _icu4x_DateTimeFormatter_create_et_with_provider_mv1(provider._ffi, locale._ffi, length != null ? _ResultInt32Void.ok(length.index) : _ResultInt32Void.err(), timePrecision != null ? _ResultInt32Void.ok(timePrecision.index) : _ResultInt32Void.err(), alignment != null ? _ResultInt32Void.ok(alignment.index) : _ResultInt32Void.err());
    if (!result.isOk) {
      throw DateTimeFormatterLoadError.values.firstWhere((v) => v._ffi == result.union.err);
    }
    return DateTimeFormatter._fromFfi(result.union.ok, []);
  }

  /// See the [Rust documentation for `format`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.format) for more information.
  String formatIso(IsoDate isoDate, Time time) {
    final write = _Write();
    _icu4x_DateTimeFormatter_format_iso_mv1(_ffi, isoDate._ffi, time._ffi, write._ffi);
    return write.finalize();
  }

  /// See the [Rust documentation for `format_same_calendar`](https://docs.rs/icu/2.0.0/icu/datetime/struct.DateTimeFormatter.html#method.format_same_calendar) for more information.
  ///
  /// Throws [DateTimeMismatchedCalendarError] on failure.
  String formatSameCalendar(Date date, Time time) {
    final write = _Write();
    final result = _icu4x_DateTimeFormatter_format_same_calendar_mv1(_ffi, date._ffi, time._ffi, write._ffi);
    if (!result.isOk) {
      throw DateTimeMismatchedCalendarError._fromFfi(result.union.err);
    }
    return write.finalize();
  }

}

@_DiplomatFfiUse('icu4x_DateTimeFormatter_destroy_mv1')
@ffi.Native<ffi.Void Function(ffi.Pointer<ffi.Void>)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_destroy_mv1')
// ignore: non_constant_identifier_names
external void _icu4x_DateTimeFormatter_destroy_mv1(ffi.Pointer<ffi.Void> self);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_dt_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_dt_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_dt_mv1(ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_dt_with_provider_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_dt_with_provider_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_dt_with_provider_mv1(ffi.Pointer<ffi.Opaque> provider, ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_mdt_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_mdt_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_mdt_mv1(ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_mdt_with_provider_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_mdt_with_provider_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_mdt_with_provider_mv1(ffi.Pointer<ffi.Opaque> provider, ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_ymdt_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_ymdt_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_ymdt_mv1(ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment, _ResultInt32Void yearStyle);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_ymdt_with_provider_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_ymdt_with_provider_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_ymdt_with_provider_mv1(ffi.Pointer<ffi.Opaque> provider, ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment, _ResultInt32Void yearStyle);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_det_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_det_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_det_mv1(ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_det_with_provider_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_det_with_provider_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_det_with_provider_mv1(ffi.Pointer<ffi.Opaque> provider, ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_mdet_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_mdet_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_mdet_mv1(ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_mdet_with_provider_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_mdet_with_provider_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_mdet_with_provider_mv1(ffi.Pointer<ffi.Opaque> provider, ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_ymdet_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_ymdet_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_ymdet_mv1(ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment, _ResultInt32Void yearStyle);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_ymdet_with_provider_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_ymdet_with_provider_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_ymdet_with_provider_mv1(ffi.Pointer<ffi.Opaque> provider, ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment, _ResultInt32Void yearStyle);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_et_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_et_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_et_mv1(ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_create_et_with_provider_mv1')
@ffi.Native<_ResultOpaqueInt32 Function(ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>, _ResultInt32Void, _ResultInt32Void, _ResultInt32Void)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_create_et_with_provider_mv1')
// ignore: non_constant_identifier_names
external _ResultOpaqueInt32 _icu4x_DateTimeFormatter_create_et_with_provider_mv1(ffi.Pointer<ffi.Opaque> provider, ffi.Pointer<ffi.Opaque> locale, _ResultInt32Void length, _ResultInt32Void timePrecision, _ResultInt32Void alignment);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_format_iso_mv1')
@ffi.Native<ffi.Void Function(ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_format_iso_mv1')
// ignore: non_constant_identifier_names
external void _icu4x_DateTimeFormatter_format_iso_mv1(ffi.Pointer<ffi.Opaque> self, ffi.Pointer<ffi.Opaque> isoDate, ffi.Pointer<ffi.Opaque> time, ffi.Pointer<ffi.Opaque> write);

@_DiplomatFfiUse('icu4x_DateTimeFormatter_format_same_calendar_mv1')
@ffi.Native<_ResultVoidDateTimeMismatchedCalendarErrorFfi Function(ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>, ffi.Pointer<ffi.Opaque>)>(isLeaf: true, symbol: 'icu4x_DateTimeFormatter_format_same_calendar_mv1')
// ignore: non_constant_identifier_names
external _ResultVoidDateTimeMismatchedCalendarErrorFfi _icu4x_DateTimeFormatter_format_same_calendar_mv1(ffi.Pointer<ffi.Opaque> self, ffi.Pointer<ffi.Opaque> date, ffi.Pointer<ffi.Opaque> time, ffi.Pointer<ffi.Opaque> write);

// dart format on
