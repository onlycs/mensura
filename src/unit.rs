//! Unit conversion structs for every physical quantity defined in
//! [`crate::quantity`].
//!
//! # Overview
//!
//! Each physical quantity has its own sub-module (e.g. [`length`],
//! [`velocity`], [`temperature`]). Inside each sub-module you will find:
//!
//! - One *base unit* struct whose [`Unit::SLOPE`] is `1.0` - values are always
//!   stored internally in this unit.
//! - Zero or more *non-base unit* structs with the appropriate slope and
//!   intercept for their affine conversion to the base unit.
//!
//! Unit structs are zero-size types; they exist only to carry conversion
//! constants as associated `const`s. Pass them as the type parameter `U` to
//! [`Quantity::new`](crate::quantity::Quantity::new) and
//! [`Quantity::get`](crate::quantity::Quantity::get):
//!
//! ```
//! use mensura::{
//!     quantity::{Length, Temperature},
//!     unit::{length, temperature},
//! };
//!
//! // Construct a length from kilometres - stored as metres internally.
//! let d = Length::new::<length::kilometer>(42.0);
//! assert_eq!(d.get::<length::meter>(), 42_000.0);
//! assert_eq!(d.get::<length::kilometer>(), 42.0);
//!
//! // Affine units work the same way. However, `new_affine` is used as a
//! // constructor instead of `new` to indicate that the unit has a non-zero intercept.
//! let t = Temperature::new_affine::<temperature::celsius>(100.0);
//! assert_eq!(t.get::<temperature::kelvin>(), 373.15);
//! ```
//!
//! # Affine units (temperature)
//!
//! Most units only need a slope (`SI = SLOPE * value`). Temperature scales
//! (Celsius, Fahrenheit) also need an intercept:
//!
//! ```text
//! SI_value   = SLOPE * unit_value + INTERCEPT
//! unit_value = (SI_value - INTERCEPT) / SLOPE
//! ```
//!
//! `Quantity::new` and `Quantity::get` apply this formula automatically, so
//! callers never need to touch [`Unit::SLOPE`] or [`UnitAffine::INTERCEPT`]
//! directly.

#![allow(non_camel_case_types)]

use core::f64;

use crate::dimension::Dimension;

/// Describes the linear relationship between a concrete unit and the SI base
/// unit for a given physical quantity.
///
/// The conversion from a value expressed *in this unit* to the internal SI
/// representation is:
///
/// ```text
/// SI_value = SLOPE * unit_value
/// ```
pub trait Unit {
    const DIM: Dimension;

    /// Multiplicative factor to convert from this unit to the SI base unit.
    ///
    /// `SI_value = SLOPE * unit_value`
    ///
    /// For the base unit itself this is always `1.0`.
    const SLOPE: f64;
}

/// Describes the affine relationship between a concrete unit and the SI base
/// unit for a given physical quantity. This is used for temperature scales
/// like Celsius and Fahrenheit, which have a non-zero intercept with the SI
/// base unit (kelvin).
///
/// The conversion from a value expressed *in this unit* to the internal SI
/// representation is:
///
/// ```text
/// SI_value   = SLOPE * unit_value + INTERCEPT
/// unit_value = (SI_value - INTERCEPT) / SLOPE
/// ```
pub trait UnitAffine {
    const DIM: Dimension;

    /// Multiplicative factor to convert from this unit to the SI base unit.
    ///
    /// For the base unit itself this is always `1.0`.
    const SLOPE: f64;

    /// Additive factor to convert from this unit to the SI base unit.
    ///
    /// For the base unit itself this is always `0.0`.
    const INTERCEPT: f64;
}

/// Blanket implementation for linear units: any `Unit` is also a `UnitAffine`
/// with zero intercept.
impl<U: Unit> UnitAffine for U {
    const DIM: Dimension = U::DIM;
    const INTERCEPT: f64 = 0.0;
    const SLOPE: f64 = U::SLOPE;
}

macro_rules! impl_unit {
    (@test $unit:ident) => {
        pastey::paste! {
            #[allow(unused)]
            #[cfg(test)]
            mod [<$unit:snake _tests>] {
                use super::*;

                #[test]
                fn conversion() {
                    let unit = rand::random::<f64>() * 1e6 - 5e5;
                    let si = <$unit as Unit>::SLOPE * unit;

                    let qty = Quantity::new::<$unit>(unit);
                    let qty_si = qty.si();
                    let qty_unit = qty.get::<$unit>();

                    let diff_si = (qty_si - si).abs();
                    let diff_unit = (qty_unit - unit).abs();

                    assert!(diff_si < 1e-4, "SI conversion error too large: {diff_si}");
                    assert!(diff_unit < 1e-4, "Unit conversion error too large: {diff_unit}");
                }

                #[test]
                fn math() {
                    let a_value = rand::random::<f64>() * 1e6 - 5e5;
                    let b_value = rand::random::<f64>() * 1e6 - 5e5;
                    let a = Quantity::new::<$unit>(a_value);
                    let b = Quantity::new::<$unit>(b_value);

                    let c = a + b;
                    let d = a - b;
                    let e = a * b;
                    let f = a / b;

                    let c_base = a_value + b_value;
                    let d_base = a_value - b_value;
                    let e_base = a_value * b_value;
                    let f_base = a_value / b_value;

                    let c_diff = (c.get::<$unit>() - c_base).abs();
                    let d_diff = (d.get::<$unit>() - d_base).abs();
                    let e_diff = (e.get::<raise<$unit, 2>>() - e_base).abs();
                    let f_diff = (f.get::<one>() - f_base).abs();

                    assert!(c_diff < 1e-4, "Addition conversion error too large: {c_diff}");
                    assert!(d_diff < 1e-4, "Subtraction conversion error too large: {d_diff}");
                    assert!(e_diff < 1e-4, "Multiplication conversion error too large: {e_diff}");
                    assert!(f_diff < 1e-4, "Division conversion error too large: {f_diff}");
                }
            }
        }
    };

    (@next) => {};

    (@next @$unit:ident $($alias:ident)* : $slope:expr; $intercept:expr, $($remainder:tt)*) => {
        pub struct $unit;
        $(pub type $alias = $unit;)*

        impl UnitAffine for $unit {
            const DIM: Dimension = Quantity::DIM;
            const SLOPE: f64 = $slope;
            const INTERCEPT: f64 = $intercept;
        }

        pastey::paste! {
            #[allow(unused)]
            #[cfg(test)]
            mod [<$unit:snake _tests>] {
                use super::*;

                #[test]
                fn conversion() {
                    let unit = rand::random::<f64>() * 1e6 - 5e5;
                    let si = <$unit as UnitAffine>::SLOPE * unit + <$unit as UnitAffine>::INTERCEPT;

                    let qty = Quantity::new_affine::<$unit>(unit);
                    let qty_si = qty.si();
                    let qty_unit = qty.get::<$unit>();

                    let diff_si = (qty_si - si).abs();
                    let diff_unit = (qty_unit - unit).abs();

                    assert!(diff_si < 1e-4, "SI conversion error too large: {diff_si}");
                    assert!(diff_unit < 1e-4, "Unit conversion error too large: {diff_unit}");
                }
            }
        }

        impl_unit!(@next $($remainder)*);
    };

    (@next @$unit:ident $($alias:ident)* : $slope:expr, $($remainder:tt)*) => {
        pub struct $unit;
        $(pub type $alias = $unit;)*

        impl Unit for $unit {
            const DIM: Dimension = Quantity::DIM;
            const SLOPE: f64 = $slope;
        }

        impl_unit!(@test $unit);
        impl_unit!(@next $($remainder)*);
    };

    (@next @$unit:ident $($alias:ident)* = $slope:ty, $($remainder:tt)*) => {
        pub type $unit = $slope;
        $(pub type $alias = $unit;)*

        impl_unit!(@test $unit);
        impl_unit!(@next $($remainder)*);
    };

    // Syntax per block:
    //
    //   QuantityName {
    //       base <base_unit_name> [alias...],
    //       @<non_base_unit> [alias...]: <slope> [; <intercept>],
    //       ...
    //   }
    //
    // Each block generates a snake_case sub-module named after the quantity.
    // `Quantity` inside that module resolves to `crate::quantity::<QuantityName>`.
    ($(
        $quantity:ident {
            base $base:ident $($base_alias:ident)*,
            $($definition:tt)*
        }
    );* $(;)?) => {
        $(
            pastey::paste! {
                #[allow(unused_imports)]
                pub mod [<$quantity:snake>] {
                    use super::*;
                    use crate::quantity::{$quantity as Quantity, QuantityMarker};

                    /// SI base (or coherent derived) unit for this quantity.
                    /// [`Unit::SLOPE`] = 1.0
                    pub struct $base;
                    $(
                        /// Alias for the base unit struct.
                        pub type $base_alias = $base;
                    )*

                    impl Unit for $base {
                        const DIM: Dimension = Quantity::DIM;
                        const SLOPE: f64 = 1.0;
                    }

                    impl_unit!(@next $($definition)*);
                }

                pub use [<$quantity:snake>]::*;
            }
        )*
    };
}

impl_unit! {
    Acceleration {
        base meter_per_second_squared,
        @kilometer_per_hour_squared = per<kilometer_per_hour, hour>,
        @foot_per_second_squared = per<foot_per_second, second>,
        @g: 9.80665,
    };
    AmountOfSubstance {
        base mole,
    };
    Angle {
        base radian rad,
        @degree deg: f64::consts::PI / 180.,
        @revolution rev: 2. * f64::consts::PI,
    };
    AngularAcceleration {
        base radian_per_second_squared,
        @degree_per_second_squared = per<degree_per_second, second>,
        @revolution_per_second_squared = per<revolution_per_second, second>,
        @revolution_per_minute_squared = per<revolution_per_minute, minute>,
    };
    AngularJerk {
        base radian_per_second_cubed,
        @degree_per_second_cubed = per<degree_per_second_squared, second>,
        @revolution_per_second_cubed = per<revolution_per_second_squared, second>,
        @revolution_per_minute_cubed = per<revolution_per_minute_squared, minute>,
    };
    AngularMomentum {
        base kilogram_meter_squared_per_second kgm2_per_s,
        @gram_centimeter_squared_per_second = per<times<gram, raise<centimeter, 2>>, second>,
    };
    AngularVelocity {
        base radian_per_second,
        @degree_per_second = per<degree, second>,
        @revolution_per_second rps = per<revolution, second>,
        @revolution_per_minute rpm = per<revolution, minute>,
    };
    Charge {
        base coulomb,
    };
    Current {
        base ampere amp,
    };
    Density {
        base kilogram_per_cubic_meter,
        @gram_per_cubic_centimeter = per<gram, raise<centimeter, 3>>,
    };
    Energy {
        base joule,
        @calorie cal: 4.184,
    };
    Force {
        base newton,
        @pound_force lbf: 4.4482216152605,
    };
    Frequency {
        base hertz Hz,
        @kilohertz kHz: 1e3,
        @megahertz MHz: 1e6,
        @gigahertz GHz: 1e9,
    };
    Length {
        base meter,
        @kilometer: 1e3,
        @centimeter: 1e-2,
        @millimeter: 1e-3,
        @micrometer micron: 1e-6,
        @nanometer: 1e-9,
        @foot: 0.3048,
        @inch: 0.0254,
    };
    LuminousIntensity {
        base candela,
    };
    Mass {
        base kilogram,
        @gram: 1e-3,
        @milligram mg: 1e-6,
        @microgram: 1e-9,
        @tonne: 1e3,
        @pound: 0.45359237,
    };
    Power {
        base watt,
        @kilowatt: 1e3,
        @megawatt: 1e6,
        @horsepower hp: 745.69987158227022,
    };
    Pressure {
        base pascal,
        @kilopascal: 1e3,
        @megapascal: 1e6,
        @bar: 1e5,
        @psi: 6894.75729,
    };
    Ratio {
        base one,
    };
    Temperature {
        base kelvin,
        @celsius: 1.0; 273.15,
        @fahrenheit: 5./9.; 45967./180.,
    };
    Time {
        base second,
        @minute: 60.,
        @hour: 3600.,
        @day: 86400.,
        @millisecond ms: 1e-3,
        @microsecond: 1e-6,
        @nanosecond: 1e-9,
    };
    Torque {
        base newton_meter Nm,
        @pound_force_foot lbf_foot = times<lbf, foot>,
    };
    Velocity {
        base meter_per_second,
        @kilometer_per_hour = per<kilometer, hour>,
        @foot_per_second = per<foot, second>,
    };
    Voltage {
        base volt,
    };
    Volume {
        base cubic_meter,
        @liter: 1e-3,
        @milliliter cubic_centimeter: 1e-6,
        @cubic_foot = raise<foot, 3>,
        @cubic_inch = raise<inch, 3>,
    };
}

/// A unit combinator representing `A` divided by `B`.
///
/// Computes the conversion slope and intercept for a ratio unit at compile
/// time, so you can express compound units without defining a new struct:
///
/// ```
/// use mensura::{
///     quantity::Velocity,
///     unit::{length, per, time},
/// };
///
/// // metres per minute -without needing a named unit struct
/// let speed = Velocity::new::<per<length::meter, time::minute>>(100.0);
/// ```
///
/// The combined slope and intercept are derived from those of `A` and `B`:
///
/// ```text
/// SLOPE     = A::SLOPE / B::SLOPE
/// ```
pub struct per<A: Unit, B: Unit>(core::marker::PhantomData<(A, B)>);

impl<A: Unit, B: Unit> Unit for per<A, B> {
    const DIM: Dimension = A::DIM - B::DIM;
    const SLOPE: f64 = A::SLOPE / B::SLOPE;
}

/// A unit combinator representing `A` multiplied by `B`.
///
/// Computes the conversion slope and intercept for a product unit at compile
/// time, so you can express compound units without defining a new struct:
///
/// ```
/// use mensura::{
///     quantity::Torque,
///     unit::{force, length, times},
/// };
///
/// // newton-millimetres -without needing a named unit struct
/// let torque = Torque::new::<times<force::newton, length::millimeter>>(500.0);
/// ```
pub struct times<A: Unit, B: Unit>(core::marker::PhantomData<(A, B)>);

impl<A: Unit, B: Unit> Unit for times<A, B> {
    const DIM: Dimension = A::DIM + B::DIM;
    const SLOPE: f64 = A::SLOPE * B::SLOPE;
}

/// A unit combinator representing the reciprocal of `U`.
///
/// Computes the conversion slope and intercept for the inverse of a unit at
/// compile time, so you can express reciprocal units without defining a new
/// struct:
///
/// ```
/// use mensura::{
///     quantity::Frequency,
///     unit::{inverse, time},
/// };
///
/// // 1 / minute (reciprocal minutes) -without needing a named unit struct
/// let rate = Frequency::new::<inverse<time::minute>>(2.0); // 2 per minute
/// ```
pub type inverse<U> = per<crate::unit::ratio::one, U>;

/// A unit combinator representing `U` raised to the integer power `E`.
///
/// Computes the conversion slope for a power unit at compile time, so you can
/// express squared or cubed units without defining a new struct:
///
/// ```
/// use mensura::{
///     quantity::Volume,
///     unit::{length, raise},
/// };
///
/// // cubic feet -without needing a named unit struct
/// let vol = Volume::new::<raise<length::foot, 3>>(27.0); // 27 ft^3
/// ```
///
/// The slope is `U::SLOPE ^ E`, computed recursively at compile time.
pub struct raise<U: Unit, const E: i32>(core::marker::PhantomData<U>);

impl<U: Unit, const E: i32> Unit for raise<U, E> {
    const DIM: Dimension = U::DIM * E;
    const SLOPE: f64 = {
        const fn _const_powi(base: f64, exp: i32) -> f64 {
            if exp == 0 {
                1.0
            } else if exp < 0 {
                1.0 / _const_powi(base, -exp)
            } else {
                base * _const_powi(base, exp - 1)
            }
        }

        _const_powi(U::SLOPE, E)
    };
}
