//! Compile-time dimensional analysis for physical quantities.
//!
//! This crate provides a [`Quantity`](quantity::Quantity) type parameterized by
//! a single [`Dimension`](dimension::Dimension) const generic, which encodes
//! the seven SI base-dimension exponents. Illegal operations like adding a
//! [`Length`](quantity::Length) to a [`Time`](quantity::Time) are caught at
//! compile time with no runtime overhead.
//!
//! # Quick start
//!
//! ```
//! #![feature(generic_const_exprs)]
//! #![allow(incomplete_features)]
//!
//! use mensura::{
//!     quantity::{Length, Time, Velocity},
//!     unit::{length, time, velocity},
//! };
//!
//! // Construct values in any unit - stored internally in SI.
//! let dist = Length::new::<length::kilometer>(1.5); // 1500 m internally
//! let dur = Time::new::<time::minute>(2.0); // 120 s internally
//!
//! // Arithmetic produces the correct derived type at compile time.
//! let vel = dist / dur; // Velocity (m·s^-1)
//!
//! // Read back in any compatible unit.
//! println!("{:.2} km/h", vel.get::<velocity::kilometer_per_hour>());
//! ```
//!
//! # Architecture
//!
//! Three modules:
//!
//! - **[`dimension`]** - the [`Dimension`](dimension::Dimension) struct, which
//!   stores seven `i32` exponents as a const-generic value. Supports
//!   compile-time math operations for deriving new dimensions from existing
//!   ones.
//! - **[`quantity`]** - [`Quantity<const D: Dimension>`](quantity::Quantity),
//!   the dimensioned value wrapper, plus type aliases for all common physical
//!   quantities ([`Length`](quantity::Length), [`Force`](quantity::Force),
//!   etc.) and type-level combinators ([`Times`](quantity::Times),
//!   [`Per`](quantity::Per), [`Raise`](quantity::Raise), etc.).
//! - **[`mod@unit`]** - zero-size unit structs (e.g.
//!   [`unit::length::kilometer`]) that carry conversion constants used by
//!   [`Quantity::new`](quantity::Quantity::new) and
//!   [`Quantity::get`](quantity::Quantity::get).
//!
//! # How dimensional encoding works
//!
//! Every [`Quantity<D>`](quantity::Quantity) carries a single
//! [`Dimension`](dimension::Dimension) const generic whose fields are the
//! exponents of the seven SI base dimensions:
//!
//! | Field | Base quantity       | SI base unit  |
//! |-------|---------------------|---------------|
//! | `L`   | Length              | metre (m)     |
//! | `M`   | Mass                | kilogram (kg) |
//! | `T`   | Time                | second (s)    |
//! | `I`   | Electric current    | ampere (A)    |
//! | `Th`  | Temperature         | kelvin (K)    |
//! | `N`   | Amount of substance | mole (mol)    |
//! | `J`   | Luminous intensity  | candela (cd)  |
//!
//! For example, velocity has `D = Dimension { L: 1, T: -1, .. }` and
//! force has `D = Dimension { L: 1, M: 1, T: -2, .. }`. Multiplying
//! or dividing two `Quantity` values adds or subtracts their `Dimension` values
//! at compile time, producing the correct derived type automatically.
//!
//! # Creating and reading quantities
//!
//! ```
//! use mensura::{
//!     quantity::{Length, Temperature},
//!     unit::{length, temperature},
//! };
//!
//! // Linear units: Quantity::new / Quantity::get
//! let d = Length::new::<length::foot>(6.0);
//! assert_eq!(d.get::<length::meter>(), 6.0 * 0.3048);
//!
//! // Affine units (non-zero intercept): Quantity::new_affine / Quantity::get
//! let t = Temperature::new_affine::<temperature::celsius>(100.0); // 373.15 K
//! assert!((t.get::<temperature::kelvin>() - 373.15).abs() < 1.0e-9);
//!
//! // Direct SI construction (skips unit conversion):
//! let d2 = Length::new_si(1.0); // 1 metre
//! assert_eq!(d2.si(), 1.0);
//! ```
//!
//! # Type-level combinators
//!
//! The [`quantity`] module re-exports combinator type aliases so you can name
//! derived dimensions without spelling out the `Dimension` struct manually:
//!
//! ```
//! use mensura::quantity::{Inverse, Length, Per, Raise, Root, Time};
//!
//! type Velocity = Per<Length, Time>; // m*s^-1
//! type Area = Raise<Length, 2>; // m^2
//! type RootArea = Root<Area, 2>; // m  (same type as Length)
//! type InvTime = Inverse<Time>; // s^-1 (= Frequency)
//! ```
//!
//! # Unit combinators
//!
//! The [`mod@unit`] module provides generic combinators for building composite
//! units at the type level:
//!
//! ```ignore
//! use mensura::unit::{force, inverse, length, per, raise, time, times};
//!
//! // Read a torque value in N*mm
//! let val = torque.get::<times<force::newton, length::millimeter>>();
//!
//! // Read a velocity in km/h
//! let val = velocity.get::<per<length::kilometer, time::hour>>();
//! ```
//!
//! # Nightly features
//!
//! This crate requires several unstable features:
//!
//! - `generic_const_exprs` - const-generic arithmetic in where clauses and
//!   output types (needed for `Mul`, `Div`, `powi`, `sqrt`, etc.).
//! - `const_trait_impl` - `const` trait implementations on operators.
//! - `const_ops` - `std::ops` traits in const contexts.
//! - `const_cmp`, `const_default`, `const_convert` - const `PartialEq`,
//!   `Default`, and `From`/`Into` used in dimension checks.
//! - `adt_const_params` - allows `Dimension` (a struct) to be used as a const
//!   generic parameter.
//!
//! To use all of the features in this crate, put this in your `main.rs` or
//! `lib.rs`:
//! ```
//! #![feature(
//!     generic_const_exprs, // required to do math
//!     const_ops, // required if you want math in `const` contexts (e.g. const fn, etc.)
//!     const_cmp, // required if you want to compare quantities in const contexts (i.e. PartialOrd/PartialEq)
//!     const_default, // useful when declaring your own Dimensions in const contexts
//!     const_convert, // required for From/Into/Deref/DerefMut for Ratio and f64 in const contexts
//!     adt_const_params, // required to use the Dimension struct as a const generic parameter
//!     core_float_math // sqrt, powi, etc in no-std contexts
//! )]
//! #![allow(incomplete_features)]
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(
    generic_const_exprs,
    const_trait_impl,
    const_ops,
    const_cmp,
    const_default,
    const_convert,
    adt_const_params,
    core_float_math
)]
#![allow(incomplete_features, non_upper_case_globals)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod dimension;
pub mod quantity;
pub mod unit;
