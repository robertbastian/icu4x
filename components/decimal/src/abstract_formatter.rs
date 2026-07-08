// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use fixed_decimal::UnsignedDecimal;
use icu_plurals::PluralOperands;
use writeable::Writeable;

use crate::{
    CompactDecimalFormatter, DecimalFormatter, FormattedSign, FormattedUnsignedCompactDecimal,
    FormattedUnsignedDecimal,
};

pub trait Sealed {}

/// A trait representing an abstract number formatter.
/// 
/// This is a building block for more complicated formatters, like currency or units.
pub trait AbstractFormatter: core::fmt::Debug + Sealed {
    #[doc(hidden)]
    type Formatted<'a>: Writeable where Self: 'a;

    #[doc(hidden)]
    fn format<'a>(&'a self, value: &'a UnsignedDecimal) -> Self::Formatted<'a>;

    #[doc(hidden)]
    fn format_sign<'a, W: Writeable>(
        &'a self,
        value: W,
        sign: fixed_decimal::Sign,
    ) -> FormattedSign<'a, W>;

    #[doc(hidden)]
    fn plural_operands(value: &Self::Formatted<'_>) -> PluralOperands;
}

impl Sealed for DecimalFormatter {}
impl AbstractFormatter for DecimalFormatter {
    type Formatted<'a> = FormattedUnsignedDecimal<'a>;

    fn format<'a>(&'a self, value: &'a UnsignedDecimal) -> Self::Formatted<'a> {
        self.format_unsigned(crate::Cow::Borrowed(value))
    }

    fn format_sign<'a, W: Writeable>(
        &'a self,
        value: W,
        sign: fixed_decimal::Sign,
    ) -> FormattedSign<'a, W> {
        self.format_sign(sign, value)
    }

    fn plural_operands(value: &Self::Formatted<'_>) -> PluralOperands {
        value.plural_operands()
    }
}

impl Sealed for CompactDecimalFormatter {}
impl AbstractFormatter for CompactDecimalFormatter {
    type Formatted<'a> = FormattedUnsignedCompactDecimal<'a>;

    fn format<'a>(&'a self, value: &'a UnsignedDecimal) -> Self::Formatted<'a> {
        self.format_unsigned(value)
    }

    fn format_sign<'a, W: Writeable>(
        &'a self,
        value: W,
        sign: fixed_decimal::Sign,
    ) -> FormattedSign<'a, W> {
        self.decimal_formatter.format_sign(sign, value)
    }

    fn plural_operands(value: &Self::Formatted<'_>) -> PluralOperands {
        value.plural_operands()
    }
}
