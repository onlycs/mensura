#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("the `nalgebra` feature requires `std` or `alloc`");

use nalgebra::SimdValue;

use super::{Dimension, Quantity};

/// `Quantity<D>` as a 1-lane scalar SIMD value.
impl<const D: Dimension> SimdValue for Quantity<D> {
    type Element = f64;
    type SimdBool = bool;

    const LANES: usize = 1;

    #[inline(always)]
    fn splat(val: Self::Element) -> Self {
        Self(val)
    }

    #[inline(always)]
    fn extract(&self, _i: usize) -> Self::Element {
        self.0
    }

    #[inline(always)]
    unsafe fn extract_unchecked(&self, _i: usize) -> Self::Element {
        self.0
    }

    #[inline(always)]
    fn replace(&mut self, _i: usize, val: Self::Element) {
        *self = Self(val);
    }

    #[inline(always)]
    unsafe fn replace_unchecked(&mut self, _i: usize, val: Self::Element) {
        *self = Self(val);
    }

    #[inline(always)]
    fn select(self, cond: Self::SimdBool, other: Self) -> Self {
        if cond { self } else { other }
    }
}
