use super::{Dimension, Quantity};

// Safety: Quantity<D> is repr(transparent) over f64, which is Pod.
unsafe impl<const D: Dimension> bytemuck::Zeroable for Quantity<D> {}
unsafe impl<const D: Dimension> bytemuck::Pod for Quantity<D> {}
