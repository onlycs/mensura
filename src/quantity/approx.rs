use approx::{AbsDiffEq, RelativeEq};

use super::{Dimension, Quantity};

/// Absolute-difference equality for quantities, delegating to the
/// underlying `f64`.
///
/// The epsilon is itself a `Quantity` of the same dimension so that the
/// tolerance is always dimensionally consistent with the values being
/// compared.
impl<const D: Dimension> AbsDiffEq for Quantity<D> {
    type Epsilon = Self;

    /// Returns a default epsilon of [`f64::EPSILON`] in the same dimension.
    fn default_epsilon() -> Self::Epsilon {
        Self(f64::EPSILON)
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.0.abs_diff_eq(&other.0, epsilon.0)
    }
}

/// Relative equality for quantities, delegating to the underlying `f64`.
///
/// Both the epsilon and the max-relative tolerance are expressed as
/// quantities of the same dimension, ensuring comparisons remain
/// dimensionally consistent.
impl<const D: Dimension> RelativeEq for Quantity<D> {
    /// Returns a default max-relative tolerance of [`f64::EPSILON`] in the
    /// same dimension.
    fn default_max_relative() -> Self::Epsilon {
        Self(f64::EPSILON)
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.0.relative_eq(&other.0, epsilon.0, max_relative.0)
    }
}
