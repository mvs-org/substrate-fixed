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

use core::fmt::{Debug, Display};

pub trait SealedInt: Copy + Ord + Debug + Display {
    type Unsigned: SealedInt;

    fn is_signed() -> bool;
    fn nbits() -> u32;
    fn one_shl(shift: u32) -> Self;
    fn all_ones_shl(shift: u32) -> Self;
    fn is_zero(self) -> bool;

    fn to_fixed_neg_abs_overflow(self, frac_bits: u32, int_bits: u32) -> (bool, u128, bool);

    fn neg_abs(self) -> (bool, Self::Unsigned);
    fn from_neg_abs(neg: bool, abs: Self::Unsigned) -> Self;

    #[inline]
    fn msb() -> Self {
        Self::one_shl(Self::nbits() - 1)
    }
}

macro_rules! sealed_int {
    ($Int:ident($Unsigned:ty, $is_signed:ident, $nbits:expr); $($rest:tt)*) => {
        impl SealedInt for $Int {
            type Unsigned = $Unsigned;

            #[inline]
            fn is_signed() -> bool {
                $is_signed
            }

            #[inline]
            fn nbits() -> u32 {
                $nbits
            }

            #[inline]
            fn one_shl(shift: u32) -> $Int {
                1 << shift
            }

            #[inline]
            fn all_ones_shl(shift: u32) -> $Int {
                !0 << shift
            }

            #[inline]
            fn is_zero(self) -> bool {
                self == 0
            }

            #[inline]
            fn to_fixed_neg_abs_overflow(
                self,
                frac_bits: u32,
                int_bits: u32,
            ) -> (bool, u128, bool) {
                let src_bits = <Self as SealedInt>::nbits() as i32;
                let dst_bits = (frac_bits + int_bits) as i32;

                if SealedInt::is_zero(self) {
                    return (false, 0, false);
                }

                let (neg, mut abs) = SealedInt::neg_abs(self);
                let leading_zeros = abs.leading_zeros();
                abs <<= leading_zeros;
                let need_to_shr =
                    leading_zeros as i32 - frac_bits as i32;
                let overflow = src_bits - need_to_shr > dst_bits;
                let abs = if need_to_shr == 0 {
                    u128::from(abs)
                } else if need_to_shr < 0 && -need_to_shr < 128 {
                    u128::from(abs) << -need_to_shr
                } else if need_to_shr > 0 && need_to_shr < 128 {
                    u128::from(abs) >> need_to_shr
                } else {
                    0
                };
                (neg, abs, overflow)
            }

            $($rest)*
        }
    };
    ($Int:ident($Unsigned:ty, false, $nbits:expr)) => {
        sealed_int! {
            $Int($Unsigned, false, $nbits);

            #[inline]
            fn neg_abs(self) -> (bool, Self::Unsigned) {
                (false, self)
            }

            #[inline]
            fn from_neg_abs(neg: bool, abs: Self::Unsigned) -> Self {
                debug_assert!(!neg || abs == 0);
                let _ = neg;
                abs
            }
        }
    };
    ($Int:ident($Unsigned:ty, true, $nbits:expr)) => {
        sealed_int! {
            $Int($Unsigned, true, $nbits);

            #[inline]
            fn neg_abs(self) -> (bool, Self::Unsigned) {
                if self < 0 {
                    (true, self.wrapping_neg() as $Unsigned)
                } else {
                    (false, self as $Unsigned)
                }
            }

            #[inline]
            fn from_neg_abs(neg: bool, abs: Self::Unsigned) -> Self {
                debug_assert!(abs <= Self::Unsigned::msb());
                if neg {
                    abs.wrapping_neg() as Self
                } else {
                    abs as Self
                }
            }
        }
    };
}

impl SealedInt for bool {
    type Unsigned = bool;

    #[inline]
    fn is_signed() -> bool {
        false
    }

    #[inline]
    fn nbits() -> u32 {
        1
    }

    #[inline]
    fn one_shl(shift: u32) -> bool {
        let _ = shift;
        debug_assert_eq!(shift, 0);
        true
    }

    #[inline]
    fn all_ones_shl(shift: u32) -> bool {
        let _ = shift;
        debug_assert_eq!(shift, 0);
        true
    }

    #[inline]
    fn is_zero(self) -> bool {
        !self
    }

    #[inline]
    fn to_fixed_neg_abs_overflow(self, frac_bits: u32, int_bits: u32) -> (bool, u128, bool) {
        if !self {
            return (false, 0, false);
        }
        let overflow = int_bits == 0;
        let abs = if frac_bits == 0 {
            1u128
        } else if frac_bits < 128 {
            1u128 << frac_bits
        } else {
            0
        };
        (false, abs, overflow)
    }

    #[inline]
    fn neg_abs(self) -> (bool, bool) {
        (false, self)
    }

    #[inline]
    fn from_neg_abs(neg: bool, abs: bool) -> bool {
        debug_assert!(!neg || !abs);
        let _ = neg;
        abs
    }
}

sealed_int! { i8(u8, true, 8) }
sealed_int! { i16(u16, true, 16) }
sealed_int! { i32(u32, true, 32) }
sealed_int! { i64(u64, true, 64) }
sealed_int! { i128(u128, true, 128) }
sealed_int! { u8(u8, false, 8) }
sealed_int! { u16(u16, false, 16) }
sealed_int! { u32(u32, false, 32) }
sealed_int! { u64(u64, false, 64) }
sealed_int! { u128(u128, false, 128) }