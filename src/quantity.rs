#![allow(non_snake_case)]

#[cfg(any(feature = "std", feature = "alloc"))]
use core::fmt;
use core::{
    f64::math,
    ops::{Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

pub(crate) use private::QuantityMarker;
use private::True;

use crate::{
    dimension::{Dimension, checks},
    unit::{Unit, UnitAffine},
};

mod private {
    use super::*;

    pub trait QuantityMarker {
        const DIM: Dimension;
    }

    pub trait True {}
    impl True for [(); 1] {}
}

/// A physical quantity whose dimension is encoded in the const generic `D`.
///
/// The inner `f64` is always stored in SI base units. Use [`new`](Self::new)
/// or [`new_affine`](Self::new_affine) to construct from a concrete unit, and
/// [`get`](Self::get) to read back in any compatible unit.
///
/// All arithmetic operators are implemented with compile-time dimension
/// checking: addition and subtraction require identical dimensions; `*` and
/// `/` produce a new `Quantity` whose dimension is the sum or difference of
/// the operands' dimensions.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Quantity<const D: Dimension>(f64);

impl<const D: Dimension> QuantityMarker for Quantity<D> {
    const DIM: Dimension = D;
}

/// The type produced by multiplying `Q1` by `Q2` (dimensions are added).
pub type Times<Q1: QuantityMarker, Q2: QuantityMarker> = Quantity<{ Q1::DIM + Q2::DIM }>;
/// The type produced by dividing `Q1` by `Q2` (dimensions are subtracted).
pub type Per<Q1: QuantityMarker, Q2: QuantityMarker> = Quantity<{ Q1::DIM - Q2::DIM }>;
/// The type produced by raising `Base` to the integer power `E` (dimensions are
/// scaled).
pub type Raise<Base: QuantityMarker, const E: i32> = Quantity<{ Base::DIM * E }>;
/// The type produced by taking the `N`th root of `Q` (dimensions are divided by
/// `N`).
///
/// Only valid when every exponent of `Q` is divisible by `N`.
pub type Root<Q: QuantityMarker, const N: i32> = Quantity<{ Q::DIM / N }>;
/// The reciprocal of dimension `D` (equivalent to `Per<Ratio, D>`).
pub type Inverse<D> = Per<Ratio, D>;

macro_rules! make_dims {
    (@impl) => {};
    () => {};

    (@expr 1 / $a:ident) => {
        Per<Ratio, $a>
    };

    (@expr $a:ident * $b:ident) => {
        Times<$a, $b>
    };

    (@expr $a:ident / $b:ident) => {
        Per<$a, $b>
    };

    (@impl $it:ident = $a:ident, $($rest:tt)*) => {
        pub type $it = $a;
        make_dims!(@impl $($rest)*);
    };

    (@impl $it:ident = $a:tt $op:tt $b:ident, $($rest:tt)*) => {
        pub type $it = make_dims!(@expr $a $op $b);
        make_dims!(@impl $($rest)*);
    };

    (@impl $it:ident : $a:ident, $($rest:tt)*) => {
        pub type $it = Quantity<{
            Dimension {
                $a: 1,
                ..Dimension::default()
            }
        }>;
        make_dims!(@impl $($rest)*);
    };

    (@impl $it:ident, $($rest:tt)*) => {
        pub type $it = Quantity<{ Dimension::default() }>;
        make_dims!(@impl $($rest)*);
    };

    ($($all:tt)*) => {
        make_dims!(@impl $($all)*);
    }
}

make_dims! {
    Acceleration = Velocity / Time,
    AmountOfSubstance: N,
    Angle = Ratio,
    AngularAcceleration = AngularVelocity / Time,
    AngularJerk = AngularAcceleration / Time,
    AngularMomentum = AngularVelocity * MomentOfInertia,
    AngularVelocity = Angle / Time,
    Area = Length * Length,
    Charge = Current * Time,
    Current: I,
    Density = Mass / Volume,
    Energy = Force * Length,
    Force = Mass * Acceleration,
    Frequency = 1 / Time,
    Jerk = Acceleration / Time,
    Length: L,
    LuminousIntensity: J,
    Mass: M,
    MomentOfInertia = Mass * Area,
    Momentum = Mass * Velocity,
    Power = Voltage * Current,
    Pressure = Force / Area,
    Ratio,
    Resistance = Voltage / Current,
    Time: T,
    Temperature: Th,
    Torque = Force * Length,
    Velocity = Length / Time,
    Voltage = Energy / Current,
    Volume = Area * Length,
}

impl<const D: Dimension> Quantity<D> {
    /// Constructs a quantity from `value` expressed in the linear unit `U`.
    ///
    /// The value is converted to SI immediately: `SI = U::SLOPE * value`.
    /// Fails to compile if `U::DIM != D`.
    ///
    /// Use [`new_affine`](Self::new_affine) for units with a non-zero intercept
    /// (e.g. Celsius, Fahrenheit).
    pub const fn new<U: Unit>(value: f64) -> Self
    where
        [(); { checks::eq(D, U::DIM) } as usize]: True,
    {
        Self(value * U::SLOPE)
    }

    /// Constructs a quantity from `value` expressed in the affine unit `U`.
    ///
    /// The value is converted to SI immediately: `SI = U::SLOPE * value +
    /// U::INTERCEPT`. Fails to compile if `U::DIM != D` or if `U` has a
    /// zero intercept (use [`new`](Self::new) for linear units).
    pub const fn new_affine<U: UnitAffine>(value: f64) -> Self
    where
        [(); { checks::eq(D, U::DIM) } as usize]: True,
        [(); { U::INTERCEPT != 0.0 } as usize]: True,
    {
        Self(value * U::SLOPE + U::INTERCEPT)
    }

    /// Constructs a quantity directly from an SI value, bypassing unit
    /// conversion. Prefer [`new`](Self::new) at call sites where a unit is
    /// known.
    pub const fn new_si(value: f64) -> Self {
        Self(value)
    }

    /// Returns the quantity expressed in unit `U`.
    ///
    /// Applies the inverse conversion: `unit_value = (SI - U::INTERCEPT) /
    /// U::SLOPE`. Works for both linear and affine units. Fails to compile
    /// if `U::DIM != D`.
    pub const fn get<U: UnitAffine>(self) -> f64
    where
        [(); { checks::eq(D, U::DIM) } as usize]: True,
    {
        (self.0 - U::INTERCEPT) / U::SLOPE
    }

    /// Returns the raw SI value stored inside this quantity.
    pub const fn si(self) -> f64 {
        self.0
    }

    /// Raises the quantity to the integer power `E`, producing a
    /// [`Raise<Self, E>`] whose dimension is `D * E`.
    pub fn powi<const E: i32>(self) -> Raise<Self, E> {
        Quantity(math::powi(self.0, E))
    }

    /// Returns the square root of this quantity.
    ///
    /// The resulting dimension is `D / 2`. Fails to compile if any exponent
    /// of `D` is odd.
    pub fn sqrt(self) -> Root<Self, 2>
    where
        [(); { checks::can_divide(D, 2) } as usize]: True,
    {
        Quantity(math::sqrt(self.0))
    }

    /// Returns the `N`th root of this quantity.
    ///
    /// The resulting dimension is `D / N`. Fails to compile if any exponent
    /// of `D` is not divisible by `N`.
    #[cfg(any(feature = "std", feature = "libm"))]
    pub fn nroot<const N: i32>(self) -> Root<Self, N>
    where
        [(); { checks::can_divide(D, N) } as usize]: True,
    {
        #[cfg(feature = "std")]
        return Quantity(self.0.powf(1.0 / N as f64));
        #[cfg(all(not(feature = "std"), feature = "libm"))]
        return Quantity(libm::pow(self.0, 1.0 / N as f64));
    }

    /// Computes `sqrt(self^2 + other^2)`, the Euclidean hypotenuse.
    ///
    /// Both arguments must have the same dimension; the result has the same
    /// dimension as the inputs.
    ///
    /// Falls back to `powi` and `sqrt` if neither `std` nor `libm` is
    /// available, which may be less efficient.
    pub fn hypot(self, other: Self) -> Self {
        #[cfg(feature = "std")]
        return Self(self.0.hypot(other.0));
        #[cfg(all(not(feature = "std"), feature = "libm"))]
        return Self(libm::hypot(self.0, other.0));
        #[cfg(not(any(feature = "std", feature = "libm")))]
        return Self(math::sqrt(math::powi(self.0, 2) + math::powi(other.0, 2)));
    }

    /// Computes the four-quadrant arctangent of `self / other`
    /// (`self.atan2(other)`), returning an [`Angle`] in radians.
    ///
    /// Both arguments must have the same dimension (they cancel in the ratio).
    #[cfg(any(feature = "std", feature = "libm"))]
    pub fn atan2(self, other: Self) -> Angle {
        #[cfg(feature = "std")]
        return Angle::new_si(self.0.atan2(other.0));
        #[cfg(all(not(feature = "std"), feature = "libm"))]
        return Angle::new_si(libm::atan2(self.0, other.0));
    }
}

/// Adds two quantities. Fails to compile if their dimensions differ.
impl<const D1: Dimension, const D2: Dimension> const Add<Quantity<D2>> for Quantity<D1>
where
    [(); { checks::eq(D1, D2) } as usize]: True,
{
    type Output = Self;

    fn add(self, rhs: Quantity<D2>) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

/// Adds another quantity to this one in-place. Fails to compile if their
/// dimensions differ.
impl<const D1: Dimension, const D2: Dimension> const AddAssign<Quantity<D2>> for Quantity<D1>
where
    [(); { checks::eq(D1, D2) } as usize]: True,
{
    fn add_assign(&mut self, rhs: Quantity<D2>) {
        self.0 += rhs.0;
    }
}

/// Subtracts two quantities. Fails to compile if their dimensions differ.
impl<const D1: Dimension, const D2: Dimension> const Sub<Quantity<D2>> for Quantity<D1>
where
    [(); { checks::eq(D1, D2) } as usize]: True,
{
    type Output = Self;

    fn sub(self, rhs: Quantity<D2>) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

/// Subtracts another quantity from this one in-place. Fails to compile if their
/// dimensions differ.
impl<const D1: Dimension, const D2: Dimension> const SubAssign<Quantity<D2>> for Quantity<D1>
where
    [(); { checks::eq(D1, D2) } as usize]: True,
{
    fn sub_assign(&mut self, rhs: Quantity<D2>) {
        self.0 -= rhs.0;
    }
}

/// Multiplies two quantities. The output dimension is `D1 + D2`.
impl<const D1: Dimension, const D2: Dimension> const Mul<Quantity<D2>> for Quantity<D1>
where
    Quantity<{ D1 + D2 }>:,
{
    type Output = Quantity<{ D1 + D2 }>;

    fn mul(self, rhs: Quantity<D2>) -> Self::Output {
        Quantity(self.0 * rhs.0)
    }
}

/// Scales a quantity by a dimensionless scalar. The dimension is unchanged.
impl<const D: Dimension> const Mul<f64> for Quantity<D> {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Quantity(self.0 * rhs)
    }
}

/// Scales a quantity by a dimensionless scalar (scalar on the left).
impl<const D: Dimension> const Mul<Quantity<D>> for f64 {
    type Output = Quantity<D>;

    fn mul(self, rhs: Quantity<D>) -> Self::Output {
        Quantity(self * rhs.0)
    }
}

/// Scales a quantity by a scalar in-place. The dimension is unchanged.
impl<const D: Dimension> const MulAssign<f64> for Quantity<D> {
    fn mul_assign(&mut self, rhs: f64) {
        self.0 *= rhs;
    }
}

/// Divides two quantities. The output dimension is `D1 - D2`.
impl<const D1: Dimension, const D2: Dimension> const Div<Quantity<D2>> for Quantity<D1>
where
    Quantity<{ D1 - D2 }>:,
{
    type Output = Quantity<{ D1 - D2 }>;

    fn div(self, rhs: Quantity<D2>) -> Self::Output {
        Quantity(self.0 / rhs.0)
    }
}

/// Divides a quantity by a dimensionless scalar. The dimension is unchanged.
impl<const D: Dimension> const Div<f64> for Quantity<D> {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Quantity(self.0 / rhs)
    }
}

/// Divides a scalar by a quantity, producing the reciprocal dimension `-D`.
impl<const D: Dimension> const Div<Quantity<D>> for f64
where
    Quantity<{ -D }>:,
{
    type Output = Quantity<{ -D }>;

    fn div(self, rhs: Quantity<D>) -> Self::Output {
        Quantity(self / rhs.0)
    }
}

/// Divides a quantity by a scalar in-place. The dimension is unchanged.
impl<const D: Dimension> const DivAssign<f64> for Quantity<D> {
    fn div_assign(&mut self, rhs: f64) {
        self.0 /= rhs;
    }
}

/// Negates the value; the dimension is unchanged.
impl<const D: Dimension> const Neg for Quantity<D> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

/// Converts a bare `f64` into a dimensionless [`Ratio`]. Only available when
/// `D` is the zero dimension.
impl const From<f64> for Ratio {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

/// Extracts the inner `f64` from a dimensionless [`Ratio`]. Only available
/// when `D` is the zero dimension.
impl const From<Ratio> for f64 {
    fn from(value: Ratio) -> Self {
        value.0
    }
}

/// Allows treating a dimensionless [`Ratio`] as a plain `f64` reference.
/// Only available when `D` is the zero dimension.
impl const Deref for Ratio {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Allows mutably treating a dimensionless [`Ratio`] as a plain `f64`.
/// Only available when `D` is the zero dimension.
impl const DerefMut for Ratio {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Compares the underlying SI values for equality.
impl<const D: Dimension> const PartialEq for Quantity<D> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// Orders quantities by their underlying SI values.
impl<const D: Dimension> const PartialOrd for Quantity<D> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// Formats the quantity as `"<value> <unit>"` where the unit string is derived
/// from the dimension exponents using SI base unit symbols.
///
/// - Exponent 1 -> bare symbol (e.g. `"m"`).
/// - Any other non-zero exponent -> symbol with superscript (e.g. `"s^-2"`).
/// - Zero exponent -> symbol omitted.
/// - Symbols are joined with `·` (cdot).
/// - Dimensionless ([`Ratio`]) -> only the numeric value is printed.
///
/// # Examples
///
/// ```ignore
/// println!("{}", some_velocity); // "12.5 m·s^-1"
/// println!("{}", some_ratio); // "0.75"
/// ```
#[cfg(any(feature = "std", feature = "alloc"))]
impl<const D: Dimension> fmt::Display for Quantity<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        use alloc::string::ToString;

        let unit_str = D.to_string();

        if unit_str.is_empty() {
            write!(f, "{}", self.0)
        } else {
            write!(f, "{} {}", self.0, unit_str)
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<const D: Dimension> fmt::Debug for Quantity<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Quantity<{:?}>({})", D, self.0)
    }
}

macro_rules! lib_impl {
    (@inner $lib:ident) => {
        pastey::paste! {
            #[cfg(feature = "" $lib "")]
            mod $lib;
        }
    };

    (@inner $feat:literal => $lib:ident) => {
        #[cfg(feature = $feat)] mod $lib;
    };

    ($( $lib:ident $(from $feat:literal)? ),*) => {
        $(lib_impl!(@inner $($feat =>)? $lib);)*
    };
}

lib_impl! {
    approx,
    bytemuck,
    nalgebra,
    rand,
    num_traits from "num-traits",
    serde
}
