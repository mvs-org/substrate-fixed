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
    types::extra::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8},
    FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32, FixedU64,
    FixedU8,
};
use az::{Cast, CheckedCast, OverflowingCast, SaturatingCast, StaticCast, WrappingCast};
use core::mem;
#[cfg(feature = "f16")]
use half::f16;

macro_rules! run_time {
    ($Fixed:ident($LeEqU:ident); $Num:ident) => {
        impl<Frac: $LeEqU> Cast<$Fixed<Frac>> for $Num {
            #[inline]
            fn cast(self) -> $Fixed<Frac> {
                <$Fixed<Frac>>::from_num(self)
            }
        }

        impl<Frac: $LeEqU> Cast<$Num> for $Fixed<Frac> {
            #[inline]
            fn cast(self) -> $Num {
                self.to_num()
            }
        }

        impl<Frac: $LeEqU> CheckedCast<$Fixed<Frac>> for $Num {
            #[inline]
            fn checked_cast(self) -> Option<$Fixed<Frac>> {
                <$Fixed<Frac>>::checked_from_num(self)
            }
        }

        impl<Frac: $LeEqU> CheckedCast<$Num> for $Fixed<Frac> {
            #[inline]
            fn checked_cast(self) -> Option<$Num> {
                self.checked_to_num()
            }
        }

        impl<Frac: $LeEqU> SaturatingCast<$Fixed<Frac>> for $Num {
            #[inline]
            fn saturating_cast(self) -> $Fixed<Frac> {
                <$Fixed<Frac>>::saturating_from_num(self)
            }
        }

        impl<Frac: $LeEqU> SaturatingCast<$Num> for $Fixed<Frac> {
            #[inline]
            fn saturating_cast(self) -> $Num {
                self.saturating_to_num()
            }
        }

        impl<Frac: $LeEqU> WrappingCast<$Fixed<Frac>> for $Num {
            #[inline]
            fn wrapping_cast(self) -> $Fixed<Frac> {
                <$Fixed<Frac>>::wrapping_from_num(self)
            }
        }

        impl<Frac: $LeEqU> WrappingCast<$Num> for $Fixed<Frac> {
            #[inline]
            fn wrapping_cast(self) -> $Num {
                self.wrapping_to_num()
            }
        }

        impl<Frac: $LeEqU> OverflowingCast<$Fixed<Frac>> for $Num {
            #[inline]
            fn overflowing_cast(self) -> ($Fixed<Frac>, bool) {
                <$Fixed<Frac>>::overflowing_from_num(self)
            }
        }

        impl<Frac: $LeEqU> OverflowingCast<$Num> for $Fixed<Frac> {
            #[inline]
            fn overflowing_cast(self) -> ($Num, bool) {
                self.overflowing_to_num()
            }
        }
    };
}

macro_rules! compile_time {
    (impl<$Frac:ident: $LeEqU:ident> StaticCast<$Dst:ty> for $Src:ty { $cond:expr }) => {
        impl<$Frac: $LeEqU> StaticCast<$Dst> for $Src {
            #[inline]
            fn static_cast(self) -> Option<$Dst> {
                if $cond {
                    Some(az::cast(self))
                } else {
                    None
                }
            }
        }
    };

    ($FixedI:ident, $FixedU:ident($LeEqU:ident); int $IntI:ident, $IntU:ident) => {
        compile_time! {
            impl<Frac: $LeEqU> StaticCast<$FixedI<Frac>> for $IntI {
                $FixedI::<Frac>::INT_NBITS >= 8 * mem::size_of::<$IntI>() as u32
            }
        }

        compile_time! {
            impl<Frac: $LeEqU> StaticCast<$IntI> for $FixedI<Frac> {
                8 * mem::size_of::<$IntI>() as u32 >= $FixedI::<Frac>::INT_NBITS
            }
        }

        compile_time! {
            impl<Frac: $LeEqU> StaticCast<$FixedU<Frac>> for $IntI {
        false
            }
        }

        compile_time! {
            impl<Frac: $LeEqU> StaticCast<$IntI> for $FixedU<Frac> {
                8 * mem::size_of::<$IntI>() as u32 > $FixedU::<Frac>::INT_NBITS
            }
        }

        compile_time! {
            impl<Frac: $LeEqU> StaticCast<$FixedI<Frac>> for $IntU {
                $FixedI::<Frac>::INT_NBITS > 8 * mem::size_of::<$IntU>() as u32
            }
        }

        compile_time! {
            impl<Frac: $LeEqU> StaticCast<$IntU> for $FixedI<Frac> {
        false
            }
        }

        compile_time! {
            impl<Frac: $LeEqU> StaticCast<$FixedU<Frac>> for $IntU {
                $FixedU::<Frac>::INT_NBITS >= 8 * mem::size_of::<$IntU>() as u32
            }
        }

        compile_time! {
            impl<Frac: $LeEqU> StaticCast<$IntU> for $FixedU<Frac> {
                8 * mem::size_of::<$IntU>() as u32 >= $FixedU::<Frac>::INT_NBITS
            }
        }
    };

    ($Fixed:ident($LeEqU:ident); float $Float:ident) => {
        compile_time! {
            impl<Frac: $LeEqU> StaticCast<$Fixed<Frac>> for $Float {
                false
            }
        }

        compile_time! {
            impl<Frac: $LeEqU> StaticCast<$Float> for $Fixed<Frac> {
        true
            }
        }
    };
}

macro_rules! cross_num {
    ($Fixed:ident($LeEqU:ident); $($Num:ident,)*) => { $(
	run_time! { $Fixed($LeEqU); $Num }
    )* };
    ($($Fixed:ident($LeEqU:ident),)*) => { $(
	cross_num! {
	    $Fixed($LeEqU);
	    i8, i16, i32, i64, i128, isize,
	    u8, u16, u32, u64, u128, usize,
	    f32, f64,
	}
	#[cfg(feature = "f16")]
	cross_num! {
	    $Fixed($LeEqU);
	    f16,
	}
    )* };
}

cross_num! {
    FixedI8(LeEqU8), FixedI16(LeEqU16), FixedI32(LeEqU32), FixedI64(LeEqU64), FixedI128(LeEqU128),
    FixedU8(LeEqU8), FixedU16(LeEqU16), FixedU32(LeEqU32), FixedU64(LeEqU64), FixedU128(LeEqU128),
}

macro_rules! cross_int {
    ($FixedI:ident, $FixedU:ident($LeEqU:ident); $(($IntI:ident, $IntU:ident),)*) => { $(
	compile_time! { $FixedI, $FixedU($LeEqU); int $IntI, $IntU }
    )* };
    ($($FixedI:ident, $FixedU:ident($LeEqU:ident),)*) => { $(
	cross_int! {
	    $FixedI, $FixedU($LeEqU);
	    (i8, u8),
	    (i16, u16),
	    (i32, u32),
	    (i64, u64),
	    (i128, u128),
	    (isize, usize),
	}
    )* };
}

cross_int! {
    FixedI8, FixedU8(LeEqU8),
    FixedI16, FixedU16(LeEqU16),
    FixedI32, FixedU32(LeEqU32),
    FixedI64, FixedU64(LeEqU64),
    FixedI128, FixedU128(LeEqU128),
}

macro_rules! cross_float {
    ($Fixed:ident($LeEqU:ident); $($Float:ident,)*) => { $(
	compile_time! { $Fixed($LeEqU); float $Float }
    )* };
    ($($Fixed:ident($LeEqU:ident),)*) => { $(
	cross_float! {
	    $Fixed($LeEqU);
	    f32, f64,
	}
	#[cfg(feature = "f16")]
	cross_float! {
	    $Fixed($LeEqU);
	    f16,
	}
    )* };
}

cross_float! {
    FixedI8(LeEqU8), FixedI16(LeEqU16), FixedI32(LeEqU32), FixedI64(LeEqU64), FixedI128(LeEqU128),
    FixedU8(LeEqU8), FixedU16(LeEqU16), FixedU32(LeEqU32), FixedU64(LeEqU64), FixedU128(LeEqU128),
}
