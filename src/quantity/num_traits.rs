use core::ops::Add;

use num_traits::{Bounded, One, Zero};

use super::{Dimension, Quantity, Ratio};

impl<const D: Dimension> Zero for Quantity<D>
where
    Quantity<D>: Add<Quantity<D>, Output = Quantity<D>>,
{
    fn zero() -> Self {
        Self(0.0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}

/// `One` is only meaningful for dimensionless [`Ratio`]; multiplying a
/// non-zero dimension by itself changes the dimension, so no multiplicative
/// identity exists there.
impl One for Ratio {
    fn one() -> Self {
        Self(1.0)
    }
}

impl<const D: Dimension> Bounded for Quantity<D> {
    fn min_value() -> Self {
        Self(f64::MIN)
    }

    fn max_value() -> Self {
        Self(f64::MAX)
    }
}
