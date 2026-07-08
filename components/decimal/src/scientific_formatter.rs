// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use core::fmt::Display;

use crate::{FormattedDecimal, FormattedUnsignedDecimal};
use crate::input::Decimal;
use crate::options::DecimalFormatterOptions;
use crate::{
    DecimalFormatter, options::ScientificDecimalFormatterOptions,
    preferences::ScientificDecimalFormatterPreferences, provider::*,
};
#[cfg(feature = "alloc")]
use alloc::string::String;
use fixed_decimal::UnsignedDecimal;
use icu_pattern::DoublePlaceholderPattern;
use icu_plurals::PluralOperands;
use icu_provider::DataError;
use icu_provider::prelude::*;
use writeable::Writeable;

/// A formatter that renders locale-sensitive compact numbers.
///
/// <div class="stab unstable">
/// 🚧 This code is considered unstable; it may change at any time, in breaking or non-breaking ways,
/// including in SemVer minor releases. Do not use this type unless you are prepared for things to occasionally break.
///
/// Graduation tracking issue: [issue #7161](https://github.com/unicode-org/icu4x/issues/7161).
/// </div>
///
/// ✨ *Enabled with the `unstable` Cargo feature.*
///
/// # Examples
///
/// ```
/// use icu::decimal::ScientificDecimalFormatter;
/// use icu::locale::locale;
/// use writeable::assert_writeable_eq;
///
/// let short_french = ScientificDecimalFormatter::try_new_short(
///     locale!("fr").into(),
///     Default::default(),
/// )
/// .unwrap();
///
/// let [long_french, long_japanese, long_bangla] =
///     [locale!("fr"), locale!("ja"), locale!("bn")].map(|locale| {
///         ScientificDecimalFormatter::try_new_long(
///             locale.into(),
///             Default::default(),
///         )
///         .unwrap()
///     });
///
/// /// Supports short and long notations:
/// # // The following line contains U+00A0 NO-BREAK SPACE.
/// assert_writeable_eq!(short_french.format(&35_357_670i64.into()), "35 M");
/// assert_writeable_eq!(
///     long_french.format(&35_357_670i64.into()),
///     "35 millions"
/// );
/// /// The powers of ten used are locale-dependent:
/// assert_writeable_eq!(long_japanese.format(&3535_7670i64.into()), "3536万");
/// /// So are the digits:
/// assert_writeable_eq!(
///     long_bangla.format(&3_53_57_670i64.into()),
///     "৩.৫ কোটি"
/// );
///
/// /// The output does not always contain digits:
/// assert_writeable_eq!(long_french.format(&1000i64.into()), "mille");
/// ```
#[derive(Debug)]
pub struct ScientificDecimalFormatter {
    pub(crate) decimal_formatter: DecimalFormatter,
}

impl ScientificDecimalFormatter {
    /// Creates a new short [`ScientificDecimalFormatter`] from compiled data and an options bag.
    ///
    /// ✨ *Enabled with the `compiled_data` Cargo feature.*
    ///
    /// [📚 Help choosing a constructor](icu_provider::constructors)
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::decimal::ScientificDecimalFormatter;
    /// use icu::locale::locale;
    /// use writeable::assert_writeable_eq;
    ///
    /// let formatter = ScientificDecimalFormatter::try_new_short(
    ///     locale!("sv").into(),
    ///     Default::default(),
    /// )
    /// .unwrap();
    ///
    /// assert_writeable_eq!(formatter.format(&1234.into()), "1,2 tn");
    /// ```
    #[cfg(feature = "compiled_data")]
    pub fn try_new_short(
        prefs: ScientificDecimalFormatterPreferences,
        options: ScientificDecimalFormatterOptions,
    ) -> Result<Self, DataError> {
        let locale = DecimalCompactShortV1::make_locale(prefs.locale_preferences);
        Ok(Self {
            decimal_formatter: DecimalFormatter::try_new((&prefs).into(), options.into())?,
        })
    }

    icu_provider::gen_buffer_data_constructors!(
        (prefs: ScientificDecimalFormatterPreferences, options: ScientificDecimalFormatterOptions) -> error: DataError,
        functions: [
            try_new_short: skip,
            try_new_short_with_buffer_provider,
            try_new_short_unstable,
            Self,
        ]
    );

    #[doc = icu_provider::gen_buffer_unstable_docs!(UNSTABLE, Self::try_new_short)]
    pub fn try_new_short_unstable<D>(
        provider: &D,
        prefs: ScientificDecimalFormatterPreferences,
        options: ScientificDecimalFormatterOptions,
    ) -> Result<Self, DataError>
    where
        D: DataProvider<DecimalSymbolsV1> + DataProvider<DecimalDigitsV1> + ?Sized,
    {
        let locale = DecimalCompactShortV1::make_locale(prefs.locale_preferences);
        Ok(Self {
            decimal_formatter: DecimalFormatter::try_new_unstable(
                provider,
                (&prefs).into(),
                options.into(),
            )?,
        })
    }

    /// Creates a new long [`ScientificDecimalFormatter`] from compiled data and an options bag.
    ///
    /// ✨ *Enabled with the `compiled_data` Cargo feature.*
    ///
    /// [📚 Help choosing a constructor](icu_provider::constructors)
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::decimal::ScientificDecimalFormatter;
    /// use icu::locale::locale;
    /// use writeable::assert_writeable_eq;
    ///
    /// let formatter = ScientificDecimalFormatter::try_new_long(
    ///     locale!("sv").into(),
    ///     Default::default(),
    /// )
    /// .unwrap();
    ///
    /// assert_writeable_eq!(formatter.format(&1234.into()), "1,2 tusen");
    /// ```
    #[cfg(feature = "compiled_data")]
    pub fn try_new_long(
        prefs: ScientificDecimalFormatterPreferences,
        options: ScientificDecimalFormatterOptions,
    ) -> Result<Self, DataError> {
        let locale = DecimalCompactLongV1::make_locale(prefs.locale_preferences);
        Ok(Self {
            decimal_formatter: DecimalFormatter::try_new((&prefs).into(), options.into())?,
        })
    }

    icu_provider::gen_buffer_data_constructors!(
        (prefs: ScientificDecimalFormatterPreferences, options: ScientificDecimalFormatterOptions) -> error: DataError,
        functions: [
            try_new_long: skip,
            try_new_long_with_buffer_provider,
            try_new_long_unstable,
            Self,
        ]
    );

    #[doc = icu_provider::gen_buffer_unstable_docs!(UNSTABLE, Self::try_new_long)]
    pub fn try_new_long_unstable<D>(
        provider: &D,
        prefs: ScientificDecimalFormatterPreferences,
        options: ScientificDecimalFormatterOptions,
    ) -> Result<Self, DataError>
    where
        D: DataProvider<DecimalCompactLongV1>
            + DataProvider<DecimalSymbolsV1>
            + DataProvider<DecimalDigitsV1>
            + ?Sized,
    {
        let locale = DecimalCompactLongV1::make_locale(prefs.locale_preferences);
        Ok(Self {
            decimal_formatter: DecimalFormatter::try_new_unstable(
                provider,
                (&prefs).into(),
                options.into(),
            )?,
        })
    }

    /// Formats a [`Decimal`] by automatically scaling and rounding it.
    ///
    /// The result may have a fractional digit only if it is compact and its
    /// significand is less than 10. Trailing fractional 0s are omitted.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::decimal::input::{Decimal, SignDisplay};
    /// use icu::decimal::ScientificDecimalFormatter;
    /// use icu::locale::locale;
    /// use writeable::assert_writeable_eq;
    ///
    /// let short_english = ScientificDecimalFormatter::try_new_short(
    ///     locale!("en").into(),
    ///     Default::default(),
    /// )
    /// .unwrap();
    ///
    /// assert_writeable_eq!(short_english.format(&Decimal::from(0)), "0");
    /// assert_writeable_eq!(short_english.format(&Decimal::from(2)), "2");
    /// assert_writeable_eq!(short_english.format(&Decimal::from(843)), "843");
    /// assert_writeable_eq!(short_english.format(&Decimal::from(2207)), "2.2K");
    /// assert_writeable_eq!(short_english.format(&Decimal::from(15127)), "15K");
    /// assert_writeable_eq!(short_english.format(&Decimal::from(3010349)), "3M");
    /// assert_writeable_eq!(short_english.format(&Decimal::from(-13132)), "-13K");
    ///
    /// // The sign display on the Decimal is respected:
    /// assert_writeable_eq!(
    ///     short_english.format(
    ///         &Decimal::from(2500).with_sign_display(SignDisplay::ExceptZero)
    ///     ),
    ///     "+2.5K"
    /// );
    /// ```
    ///
    /// The result is the nearest such compact number, with halfway cases-
    /// rounded towards the number with an even least significant digit.
    ///
    /// ```
    /// # use icu::decimal::ScientificDecimalFormatter;
    /// # use icu::locale::locale;
    /// # use writeable::assert_writeable_eq;
    /// #
    /// # let short_english = ScientificDecimalFormatter::try_new_short(
    /// #    locale!("en").into(),
    /// #    Default::default(),
    /// # ).unwrap();
    /// assert_writeable_eq!(
    ///     short_english.format(&"999499.99".parse().unwrap()),
    ///     "999K"
    /// );
    /// assert_writeable_eq!(
    ///     short_english.format(&"999500.00".parse().unwrap()),
    ///     "1M"
    /// );
    /// assert_writeable_eq!(
    ///     short_english.format(&"1650".parse().unwrap()),
    ///     "1.6K"
    /// );
    /// assert_writeable_eq!(
    ///     short_english.format(&"1750".parse().unwrap()),
    ///     "1.8K"
    /// );
    /// assert_writeable_eq!(short_english.format(&"1950".parse().unwrap()), "2K");
    /// assert_writeable_eq!(
    ///     short_english.format(&"-1172700".parse().unwrap()),
    ///     "-1.2M"
    /// );
    /// assert_writeable_eq!(
    ///     short_english.format(&"0.2222".parse().unwrap()),
    ///     "0.22"
    /// );
    /// ```
    ///
    /// Floating point inputs should use [`FloatPrecision::RoundTrip`](fixed_decimal::FloatPrecision::RoundTrip).
    ///
    /// ```
    /// # use icu::decimal::input::{Decimal, FloatPrecision};
    /// # use icu::decimal::ScientificDecimalFormatter;
    /// # use icu::locale::locale;
    /// # use writeable::assert_writeable_eq;
    /// #
    /// # let short_english = ScientificDecimalFormatter::try_new_short(
    /// #    locale!("en").into(),
    /// #    Default::default(),
    /// # ).unwrap();
    /// assert_writeable_eq!(
    ///     short_english.format(
    ///         &Decimal::try_from_f64(999_499.99, FloatPrecision::RoundTrip)
    ///             .unwrap()
    ///     ),
    ///     "999K"
    /// );
    /// ```
    pub fn format<'a>(&'a self, value: &Decimal) -> impl Writeable + Display + 'a {
        self.decimal_formatter
            .format_sign(value.sign, self.format_unsigned(&value.absolute))
    }

    pub(crate) fn format_unsigned<'a>(
        &'a self,
        value: &UnsignedDecimal,
    ) -> FormattedUnsignedScientificDecimal<'a> {
        const ROOT_PATTERN: &DoublePlaceholderPattern =
            unsafe { DoublePlaceholderPattern::from_ref_store_unchecked("\u{2}\u{5}E") };

        FormattedUnsignedScientificDecimal {
            value: value.clone().multiplied_pow10(0),
            exponent: Decimal::from(0),
            decimal_formatter: &self.decimal_formatter,
            pattern: ROOT_PATTERN,
        }
    }

    /// Formats a [`Decimal`], returning a [`String`].
    ///
    /// ✨ *Enabled with the `alloc` Cargo feature.*
    #[cfg(feature = "alloc")]
    pub fn format_to_string(&self, value: &Decimal) -> String {
        use writeable::Writeable;
        self.format(value).write_to_string().into_owned()
    }
}

#[doc(hidden)] // TODO(#3647): should be private
#[derive(Debug)]
pub struct FormattedUnsignedScientificDecimal<'l> {
    value: UnsignedDecimal,
    exponent: Decimal,
    decimal_formatter: &'l DecimalFormatter,
    pattern: &'l DoublePlaceholderPattern,
}

impl Writeable for FormattedUnsignedScientificDecimal<'_> {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        self.pattern
            .interpolate((
                self.decimal_formatter
                    .format_unsigned(crate::Cow::Borrowed(&self.value)),
                FormattedDecimal(self.decimal_formatter.format_sign(self.exponent.sign, {
                    FormattedUnsignedDecimal {
                        value: crate::Cow::Borrowed(&self.exponent.absolute),
                        options: &DecimalFormatterOptions {
                            grouping_strategy: Some(crate::options::GroupingStrategy::Never),
                        },
                        symbols: self.decimal_formatter.symbols.get(),
                        digits: self.decimal_formatter.digits.get(),
                    }
                })),
            ))
            .write_to(sink)
    }
}

impl FormattedUnsignedScientificDecimal<'_> {
    pub(crate) fn plural_operands(&self) -> PluralOperands {
        PluralOperands::from_significand_and_exponent(&self.value, 0)
    }
}
