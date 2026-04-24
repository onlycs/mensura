# mensura

Compile-time dimensional analysis for physical quantities in Rust, built on const generics.

## Features

- **Same dimension? Same type**: There is exactly one type per physical dimension. `Per<Length, Time>`, `Velocity`, and any other expression that resolves to `L=1 T=-1` are all the exact same Rust type: `Quantity<{ Dimension { L: 1, T: -1, .. } }>`.

- **Fully const**: Construction, conversion, and arithmetic are all `const fn`. You can define physical constants in `const` items and use them in array sizes, pattern guards, and other const contexts without lifting to runtime.

- **Compiles fast**: With trust me bro benchmarks, I can confidently say that this compiles faster than `uom`.

- **Make your own units**: With a couple of trait implementations and a type alias, you can make your very own [`furlong_per_fortnight`](#furlong-per-fortnight).

- **No `std`? No problem**: No heap? No allocator? Weird requirements? We got you covered. Features are gated behind `std`, `alloc`, and `libm` features (note: on by default, certain math operations are not possible in `core`).

- **Works with some libraries**: `serde`, `bytemuck`, `num-traits`, `approx`, and `rand`.

Requires nightly Rust (see [Nightly features](#nightly-features)).

---

## Quick start

```rust
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use mensura::{
    quantity::{Length, Time, Velocity},
    unit::{length, time, velocity},
};

// Construct values in any unit - stored internally in SI.
let dist = Length::new::<length::kilometer>(1.5); // 1500 m internally
let dur  = Time::new::<time::minute>(2.0);         // 120 s internally

// Arithmetic produces the correct derived type at compile time.
let vel: Velocity = dist / dur;

// Read back in any compatible unit.
println!("{:.2} km/h", vel.get::<velocity::kilometer_per_hour>());
```

Adding mismatched dimensions is a **compile error** -- no runtime panics, no newtypes to unwrap:

```rust
let _ = dist + dur; // error[E0080]: evaluation of constant value failed
```

## Same dimension = same type

Most dimensional-analysis libraries assign a distinct named type to every quantity. That means `Torque` and `Energy` are different types even though both have dimension `M^1 L^2 T^-2`, and converting between them requires explicit ceremony.

Here they are the same type:

```rust
use mensura::quantity::{Torque, Energy, Times, Force, Length, Mass, Acceleration};

// All three resolve to Quantity<{ L:2, M:1, T:-2, .. }>:
type A = Times<Force, Length>;
type B = Times<Mass, Times<Acceleration, Length>>;

fn work_from_torque(t: Torque) -> Energy { t } // no conversion needed
```

You can freely name derived quantities with combinators, without registering them anywhere:

```rust
use mensura::quantity::{Per, Times, Raise, Length, Time, Mass};

type Velocity       = Per<Length, Time>;           // m·s^-1
type Area           = Raise<Length, 2>;            // m^2
type MassFlowRate   = Per<Mass, Time>;             // kg·s^-1
type SpecificVolume = Per<Raise<Length, 3>, Mass>; // m^3·kg^-1
```

## Const safety

All core operations are `const fn`:

```rust
const EARTH_RADIUS: Length   = Length::new_si(6_371_000.0);
const LIGHT_SPEED:  Velocity = Velocity::new_si(299_792_458.0);
const ONE_MINUTE:   Time     = Time::new::<time::minute>(1.0);
```

Dimension checks are evaluated at compile time as const-generic predicates -- no macros, no runtime dispatch, no unsafe.

## Architecture

| Module      | Contents                                                                                                                                                     |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `dimension` | `Dimension` -- seven `i32` exponents as a const-generic struct. Supports `+`, `-`, `*`, `/`, and negation at compile time.                                   |
| `quantity`  | `Quantity<const D: Dimension>` -- the dimensioned value wrapper; type aliases for common quantities; `Times`, `Per`, `Raise`, `Root`, `Inverse` combinators. |
| `unit`      | Zero-size unit structs (e.g. `unit::length::kilometer`) carrying conversion constants; generic `per`, `times`, `raise` combinators.                          |

## Creating and reading quantities

```rust
use mensura::{
    quantity::{Length, Temperature},
    unit::{length, temperature},
};

// Linear units
let d = Length::new::<length::foot>(6.0);
assert_eq!(d.get::<length::meter>(), 6.0 * 0.3048);

// Affine units (non-zero intercept, e.g. Celsius, Fahrenheit)
let t = Temperature::new_affine::<temperature::celsius>(100.0); // stored as 373.15 K
assert!((t.get::<temperature::kelvin>() - 373.15).abs() < 1e-9);

// Direct SI (skip unit conversion)
let d2 = Length::new_si(1.0);
assert_eq!(d2.si(), 1.0);
```

## Unit combinators

```rust
use mensura::unit::{per, times};
use mensura::unit::{length, time, force};

// Read a torque value in N·mm
let val = torque.get::<times<force::newton, length::millimeter>>();

// Read a velocity in km/h
let val = velocity.get::<per<length::kilometer, time::hour>>();
```

## How dimensional encoding works

Each `Quantity<D>` carries a single `Dimension` const generic whose fields are the exponents of the seven SI base dimensions:

| Field | Base quantity       | SI base unit  |
| ----- | ------------------- | ------------- |
| `L`   | Length              | metre (m)     |
| `M`   | Mass                | kilogram (kg) |
| `T`   | Time                | second (s)    |
| `I`   | Electric current    | ampere (A)    |
| `Th`  | Temperature         | kelvin (K)    |
| `N`   | Amount of substance | mole (mol)    |
| `J`   | Luminous intensity  | candela (cd)  |

Multiplying two quantities adds their `Dimension` values; dividing subtracts them. The resulting type is determined entirely at compile time with no runtime cost.

## Nightly features

Add this to the top of your `main.rs` or `lib.rs`:

```rust
#![feature(
    generic_const_exprs,
    const_trait_impl,
    const_ops,
    const_cmp,
    const_default,
    const_convert,
    adt_const_params
)]
#![allow(incomplete_features)]
```

## Furlong per fortnight

```rust
use mensura::{
    unit::{length, time, per, Unit},
    quantity::{Length, Time},
};

struct furlong;

impl Unit for furlong {
    const DIM: Dimension = Length::DIM;
    const SLOPE: f64 = 201.168; // m per furlong
}

struct fortnight;

impl Unit for fortnight {
    const DIM: Dimension = Time::DIM;
    const SLOPE: f64 = 14.0 * 24.0 * 3600.0; // s per fortnight
}

type furlong_per_fortnight = per<furlong, fortnight>;
```

## License

Mozilla Public License 2.0
