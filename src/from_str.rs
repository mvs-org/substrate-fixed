// Copyright © 2018–2019 Trevor Spiteri

// This library is free software: you can redistribute it and/or
// modify it under the terms of either
//
//   * the Apache License, Version 2.0 or
//   * the MIT License
//
// at your option.
//
// You should have recieved copies of the Apache License and the MIT
// License along with the library. If not, see
// <https://www.apache.org/licenses/LICENSE-2.0> and
// <https://opensource.org/licenses/MIT>.

use crate::{
    frac::False,
    sealed::SealedInt,
    types::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8},
    wide_div::WideDivRem,
    FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32, FixedU64,
    FixedU8,
};
use core::{
    cmp::{self, Ordering},
    fmt::{Display, Formatter, Result as FmtResult},
    ops::{Add, Shl, Shr},
    str::FromStr,
};

fn bin_str_int_to_bin<I>(s: &str) -> Option<I>
where
    I: SealedInt<IsSigned = False> + From<u8>,
    I: Shl<u32, Output = I> + Shr<u32, Output = I> + Add<Output = I>,
{
    debug_assert!(!s.is_empty());
    let mut bytes = s.as_bytes().iter();
    let first_val = *bytes.next().unwrap() - b'0';
    let mut acc = I::from(first_val);
    let mut leading_zeros = acc.leading_zeros();
    for &byte in bytes {
        let val = byte - b'0';
        leading_zeros = leading_zeros.checked_sub(1)?;
        acc = (acc << 1) + I::from(val);
    }
    Some(acc)
}

fn bin_str_frac_to_bin<I>(s: &str, nbits: u32) -> Option<I>
where
    I: SealedInt<IsSigned = False> + From<u8>,
    I: Shl<u32, Output = I> + Shr<u32, Output = I> + Add<Output = I>,
{
    debug_assert!(!s.is_empty());
    let dump_bits = I::NBITS - nbits;
    let mut rem_bits = nbits;
    let mut acc = I::ZERO;
    for &byte in s.as_bytes() {
        let val = byte - b'0';
        if rem_bits < 1 {
            // round
            acc = acc.checked_add(I::from(val))?;
            if dump_bits != 0 && !(acc >> nbits).is_zero() {
                return None;
            }
            return Some(acc);
        }
        acc = (acc << 1) + I::from(val);
        rem_bits -= 1;
    }
    Some(acc << rem_bits)
}

fn oct_str_int_to_bin<I>(s: &str) -> Option<I>
where
    I: SealedInt<IsSigned = False> + From<u8>,
    I: Shl<u32, Output = I> + Shr<u32, Output = I> + Add<Output = I>,
{
    debug_assert!(!s.is_empty());
    let mut bytes = s.as_bytes().iter();
    let first_val = *bytes.next().unwrap() - b'0';
    let mut acc = I::from(first_val);
    let mut leading_zeros = acc.leading_zeros();
    for &byte in bytes {
        let val = byte - b'0';
        leading_zeros = leading_zeros.checked_sub(3)?;
        acc = (acc << 3) + I::from(val);
    }
    Some(acc)
}

fn oct_str_frac_to_bin<I>(s: &str, nbits: u32) -> Option<I>
where
    I: SealedInt<IsSigned = False> + From<u8>,
    I: Shl<u32, Output = I> + Shr<u32, Output = I> + Add<Output = I>,
{
    debug_assert!(!s.is_empty());
    let dump_bits = I::NBITS - nbits;
    let mut rem_bits = nbits;
    let mut acc = I::ZERO;
    for &byte in s.as_bytes() {
        let val = byte - b'0';
        if rem_bits < 3 {
            acc = (acc << rem_bits) + I::from(val >> (3 - rem_bits));
            // round
            acc = acc.checked_add(I::from((val >> (2 - rem_bits)) & 1))?;
            if dump_bits != 0 && !(acc >> nbits).is_zero() {
                return None;
            }
            return Some(acc);
        }
        acc = (acc << 3) + I::from(val);
        rem_bits -= 3;
    }
    Some(acc << rem_bits)
}

#[inline]
fn unchecked_hex_digit(byte: u8) -> u8 {
    // We know that byte is a valid hex:
    //   * b'0'..=b'9' (0x30..=0x39) => byte & 0x0f
    //   * b'A'..=b'F' (0x41..=0x46) => byte & 0x0f + 9
    //   * b'a'..=b'f' (0x61..=0x66) => byte & 0x0f + 9
    (byte & 0x0f) + if byte >= 0x40 { 9 } else { 0 }
}

fn hex_str_int_to_bin<I>(s: &str) -> Option<I>
where
    I: SealedInt<IsSigned = False> + From<u8>,
    I: Shl<u32, Output = I> + Add<Output = I>,
{
    debug_assert!(!s.is_empty());
    let mut bytes = s.as_bytes().iter();
    let first_val = unchecked_hex_digit(*bytes.next().unwrap());
    let mut acc = I::from(first_val);
    let mut leading_zeros = acc.leading_zeros();
    for &byte in bytes {
        let val = unchecked_hex_digit(byte);
        leading_zeros = leading_zeros.checked_sub(4)?;
        acc = (acc << 4) + I::from(val);
    }
    Some(acc)
}

fn hex_str_frac_to_bin<I>(s: &str, nbits: u32) -> Option<I>
where
    I: SealedInt<IsSigned = False> + From<u8>,
    I: Shl<u32, Output = I> + Shr<u32, Output = I> + Add<Output = I>,
{
    debug_assert!(!s.is_empty());
    let dump_bits = I::NBITS - nbits;
    let mut rem_bits = nbits;
    let mut acc = I::ZERO;
    for &byte in s.as_bytes() {
        let val = unchecked_hex_digit(byte);
        if rem_bits < 4 {
            acc = (acc << rem_bits) + I::from(val >> (4 - rem_bits));
            // round
            acc = acc.checked_add(I::from((val >> (3 - rem_bits)) & 1))?;
            if dump_bits != 0 && !(acc >> nbits).is_zero() {
                return None;
            }
            return Some(acc);
        }
        acc = (acc << 4) + I::from(val);
        rem_bits -= 4;
    }
    Some(acc << rem_bits)
}

// 5^3 × 2 < 2^8 => (10^3 - 1) × 2^(8-3+1) < 2^16
// Returns None for large fractions that are rounded to 1.0
fn dec3_to_bin8(val: u16, nbits: u32) -> Option<u8> {
    debug_assert!(val < 10u16.pow(3));
    let dump_bits = 8 - nbits;
    let divisor = 5u16.pow(3) * 2;
    let shift = val << (8 - 3 + 1) >> dump_bits;
    let round = shift + (divisor / 2);
    if round >> nbits >= divisor {
        None
    } else {
        Some((round / divisor) as u8)
    }
}
// 5^6 × 2 < 2^16 => (10^6 - 1) × 2^(16-6+1) < 2^32
// Returns None for large fractions that are rounded to 1.0
fn dec6_to_bin16(val: u32, nbits: u32) -> Option<u16> {
    debug_assert!(val < 10u32.pow(6));
    let dump_bits = 16 - nbits;
    let divisor = 5u32.pow(6) * 2;
    let shift = val << (16 - 6 + 1) >> dump_bits;
    let round = shift + (divisor / 2);
    if round >> nbits >= divisor {
        None
    } else {
        Some((round / divisor) as u16)
    }
}
// 5^13 × 2 < 2^32 => (10^13 - 1) × 2^(32-13+1) < 2^64
// Returns None for large fractions that are rounded to 1.0
fn dec13_to_bin32(val: u64, nbits: u32) -> Option<u32> {
    debug_assert!(val < 10u64.pow(13));
    let dump_bits = 32 - nbits;
    let divisor = 5u64.pow(13) * 2;
    let shift = val << (32 - 13 + 1) >> dump_bits;
    let round = shift + (divisor / 2);
    if round >> nbits >= divisor {
        None
    } else {
        Some((round / divisor) as u32)
    }
}
// 5^27 × 2 < 2^64 => (10^27 - 1) × 2^(64-27+1) < 2^128
// Returns None for large fractions that are rounded to 1.0
fn dec27_to_bin64(val: u128, nbits: u32) -> Option<u64> {
    debug_assert!(val < 10u128.pow(27));
    let dump_bits = 64 - nbits;
    let divisor = 5u128.pow(27) * 2;
    let shift = val << (64 - 27 + 1) >> dump_bits;
    let round = shift + (divisor / 2);
    if round >> nbits >= divisor {
        None
    } else {
        Some((round / divisor) as u64)
    }
}
// 5^54 × 2 < 2^128 => (10^54 - 1) × 2^(128-54+1) < 2^256
// Returns None for large fractions that are rounded to 1.0
fn dec27_27_to_bin128(hi: u128, lo: u128, nbits: u32) -> Option<u128> {
    debug_assert!(hi < 10u128.pow(27));
    debug_assert!(lo < 10u128.pow(27));
    let dump_bits = 128 - nbits;
    let divisor = 5u128.pow(54) * 2;
    // we actually need to combine (10^27*hi + lo) << (128 - 54 + 1)
    let (hi_hi, hi_lo) = mul_hi_lo(hi, 10u128.pow(27));
    let (comb_lo, overflow) = hi_lo.overflowing_add(lo);
    let comb_hi = if overflow { hi_hi + 1 } else { hi_hi };
    let shift_lo;
    let shift_hi;
    match nbits.cmp(&(54 - 1)) {
        Ordering::Less => {
            let shr = (54 - 1) - nbits;
            shift_lo = (comb_lo >> shr) | (comb_hi << (128 - shr));
            shift_hi = comb_hi >> shr;
        }
        Ordering::Greater => {
            let shl = nbits - (54 - 1);
            shift_lo = comb_lo << shl;
            shift_hi = (comb_hi << shl) | (comb_lo >> (128 - shl));
        }
        Ordering::Equal => {
            shift_lo = comb_lo;
            shift_hi = comb_hi;
        }
    };
    let (round_lo, overflow) = shift_lo.overflowing_add(divisor / 2);
    let round_hi = if overflow { shift_hi + 1 } else { shift_hi };
    let whole_compare = if dump_bits == 0 {
        round_hi
    } else if nbits == 0 {
        round_lo
    } else {
        (round_lo >> nbits) | (round_hi << dump_bits)
    };
    if whole_compare >= divisor {
        None
    } else {
        Some(div_wide(round_hi, round_lo, divisor))
    }
}
fn mul_hi_lo(lhs: u128, rhs: u128) -> (u128, u128) {
    const LO: u128 = !(!0 << 64);
    let (lhs_hi, lhs_lo) = (lhs >> 64, lhs & LO);
    let (rhs_hi, rhs_lo) = (rhs >> 64, rhs & LO);
    let lhs_lo_rhs_lo = lhs_lo.wrapping_mul(rhs_lo);
    let lhs_hi_rhs_lo = lhs_hi.wrapping_mul(rhs_lo);
    let lhs_lo_rhs_hi = lhs_lo.wrapping_mul(rhs_hi);
    let lhs_hi_rhs_hi = lhs_hi.wrapping_mul(rhs_hi);

    let col01 = lhs_lo_rhs_lo;
    let (col01_hi, col01_lo) = (col01 >> 64, col01 & LO);
    let partial_col12 = lhs_hi_rhs_lo + col01_hi;
    let (col12, carry_col3) = partial_col12.overflowing_add(lhs_lo_rhs_hi);
    let (col12_hi, col12_lo) = (col12 >> 64, col12 & LO);
    let ans01 = (col12_lo << 64) + col01_lo;
    let ans23 = lhs_hi_rhs_hi + col12_hi + if carry_col3 { 1u128 << 64 } else { 0 };
    (ans23, ans01)
}
fn div_wide(dividend_hi: u128, dividend_lo: u128, divisor: u128) -> u128 {
    divisor.lo_div_from(dividend_hi, dividend_lo)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Parse<'a> {
    neg: bool,
    int: &'a str,
    frac: &'a str,
}

/**
An error which can be returned when parsing a fixed-point number.

# Examples

```rust
use fixed::{types::I16F16, ParseFixedError};
// This string is not a fixed-point number.
let s = "something completely different (_!_!_)";
let error: ParseFixedError = match s.parse::<I16F16>() {
    Ok(_) => unreachable!(),
    Err(error) => error,
};
println!("Parse error: {}", error);
```
*/
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseFixedError {
    kind: ParseErrorKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParseErrorKind {
    InvalidDigit,
    NoDigits,
    TooManyPoints,
    Overflow,
}

macro_rules! err {
    ($cond:expr, $kind:ident) => {
        if $cond {
            err!($kind);
        }
    };
    ($kind:ident) => {
        return Err(ParseFixedError {
            kind: ParseErrorKind::$kind,
        });
    };
}

impl Display for ParseFixedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use self::ParseErrorKind::*;
        let message = match self.kind {
            InvalidDigit => "invalid digit found in string",
            NoDigits => "string has no digits",
            TooManyPoints => "more than one decimal point found in string",
            Overflow => "overflow",
        };
        Display::fmt(message, f)
    }
}

// also trims zeros at start of int and at end of frac
fn parse_bounds(s: &str, can_be_neg: bool, radix: u32) -> Result<Parse<'_>, ParseFixedError> {
    let mut sign: Option<bool> = None;
    let mut trimmed_int_start: Option<usize> = None;
    let mut point: Option<usize> = None;
    let mut trimmed_frac_end: Option<usize> = None;
    let mut has_any_digit = false;

    for (index, &byte) in s.as_bytes().iter().enumerate() {
        match (byte, radix) {
            (b'+', _) => {
                err!(
                    sign.is_some() || point.is_some() || has_any_digit,
                    InvalidDigit
                );
                sign = Some(false);
                continue;
            }
            (b'-', _) => {
                err!(
                    !can_be_neg || sign.is_some() || point.is_some() || has_any_digit,
                    InvalidDigit
                );
                sign = Some(true);
                continue;
            }
            (b'.', _) => {
                err!(point.is_some(), TooManyPoints);
                point = Some(index);
                trimmed_frac_end = Some(index + 1);
                continue;
            }
            (b'0'..=b'1', 2)
            | (b'0'..=b'7', 8)
            | (b'0'..=b'9', 10)
            | (b'0'..=b'9', 16)
            | (b'a'..=b'f', 16)
            | (b'A'..=b'F', 16) => {
                if trimmed_int_start.is_none() && point.is_none() && byte != b'0' {
                    trimmed_int_start = Some(index);
                }
                if trimmed_frac_end.is_some() && byte != b'0' {
                    trimmed_frac_end = Some(index + 1);
                }
                has_any_digit = true;
            }
            _ => {
                err!(InvalidDigit);
            }
        }
    }
    err!(!has_any_digit, NoDigits);
    let neg = sign.unwrap_or(false);
    let int = match (trimmed_int_start, point) {
        (Some(start), Some(point)) => &s[start..point],
        (Some(start), None) => &s[start..],
        (None, _) => "",
    };
    let frac = match (point, trimmed_frac_end) {
        (Some(point), Some(end)) => &s[(point + 1)..end],
        _ => "",
    };
    Ok(Parse { neg, int, frac })
}

pub(crate) trait FromStrRadix: Sized {
    type Err;
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err>;
}

macro_rules! impl_from_str {
    ($Fixed:ident, $LeEqU:ident, $method:ident) => {
        impl<Frac: $LeEqU> FromStr for $Fixed<Frac> {
            type Err = ParseFixedError;
            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $method(s, 10, Self::int_nbits(), Self::frac_nbits()).map(Self::from_bits)
            }
        }
        impl<Frac: $LeEqU> FromStrRadix for $Fixed<Frac> {
            type Err = ParseFixedError;
            #[inline]
            fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err> {
                $method(s, radix, Self::int_nbits(), Self::frac_nbits()).map(Self::from_bits)
            }
        }
    };
}

macro_rules! impl_from_str_signed {
    (
        $Fixed:ident, $LeEqU:ident, $Bits:ident;
        fn $all:ident;
        $int:ident;
        $frac:ident;
    ) => {
        impl_from_str! { $Fixed, $LeEqU, $all }

        fn $all(
            s: &str,
            radix: u32,
            int_nbits: u32,
            frac_nbits: u32,
        ) -> Result<$Bits, ParseFixedError> {
            let Parse { neg, int, frac } = parse_bounds(s, true, radix)?;
            let (abs_frac, whole_frac) = match $frac(frac, radix, frac_nbits) {
                Some(frac) => (frac, false),
                None => (0, true),
            };
            let abs_int = match $int(int, radix, int_nbits, whole_frac) {
                Some(i) => i,
                None => err!(Overflow),
            };
            let abs = abs_int | abs_frac;
            let max_abs = if neg {
                <$Bits as SealedInt>::Unsigned::MSB
            } else {
                <$Bits as SealedInt>::Unsigned::MSB - 1
            };
            err!(abs > max_abs, Overflow);
            let f = if neg {
                abs.wrapping_neg() as $Bits
            } else {
                abs as $Bits
            };
            Ok(f)
        }
    };
}

macro_rules! impl_from_str_unsigned {
    (
        $Fixed:ident, $LeEqU:ident, $Bits:ident;
        fn $all:ident;
        fn $int:ident, ($int_half:ident, $int_half_cond:expr);
        fn $frac:ident, ($frac_half:ident, $frac_half_cond:expr);
        $frac_dec:ident;
    ) => {
        impl_from_str! { $Fixed, $LeEqU, $all }

        fn $all(
            s: &str,
            radix: u32,
            int_nbits: u32,
            frac_nbits: u32,
        ) -> Result<$Bits, ParseFixedError> {
            let Parse { int, frac, .. } = parse_bounds(s, false, radix)?;
            let (frac, whole_frac) = match $frac(frac, radix, frac_nbits) {
                Some(frac) => (frac, false),
                None => (0, true),
            };
            let int = match $int(int, radix, int_nbits, whole_frac) {
                Some(i) => i,
                None => err!(Overflow),
            };
            Ok(int | frac)
        }

        fn $int(int: &str, radix: u32, nbits: u32, whole_frac: bool) -> Option<$Bits> {
            const HALF: u32 = <$Bits as SealedInt>::NBITS / 2;
            if $int_half_cond && nbits <= HALF {
                return $int_half(int, radix, nbits, whole_frac).map(|x| $Bits::from(x) << HALF);
            }
            if int.is_empty() && !whole_frac {
                return Some(0);
            } else if int.is_empty() || nbits == 0 {
                return None;
            }
            let mut parsed_int = match radix {
                2 => bin_str_int_to_bin(int)?,
                8 => oct_str_int_to_bin(int)?,
                16 => hex_str_int_to_bin(int)?,
                10 => int.parse::<$Bits>().ok()?,
                _ => unreachable!(),
            };
            if whole_frac {
                parsed_int = parsed_int.checked_add(1)?;
            }
            let remove_bits = <$Bits as SealedInt>::NBITS - nbits;
            if remove_bits > 0 && (parsed_int >> nbits) != 0 {
                None
            } else {
                Some(parsed_int << remove_bits)
            }
        }

        fn $frac(frac: &str, radix: u32, nbits: u32) -> Option<$Bits> {
            if $frac_half_cond && nbits <= <$Bits as SealedInt>::NBITS / 2 {
                return $frac_half(frac, radix, nbits).map($Bits::from);
            }
            if frac.is_empty() {
                return Some(0);
            }
            match radix {
                2 => bin_str_frac_to_bin(frac, nbits),
                8 => oct_str_frac_to_bin(frac, nbits),
                16 => hex_str_frac_to_bin(frac, nbits),
                10 => $frac_dec(frac, nbits),
                _ => unreachable!(),
            }
        }
    };
}

macro_rules! impl_from_str_unsigned_not128 {
    (
        $Fixed:ident, $LeEqU:ident, $Bits:ident;
        fn $all:ident;
        fn $int:ident, ($int_half:ident, $int_half_cond:expr);
        fn $frac:ident, ($frac_half:ident, $frac_half_cond:expr);
        fn $frac_dec:ident;
        $decode_frac:ident, $dec_frac_digits:expr, $DoubleBits:ident;
    ) => {
        impl_from_str_unsigned! {
            $Fixed, $LeEqU, $Bits;
            fn $all;
            fn $int, ($int_half, $int_half_cond);
            fn $frac, ($frac_half, $frac_half_cond);
            $frac_dec;
        }

        fn $frac_dec(frac: &str, nbits: u32) -> Option<$Bits> {
            let end = cmp::min(frac.len(), $dec_frac_digits);
            let rem = $dec_frac_digits - end;
            let ten: $DoubleBits = 10;
            let i = frac[..end].parse::<$DoubleBits>().unwrap() * ten.pow(rem as u32);
            $decode_frac(i, nbits)
        }
    };
}

impl_from_str_signed! {
    FixedI8, LeEqU8, i8;
    fn from_str_i8;
    get_int8;
    get_frac8;
}
impl_from_str_unsigned_not128! {
    FixedU8, LeEqU8, u8;
    fn from_str_u8;
    fn get_int8, (get_int8, false);
    fn get_frac8, (get_frac8, false);
    fn get_frac8_dec;
    dec3_to_bin8, 3, u16;
}

impl_from_str_signed! {
    FixedI16, LeEqU16, i16;
    fn from_str_i16;
    get_int16;
    get_frac16;
}
impl_from_str_unsigned_not128! {
    FixedU16, LeEqU16, u16;
    fn from_str_u16;
    fn get_int16, (get_int8, true);
    fn get_frac16, (get_frac8, true);
    fn get_frac16_dec;
    dec6_to_bin16, 6, u32;
}

impl_from_str_signed! {
    FixedI32, LeEqU32, i32;
    fn from_str_i32;
    get_int32;
    get_frac32;
}
impl_from_str_unsigned_not128! {
    FixedU32, LeEqU32, u32;
    fn from_str_u32;
    fn get_int32, (get_int16, true);
    fn get_frac32, (get_frac16, true);
    fn get_frac32_dec;
    dec13_to_bin32, 13, u64;
}

impl_from_str_signed! {
    FixedI64, LeEqU64, i64;
    fn from_str_i64;
    get_int64;
    get_frac64;
}
impl_from_str_unsigned_not128! {
    FixedU64, LeEqU64, u64;
    fn from_str_u64;
    fn get_int64, (get_int32, true);
    fn get_frac64, (get_frac32, true);
    fn get_frac64_dec;
    dec27_to_bin64, 27, u128;
}

impl_from_str_signed! {
    FixedI128, LeEqU128, i128;
    fn from_str_i128;
    get_int128;
    get_frac128;
}
impl_from_str_unsigned! {
    FixedU128, LeEqU128, u128;
    fn from_str_u128;
    fn get_int128, (get_int64, true);
    fn get_frac128, (get_frac64, true);
    get_frac128_dec;
}

fn get_frac128_dec(frac: &str, nbits: u32) -> Option<u128> {
    let (hi, lo) = if frac.len() <= 27 {
        let rem = 27 - frac.len();
        let hi = frac.parse::<u128>().unwrap() * 10u128.pow(rem as u32);
        (hi, 0)
    } else {
        let hi = frac[..27].parse::<u128>().unwrap();
        let lo_end = cmp::min(frac.len(), 54);
        let rem = 54 - lo_end;
        let lo = frac[27..lo_end].parse::<u128>().unwrap() * 10u128.pow(rem as u32);
        (hi, lo)
    };
    dec27_27_to_bin128(hi, lo, nbits)
}

#[cfg(test)]
mod tests {
    use crate::{from_str::*, traits::Fixed};
    use core::fmt::Debug;

    #[test]
    fn check_dec3() {
        let two_pow = 8f64.exp2();
        let limit = 1000;
        for i in 0..limit {
            let ans = dec3_to_bin8(i, 8);
            let approx = two_pow * f64::from(i) / f64::from(limit);
            let error = (ans.map(f64::from).unwrap_or(two_pow) - approx).abs();
            assert!(
                error <= 0.5,
                "i {} ans {:?}  approx {} error {}",
                i,
                ans,
                approx,
                error
            );
        }
    }

    #[test]
    fn check_dec6() {
        let two_pow = 16f64.exp2();
        let limit = 1_000_000;
        for i in 0..limit {
            let ans = dec6_to_bin16(i, 16);
            let approx = two_pow * f64::from(i) / f64::from(limit);
            let error = (ans.map(f64::from).unwrap_or(two_pow) - approx).abs();
            assert!(
                error <= 0.5,
                "i {} ans {:?}  approx {} error {}",
                i,
                ans,
                approx,
                error
            );
        }
    }

    #[test]
    fn check_dec13() {
        let two_pow = 32f64.exp2();
        let limit = 10_000_000_000_000;
        for iter in 0..1_000_000 {
            for &i in &[
                iter,
                limit / 4 - 1 - iter,
                limit / 4 + iter,
                limit / 3 - 1 - iter,
                limit / 3 + iter,
                limit / 2 - 1 - iter,
                limit / 2 + iter,
                limit - iter - 1,
            ] {
                let ans = dec13_to_bin32(i, 32);
                let approx = two_pow * i as f64 / limit as f64;
                let error = (ans.map(f64::from).unwrap_or(two_pow) - approx).abs();
                assert!(
                    error <= 0.5,
                    "i {} ans {:?}  approx {} error {}",
                    i,
                    ans,
                    approx,
                    error
                );
            }
        }
    }

    #[test]
    fn check_dec27() {
        let two_pow = 64f64.exp2();
        let limit = 1_000_000_000_000_000_000_000_000_000;
        for iter in 0..200_000 {
            for &i in &[
                iter,
                limit / 4 - 1 - iter,
                limit / 4 + iter,
                limit / 3 - 1 - iter,
                limit / 3 + iter,
                limit / 2 - 1 - iter,
                limit / 2 + iter,
                limit - iter - 1,
            ] {
                let ans = dec27_to_bin64(i, 64);
                let approx = two_pow * i as f64 / limit as f64;
                let error = (ans.map(|x| x as f64).unwrap_or(two_pow) - approx).abs();
                assert!(
                    error <= 0.5,
                    "i {} ans {:?}  approx {} error {}",
                    i,
                    ans,
                    approx,
                    error
                );
            }
        }
    }

    #[test]
    fn check_dec27_27() {
        let nines = 10u128.pow(27) - 1;
        let zeros = 0;
        let too_big = dec27_27_to_bin128(nines, nines, 128);
        assert_eq!(too_big, None);
        let big = dec27_27_to_bin128(nines, zeros, 128);
        assert_eq!(
            big,
            Some(340_282_366_920_938_463_463_374_607_091_485_844_535)
        );
        let small = dec27_27_to_bin128(zeros, nines, 128);
        assert_eq!(small, Some(340_282_366_921));
        let zero = dec27_27_to_bin128(zeros, zeros, 128);
        assert_eq!(zero, Some(0));
        let x = dec27_27_to_bin128(
            123_456_789_012_345_678_901_234_567,
            987_654_321_098_765_432_109_876_543,
            128,
        );
        assert_eq!(x, Some(42_010_168_377_579_896_403_540_037_811_203_677_112));
    }

    #[test]
    fn check_parse_bounds() {
        let Parse { neg, int, frac } = parse_bounds("-12.34", true, 10).unwrap();
        assert_eq!((neg, int, frac), (true, "12", "34"));
        let Parse { neg, int, frac } = parse_bounds("012.", true, 10).unwrap();
        assert_eq!((neg, int, frac), (false, "12", ""));
        let Parse { neg, int, frac } = parse_bounds("+.340", false, 10).unwrap();
        assert_eq!((neg, int, frac), (false, "", "34"));
        let Parse { neg, int, frac } = parse_bounds("0", false, 10).unwrap();
        assert_eq!((neg, int, frac), (false, "", ""));
        let Parse { neg, int, frac } = parse_bounds("-.C1A0", true, 16).unwrap();
        assert_eq!((neg, int, frac), (true, "", "C1A"));

        let ParseFixedError { kind } = parse_bounds("0 ", true, 10).unwrap_err();
        assert_eq!(kind, ParseErrorKind::InvalidDigit);
        let ParseFixedError { kind } = parse_bounds("+.", true, 10).unwrap_err();
        assert_eq!(kind, ParseErrorKind::NoDigits);
        let ParseFixedError { kind } = parse_bounds(".1.", true, 10).unwrap_err();
        assert_eq!(kind, ParseErrorKind::TooManyPoints);
        let ParseFixedError { kind } = parse_bounds("1+2", true, 10).unwrap_err();
        assert_eq!(kind, ParseErrorKind::InvalidDigit);
        let ParseFixedError { kind } = parse_bounds("1-2", true, 10).unwrap_err();
        assert_eq!(kind, ParseErrorKind::InvalidDigit);
        let ParseFixedError { kind } = parse_bounds("-12", false, 10).unwrap_err();
        assert_eq!(kind, ParseErrorKind::InvalidDigit);
    }

    fn assert_ok<F>(s: &str, bits: F::Bits)
    where
        F: Fixed + FromStr<Err = ParseFixedError>,
        F::Bits: Eq + Debug,
    {
        match s.parse::<F>() {
            Ok(f) => assert_eq!(f.to_bits(), bits),
            Err(e) => panic!("could not parse {}: {}", s, e),
        }
    }
    fn assert_err<F>(s: &str, kind: ParseErrorKind)
    where
        F: Fixed + FromStr<Err = ParseFixedError>,
    {
        match s.parse::<F>() {
            Ok(f) => panic!("incorrectly parsed {} as {}", s, f),
            Err(ParseFixedError { kind: err }) => assert_eq!(err, kind),
        }
    }

    fn assert_ok_radix<F>(s: &str, radix: u32, bits: F::Bits)
    where
        F: Fixed + FromStrRadix<Err = ParseFixedError>,
        F::Bits: Eq + Debug,
    {
        match <F as FromStrRadix>::from_str_radix(s, radix) {
            Ok(f) => assert_eq!(f.to_bits(), bits),
            Err(e) => panic!("could not parse {}: {}", s, e),
        }
    }
    fn assert_err_radix<F>(s: &str, radix: u32, kind: ParseErrorKind)
    where
        F: Fixed + FromStrRadix<Err = ParseFixedError>,
    {
        match <F as FromStrRadix>::from_str_radix(s, radix) {
            Ok(f) => panic!("incorrectly parsed {} as {}", s, f),
            Err(ParseFixedError { kind: err }) => assert_eq!(err, kind),
        }
    }

    #[test]
    fn check_i8_u8_from_str() {
        use crate::types::*;

        assert_err::<I0F8>("-1", ParseErrorKind::Overflow);
        assert_err::<I0F8>("-0.502", ParseErrorKind::Overflow);
        assert_ok::<I0F8>("-0.501", -0x80);
        assert_ok::<I0F8>("0.498", 0x7F);
        assert_err::<I0F8>("0.499", ParseErrorKind::Overflow);
        assert_err::<I0F8>("1", ParseErrorKind::Overflow);

        assert_err::<I4F4>("-8.04", ParseErrorKind::Overflow);
        assert_ok::<I4F4>("-8.03", -0x80);
        assert_ok::<I4F4>("7.96", 0x7F);
        assert_err::<I4F4>("7.97", ParseErrorKind::Overflow);

        assert_err::<I8F0>("-128.5", ParseErrorKind::Overflow);
        assert_ok::<I8F0>("-128.499", -0x80);
        assert_ok::<I8F0>("127.499", 0x7F);
        assert_err::<I8F0>("127.5", ParseErrorKind::Overflow);

        assert_err::<U0F8>("-0", ParseErrorKind::InvalidDigit);
        assert_ok::<U0F8>("0.498", 0x7F);
        assert_ok::<U0F8>("0.499", 0x80);
        assert_ok::<U0F8>("0.998", 0xFF);
        assert_err::<U0F8>("0.999", ParseErrorKind::Overflow);
        assert_err::<U0F8>("1", ParseErrorKind::Overflow);

        assert_ok::<U4F4>("7.96", 0x7F);
        assert_ok::<U4F4>("7.97", 0x80);
        assert_ok::<U4F4>("15.96", 0xFF);
        assert_err::<U4F4>("15.97", ParseErrorKind::Overflow);

        assert_ok::<U8F0>("127.499", 0x7F);
        assert_ok::<U8F0>("127.5", 0x80);
        assert_ok::<U8F0>("255.499", 0xFF);
        assert_err::<U8F0>("255.5", ParseErrorKind::Overflow);
    }

    #[test]
    fn check_i16_u16_from_str() {
        use crate::types::*;

        assert_err::<I0F16>("-1", ParseErrorKind::Overflow);
        assert_err::<I0F16>("-0.500008", ParseErrorKind::Overflow);
        assert_ok::<I0F16>("-0.500007", -0x8000);
        assert_ok::<I0F16>("0.499992", 0x7FFF);
        assert_err::<I0F16>("0.499993", ParseErrorKind::Overflow);
        assert_err::<I0F16>("1", ParseErrorKind::Overflow);

        assert_err::<I8F8>("-128.002", ParseErrorKind::Overflow);
        assert_ok::<I8F8>("-128.001", -0x8000);
        assert_ok::<I8F8>("127.998", 0x7FFF);
        assert_err::<I8F8>("127.999", ParseErrorKind::Overflow);

        assert_err::<I16F0>("-32768.5", ParseErrorKind::Overflow);
        assert_ok::<I16F0>("-32768.499999", -0x8000);
        assert_ok::<I16F0>("32767.499999", 0x7FFF);
        assert_err::<I16F0>("32767.5", ParseErrorKind::Overflow);

        assert_err::<U0F16>("-0", ParseErrorKind::InvalidDigit);
        assert_ok::<U0F16>("0.499992", 0x7FFF);
        assert_ok::<U0F16>("0.499993", 0x8000);
        assert_ok::<U0F16>("0.999992", 0xFFFF);
        assert_err::<U0F16>("0.999993", ParseErrorKind::Overflow);
        assert_err::<U0F16>("1", ParseErrorKind::Overflow);

        assert_ok::<U8F8>("127.998", 0x7FFF);
        assert_ok::<U8F8>("127.999", 0x8000);
        assert_ok::<U8F8>("255.998", 0xFFFF);
        assert_err::<U8F8>("255.999", ParseErrorKind::Overflow);

        assert_ok::<U16F0>("32767.499999", 0x7FFF);
        assert_ok::<U16F0>("32767.5", 0x8000);
        assert_ok::<U16F0>("65535.499999", 0xFFFF);
        assert_err::<U16F0>("65535.5", ParseErrorKind::Overflow);
    }

    #[test]
    fn check_i32_u32_from_str() {
        use crate::types::*;

        assert_err::<I0F32>("-1", ParseErrorKind::Overflow);
        assert_err::<I0F32>("-0.5000000002", ParseErrorKind::Overflow);
        assert_ok::<I0F32>("-0.5000000001", -0x8000_0000);
        assert_ok::<I0F32>("0.4999999998", 0x7FFF_FFFF);
        assert_err::<I0F32>("0.4999999999", ParseErrorKind::Overflow);
        assert_err::<I0F32>("1", ParseErrorKind::Overflow);

        assert_err::<I16F16>("-32768.000008", ParseErrorKind::Overflow);
        assert_ok::<I16F16>("-32768.000007", -0x8000_0000);
        assert_ok::<I16F16>("32767.999992", 0x7FFF_FFFF);
        assert_err::<I16F16>("32767.999993", ParseErrorKind::Overflow);

        assert_err::<I32F0>("-2147483648.5", ParseErrorKind::Overflow);
        assert_ok::<I32F0>("-2147483648.4999999999", -0x8000_0000);
        assert_ok::<I32F0>("2147483647.4999999999", 0x7FFF_FFFF);
        assert_err::<I32F0>("2147483647.5", ParseErrorKind::Overflow);

        assert_err::<U0F32>("-0", ParseErrorKind::InvalidDigit);
        assert_ok::<U0F32>("0.4999999998", 0x7FFF_FFFF);
        assert_ok::<U0F32>("0.4999999999", 0x8000_0000);
        assert_ok::<U0F32>("0.9999999998", 0xFFFF_FFFF);
        assert_err::<U0F32>("0.9999999999", ParseErrorKind::Overflow);
        assert_err::<U0F32>("1", ParseErrorKind::Overflow);

        assert_ok::<U16F16>("32767.999992", 0x7FFF_FFFF);
        assert_ok::<U16F16>("32767.999993", 0x8000_0000);
        assert_ok::<U16F16>("65535.999992", 0xFFFF_FFFF);
        assert_err::<U16F16>("65535.999993", ParseErrorKind::Overflow);

        assert_ok::<U32F0>("2147483647.4999999999", 0x7FFF_FFFF);
        assert_ok::<U32F0>("2147483647.5", 0x8000_0000);
        assert_ok::<U32F0>("4294967295.4999999999", 0xFFFF_FFFF);
        assert_err::<U32F0>("4294967295.5", ParseErrorKind::Overflow);
    }

    #[test]
    fn check_i16_u16_from_str_binary() {
        use crate::types::*;

        assert_err_radix::<I0F16>("-1", 2, ParseErrorKind::Overflow);
        assert_err_radix::<I0F16>("-0.100000000000000010", 2, ParseErrorKind::Overflow);
        assert_ok_radix::<I0F16>("-0.100000000000000001", 2, -0x8000);
        assert_ok_radix::<I0F16>("0.011111111111111101", 2, 0x7FFF);
        assert_err_radix::<I0F16>("0.011111111111111110", 2, ParseErrorKind::Overflow);
        assert_err_radix::<I0F16>("1", 2, ParseErrorKind::Overflow);

        assert_err_radix::<I8F8>("-10000000.0000000010", 2, ParseErrorKind::Overflow);
        assert_ok_radix::<I8F8>("-10000000.0000000001", 2, -0x8000);
        assert_ok_radix::<I8F8>("1111111.1111111101", 2, 0x7FFF);
        assert_err_radix::<I8F8>("1111111.1111111110", 2, ParseErrorKind::Overflow);

        assert_err_radix::<I16F0>("-1000000000000000.10", 2, ParseErrorKind::Overflow);
        assert_ok_radix::<I16F0>("-1000000000000000.01", 2, -0x8000);
        assert_ok_radix::<I16F0>("111111111111111.01", 2, 0x7FFF);
        assert_err_radix::<I16F0>("111111111111111.10", 2, ParseErrorKind::Overflow);

        assert_err_radix::<U0F16>("-0", 2, ParseErrorKind::InvalidDigit);
        assert_ok_radix::<U0F16>("0.011111111111111101", 2, 0x7FFF);
        assert_ok_radix::<U0F16>("0.011111111111111110", 2, 0x8000);
        assert_ok_radix::<U0F16>("0.111111111111111101", 2, 0xFFFF);
        assert_err_radix::<U0F16>("0.111111111111111110", 2, ParseErrorKind::Overflow);
        assert_err_radix::<U0F16>("1", 2, ParseErrorKind::Overflow);

        assert_ok_radix::<U8F8>("1111111.1111111101", 2, 0x7FFF);
        assert_ok_radix::<U8F8>("1111111.1111111110", 2, 0x8000);
        assert_ok_radix::<U8F8>("11111111.1111111101", 2, 0xFFFF);
        assert_err_radix::<U8F8>("11111111.1111111110", 2, ParseErrorKind::Overflow);

        assert_ok_radix::<U16F0>("111111111111111.01", 2, 0x7FFF);
        assert_ok_radix::<U16F0>("111111111111111.10", 2, 0x8000);
        assert_ok_radix::<U16F0>("1111111111111111.01", 2, 0xFFFF);
        assert_err_radix::<U16F0>("1111111111111111.10", 2, ParseErrorKind::Overflow);
    }

    #[test]
    fn check_i16_u16_from_str_octal() {
        use crate::types::*;

        assert_err_radix::<I0F16>("-1", 8, ParseErrorKind::Overflow);
        assert_err_radix::<I0F16>("-0.400002", 8, ParseErrorKind::Overflow);
        assert_ok_radix::<I0F16>("-0.400001", 8, -0x8000);
        assert_ok_radix::<I0F16>("0.377775", 8, 0x7FFF);
        assert_err_radix::<I0F16>("0.377776", 8, ParseErrorKind::Overflow);
        assert_err_radix::<I0F16>("1", 8, ParseErrorKind::Overflow);

        assert_err_radix::<I8F8>("-2000.0010", 8, ParseErrorKind::Overflow);
        assert_ok_radix::<I8F8>("-200.0007", 8, -0x8000);
        assert_ok_radix::<I8F8>("177.7767", 8, 0x7FFF);
        assert_err_radix::<I8F8>("177.7770", 8, ParseErrorKind::Overflow);

        assert_err_radix::<I16F0>("-100000.4", 8, ParseErrorKind::Overflow);
        assert_ok_radix::<I16F0>("-100000.3", 8, -0x8000);
        assert_ok_radix::<I16F0>("77777.3", 8, 0x7FFF);
        assert_err_radix::<I16F0>("77777.4", 8, ParseErrorKind::Overflow);

        assert_err_radix::<U0F16>("-0", 8, ParseErrorKind::InvalidDigit);
        assert_ok_radix::<U0F16>("0.377775", 8, 0x7FFF);
        assert_ok_radix::<U0F16>("0.377776", 8, 0x8000);
        assert_ok_radix::<U0F16>("0.777775", 8, 0xFFFF);
        assert_err_radix::<U0F16>("0.777776", 8, ParseErrorKind::Overflow);
        assert_err_radix::<U0F16>("1", 8, ParseErrorKind::Overflow);

        assert_ok_radix::<U8F8>("177.7767", 8, 0x7FFF);
        assert_ok_radix::<U8F8>("177.7770", 8, 0x8000);
        assert_ok_radix::<U8F8>("377.7767", 8, 0xFFFF);
        assert_err_radix::<U8F8>("377.7770", 8, ParseErrorKind::Overflow);

        assert_ok_radix::<U16F0>("77777.3", 8, 0x7FFF);
        assert_ok_radix::<U16F0>("77777.4", 8, 0x8000);
        assert_ok_radix::<U16F0>("177777.3", 8, 0xFFFF);
        assert_err_radix::<U16F0>("177777.4", 8, ParseErrorKind::Overflow);
    }

    #[test]
    fn check_i16_u16_from_str_hex() {
        use crate::types::*;

        assert_err_radix::<I0F16>("-1", 16, ParseErrorKind::Overflow);
        assert_err_radix::<I0F16>("-0.80008", 16, ParseErrorKind::Overflow);
        assert_ok_radix::<I0F16>("-0.80007", 16, -0x8000);
        assert_ok_radix::<I0F16>("0.7FFF7", 16, 0x7FFF);
        assert_err_radix::<I0F16>("0.7FFF8", 16, ParseErrorKind::Overflow);
        assert_err_radix::<I0F16>("1", 16, ParseErrorKind::Overflow);

        assert_err_radix::<I8F8>("-80.008", 16, ParseErrorKind::Overflow);
        assert_ok_radix::<I8F8>("-80.007", 16, -0x8000);
        assert_ok_radix::<I8F8>("7F.FF7", 16, 0x7FFF);
        assert_err_radix::<I8F8>("7F.FF8", 16, ParseErrorKind::Overflow);

        assert_err_radix::<I16F0>("-8000.8", 16, ParseErrorKind::Overflow);
        assert_ok_radix::<I16F0>("-8000.7", 16, -0x8000);
        assert_ok_radix::<I16F0>("7FFF.7", 16, 0x7FFF);
        assert_err_radix::<I16F0>("7FFF.8", 16, ParseErrorKind::Overflow);

        assert_err_radix::<U0F16>("-0", 16, ParseErrorKind::InvalidDigit);
        assert_ok_radix::<U0F16>("0.7FFF7", 16, 0x7FFF);
        assert_ok_radix::<U0F16>("0.7FFF8", 16, 0x8000);
        assert_ok_radix::<U0F16>("0.FFFF7", 16, 0xFFFF);
        assert_err_radix::<U0F16>("0.FFFF8", 16, ParseErrorKind::Overflow);
        assert_err_radix::<U0F16>("1", 16, ParseErrorKind::Overflow);

        assert_ok_radix::<U8F8>("7F.FF7", 16, 0x7FFF);
        assert_ok_radix::<U8F8>("7F.FF8", 16, 0x8000);
        assert_ok_radix::<U8F8>("FF.FF7", 16, 0xFFFF);
        assert_err_radix::<U8F8>("FF.FF8", 16, ParseErrorKind::Overflow);

        assert_ok_radix::<U16F0>("7FFF.7", 16, 0x7FFF);
        assert_ok_radix::<U16F0>("7FFF.8", 16, 0x8000);
        assert_ok_radix::<U16F0>("FFFF.7", 16, 0xFFFF);
        assert_err_radix::<U16F0>("FFFF.8", 16, ParseErrorKind::Overflow);
    }

    #[test]
    fn check_i64_u64_from_str() {
        use crate::types::*;

        assert_err::<I0F64>("-1", ParseErrorKind::Overflow);
        assert_err::<I0F64>("-0.50000000000000000003", ParseErrorKind::Overflow);
        assert_ok::<I0F64>("-0.50000000000000000002", -0x8000_0000_0000_0000);
        assert_ok::<I0F64>("0.49999999999999999997", 0x7FFF_FFFF_FFFF_FFFF);
        assert_err::<I0F64>("0.49999999999999999998", ParseErrorKind::Overflow);
        assert_err::<I0F64>("1", ParseErrorKind::Overflow);

        assert_err::<I32F32>("-2147483648.0000000002", ParseErrorKind::Overflow);
        assert_ok::<I32F32>("-2147483648.0000000001", -0x8000_0000_0000_0000);
        assert_ok::<I32F32>("2147483647.9999999998", 0x7FFF_FFFF_FFFF_FFFF);
        assert_err::<I32F32>("2147483647.9999999999", ParseErrorKind::Overflow);

        assert_err::<I64F0>("-9223372036854775808.5", ParseErrorKind::Overflow);
        assert_ok::<I64F0>(
            "-9223372036854775808.49999999999999999999",
            -0x8000_0000_0000_0000,
        );
        assert_ok::<I64F0>(
            "9223372036854775807.49999999999999999999",
            0x7FFF_FFFF_FFFF_FFFF,
        );
        assert_err::<I64F0>("9223372036854775807.5", ParseErrorKind::Overflow);

        assert_err::<U0F64>("-0", ParseErrorKind::InvalidDigit);
        assert_ok::<U0F64>("0.49999999999999999997", 0x7FFF_FFFF_FFFF_FFFF);
        assert_ok::<U0F64>("0.49999999999999999998", 0x8000_0000_0000_0000);
        assert_ok::<U0F64>("0.99999999999999999997", 0xFFFF_FFFF_FFFF_FFFF);
        assert_err::<U0F64>("0.99999999999999999998", ParseErrorKind::Overflow);
        assert_err::<U0F64>("1", ParseErrorKind::Overflow);

        assert_ok::<U32F32>("2147483647.9999999998", 0x7FFF_FFFF_FFFF_FFFF);
        assert_ok::<U32F32>("2147483647.9999999999", 0x8000_0000_0000_0000);
        assert_ok::<U32F32>("4294967295.9999999998", 0xFFFF_FFFF_FFFF_FFFF);
        assert_err::<U32F32>("4294967295.9999999999", ParseErrorKind::Overflow);

        assert_ok::<U64F0>(
            "9223372036854775807.49999999999999999999",
            0x7FFF_FFFF_FFFF_FFFF,
        );
        assert_ok::<U64F0>("9223372036854775807.5", 0x8000_0000_0000_0000);
        assert_ok::<U64F0>(
            "18446744073709551615.49999999999999999999",
            0xFFFF_FFFF_FFFF_FFFF,
        );
        assert_err::<U64F0>("18446744073709551615.5", ParseErrorKind::Overflow);
    }

    #[test]
    fn check_i128_u128_from_str() {
        use crate::types::*;

        assert_err::<I0F128>("-1", ParseErrorKind::Overflow);
        assert_err::<I0F128>(
            "-0.500000000000000000000000000000000000002",
            ParseErrorKind::Overflow,
        );
        assert_ok::<I0F128>(
            "-0.500000000000000000000000000000000000001",
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
        );
        assert_ok::<I0F128>(
            "0.499999999999999999999999999999999999998",
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_err::<I0F128>(
            "0.499999999999999999999999999999999999999",
            ParseErrorKind::Overflow,
        );
        assert_err::<I0F128>("1", ParseErrorKind::Overflow);

        assert_err::<I64F64>(
            "-9223372036854775808.00000000000000000003",
            ParseErrorKind::Overflow,
        );
        assert_ok::<I64F64>(
            "-9223372036854775808.00000000000000000002",
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
        );
        assert_ok::<I64F64>(
            "9223372036854775807.99999999999999999997",
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_err::<I64F64>(
            "9223372036854775807.99999999999999999998",
            ParseErrorKind::Overflow,
        );

        assert_err::<I128F0>(
            "-170141183460469231731687303715884105728.5",
            ParseErrorKind::Overflow,
        );
        assert_ok::<I128F0>(
            "-170141183460469231731687303715884105728.4999999999999999999999999999999999999999",
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
        );
        assert_ok::<I128F0>(
            "170141183460469231731687303715884105727.4999999999999999999999999999999999999999",
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_err::<I128F0>(
            "170141183460469231731687303715884105727.5",
            ParseErrorKind::Overflow,
        );

        assert_err::<U0F128>("-0", ParseErrorKind::InvalidDigit);
        assert_ok::<U0F128>(
            "0.499999999999999999999999999999999999998",
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_ok::<U0F128>(
            "0.499999999999999999999999999999999999999",
            0x8000_0000_0000_0000_0000_0000_0000_0000,
        );
        assert_ok::<U0F128>(
            "0.999999999999999999999999999999999999998",
            0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_err::<U0F128>(
            "0.999999999999999999999999999999999999999",
            ParseErrorKind::Overflow,
        );
        assert_err::<U0F128>("1", ParseErrorKind::Overflow);

        assert_ok::<U64F64>(
            "9223372036854775807.99999999999999999997",
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_ok::<U64F64>(
            "9223372036854775807.99999999999999999998",
            0x8000_0000_0000_0000_0000_0000_0000_0000,
        );
        assert_ok::<U64F64>(
            "18446744073709551615.99999999999999999997",
            0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_err::<U64F64>(
            "18446744073709551615.99999999999999999998",
            ParseErrorKind::Overflow,
        );

        assert_ok::<U128F0>(
            "170141183460469231731687303715884105727.4999999999999999999999999999999999999999",
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_ok::<U128F0>(
            "170141183460469231731687303715884105727.5",
            0x8000_0000_0000_0000_0000_0000_0000_0000,
        );
        assert_ok::<U128F0>(
            "340282366920938463463374607431768211455.4999999999999999999999999999999999999999",
            0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_err::<U128F0>(
            "340282366920938463463374607431768211455.5",
            ParseErrorKind::Overflow,
        );
    }

    #[test]
    fn check_i128_u128_from_str_hex() {
        use crate::types::*;

        assert_err_radix::<I0F128>("-1", 16, ParseErrorKind::Overflow);
        assert_err_radix::<I0F128>(
            "-0.800000000000000000000000000000008",
            16,
            ParseErrorKind::Overflow,
        );
        assert_ok_radix::<I0F128>(
            "-0.800000000000000000000000000000007",
            16,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
        );
        assert_ok_radix::<I0F128>(
            "0.7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF7",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_err_radix::<I0F128>(
            "0.7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF8",
            16,
            ParseErrorKind::Overflow,
        );
        assert_err_radix::<I0F128>("1", 16, ParseErrorKind::Overflow);

        assert_err_radix::<I64F64>(
            "-8000000000000000.00000000000000008",
            16,
            ParseErrorKind::Overflow,
        );
        assert_ok_radix::<I64F64>(
            "-8000000000000000.00000000000000007",
            16,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
        );
        assert_ok_radix::<I64F64>(
            "7FFFFFFFFFFFFFFF.FFFFFFFFFFFFFFFF7",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_err_radix::<I64F64>(
            "7FFFFFFFFFFFFFFF.FFFFFFFFFFFFFFFF8",
            16,
            ParseErrorKind::Overflow,
        );

        assert_err_radix::<I128F0>(
            "-80000000000000000000000000000000.8",
            16,
            ParseErrorKind::Overflow,
        );
        assert_ok_radix::<I128F0>(
            "-80000000000000000000000000000000.7",
            16,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
        );
        assert_ok_radix::<I128F0>(
            "7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.7",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_err_radix::<I128F0>(
            "7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.8",
            16,
            ParseErrorKind::Overflow,
        );

        assert_err_radix::<U0F128>("-0", 16, ParseErrorKind::InvalidDigit);
        assert_ok_radix::<U0F128>(
            "0.7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF7",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_ok_radix::<U0F128>(
            "0.7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF8",
            16,
            0x8000_0000_0000_0000_0000_0000_0000_0000,
        );
        assert_ok_radix::<U0F128>(
            "0.FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF7",
            16,
            0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_err_radix::<U0F128>(
            "0.FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF8",
            16,
            ParseErrorKind::Overflow,
        );
        assert_err_radix::<U0F128>("1", 16, ParseErrorKind::Overflow);

        assert_ok_radix::<U64F64>(
            "7FFFFFFFFFFFFFFF.FFFFFFFFFFFFFFFF7",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_ok_radix::<U64F64>(
            "7FFFFFFFFFFFFFFF.FFFFFFFFFFFFFFFF8",
            16,
            0x8000_0000_0000_0000_0000_0000_0000_0000,
        );
        assert_ok_radix::<U64F64>(
            "FFFFFFFFFFFFFFFF.FFFFFFFFFFFFFFFF7",
            16,
            0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_err_radix::<U64F64>(
            "FFFFFFFFFFFFFFFF.FFFFFFFFFFFFFFFF8",
            16,
            ParseErrorKind::Overflow,
        );

        assert_ok_radix::<U128F0>(
            "7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.7",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_ok_radix::<U128F0>(
            "7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.8",
            16,
            0x8000_0000_0000_0000_0000_0000_0000_0000,
        );
        assert_ok_radix::<U128F0>(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.7",
            16,
            0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
        );
        assert_err_radix::<U128F0>(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.8",
            16,
            ParseErrorKind::Overflow,
        );
    }
}