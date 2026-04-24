use rand::distr::{Distribution, StandardUniform};

use super::{Dimension, Quantity};

/// Samples a quantity whose SI value is drawn from `StandardUniform` (i.e.
/// uniformly in `[0, 1)`).
impl<const D: Dimension> Distribution<Quantity<D>> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Quantity<D> {
        use rand::RngExt;
        Quantity(rng.random::<f64>())
    }
}
