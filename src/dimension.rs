#![allow(non_snake_case)]

#[cfg(any(feature = "std", feature = "alloc"))]
use core::fmt;
use core::{marker::ConstParamTy, ops::*};

/// The seven SI base-dimension exponents, stored as a single const-generic
/// value on [`Quantity`](crate::quantity::Quantity).
///
/// Each field is the exponent of the corresponding SI base dimension. A
/// dimensionless ratio has all fields equal to zero (the `Default` value).
/// The arithmetic impls let you derive new dimensions from existing ones
/// entirely at compile time.
///
/// # Example
///
/// ```
/// #![feature(const_default, const_trait_impl)]
///
/// use mensura::dimension::Dimension;
///
/// const VELOCITY: Dimension = Dimension {
///     L: 1,
///     T: -1,
///     ..Dimension::default()
/// };
/// const FORCE: Dimension = Dimension {
///     L: 1,
///     M: 1,
///     T: -2,
///     ..Dimension::default()
/// };
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, ConstParamTy)]
pub struct Dimension {
    /// Length exponent (SI base unit: metre, m).
    pub L: i32,
    /// Mass exponent (SI base unit: kilogram, kg).
    pub M: i32,
    /// Time exponent (SI base unit: second, s).
    pub T: i32,
    /// Electric-current exponent (SI base unit: ampere, A).
    pub I: i32,
    /// Thermodynamic-temperature exponent (SI base unit: kelvin, K).
    pub Th: i32,
    /// Amount-of-substance exponent (SI base unit: mole, mol).
    pub N: i32,
    /// Luminous-intensity exponent (SI base unit: candela, cd).
    pub J: i32,
}

/// Returns the dimensionless (all-zeros) dimension, representing a pure ratio.
impl const Default for Dimension {
    fn default() -> Self {
        Self {
            L: 0,
            M: 0,
            T: 0,
            I: 0,
            Th: 0,
            N: 0,
            J: 0,
        }
    }
}

/// Element-wise addition of exponents - used when multiplying two quantities.
impl const Add for Dimension {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            L: self.L + rhs.L,
            M: self.M + rhs.M,
            T: self.T + rhs.T,
            I: self.I + rhs.I,
            Th: self.Th + rhs.Th,
            N: self.N + rhs.N,
            J: self.J + rhs.J,
        }
    }
}

/// Element-wise subtraction of exponents - used when dividing two quantities.
impl const Sub for Dimension {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            L: self.L - rhs.L,
            M: self.M - rhs.M,
            T: self.T - rhs.T,
            I: self.I - rhs.I,
            Th: self.Th - rhs.Th,
            N: self.N - rhs.N,
            J: self.J - rhs.J,
        }
    }
}

/// Scales every exponent by a scalar - used when raising a quantity to an
/// integer power via [`Quantity::powi`](crate::quantity::Quantity::powi).
impl const Mul<i32> for Dimension {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            L: self.L * rhs,
            M: self.M * rhs,
            T: self.T * rhs,
            I: self.I * rhs,
            Th: self.Th * rhs,
            N: self.N * rhs,
            J: self.J * rhs,
        }
    }
}

/// Divides every exponent by a scalar - used when taking an nth root via
/// [`Quantity::nroot`](crate::quantity::Quantity::nroot).
///
/// Panics at compile time if any exponent is not evenly divisible by `rhs`.
impl const Div<i32> for Dimension {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        let rhs = (checks::can_divide(self, rhs) as i32) * rhs;

        Self {
            L: self.L / rhs,
            M: self.M / rhs,
            T: self.T / rhs,
            I: self.I / rhs,
            Th: self.Th / rhs,
            N: self.N / rhs,
            J: self.J / rhs,
        }
    }
}

/// Negates every exponent - produces the reciprocal dimension (e.g. `s^-1`
/// from `s`). Used by `f64 / Quantity<D>`.
impl const Neg for Dimension {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            L: -self.L,
            M: -self.M,
            T: -self.T,
            I: -self.I,
            Th: -self.Th,
            N: -self.N,
            J: -self.J,
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl fmt::Debug for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        use alloc::{format, string::String};

        let mut buf = String::new();

        macro_rules! write_dim {
            ($d:ident) => {
                if self.$d != 0 {
                    buf.push_str(&format!("{}={}, ", stringify!($d), self.$d));
                }
            };
        }

        write_dim!(L);
        write_dim!(M);
        write_dim!(T);
        write_dim!(I);
        write_dim!(Th);
        write_dim!(N);
        write_dim!(J);

        buf.pop(); // Remove trailing space
        buf.pop(); // Remove trailing comma

        write!(f, "{buf}")
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        use alloc::{format, string::String};

        fn format_unit(v: i32, s: &str, buf: &mut String) {
            if v == 0 {
                return;
            } else if v == 1 {
                buf.push_str(s);
            } else {
                buf.push_str(format!("{}^{}", s, v).as_str());
            }

            buf.push('·');
        }

        const L_STR: &str = "m";
        const M_STR: &str = "kg";
        const T_STR: &str = "s";
        const I_STR: &str = "A";
        const TH_STR: &str = "K";
        const N_STR: &str = "mol";
        const J_STR: &str = "cd";

        let mut buf = String::new();

        format_unit(self.L, L_STR, &mut buf);
        format_unit(self.M, M_STR, &mut buf);
        format_unit(self.T, T_STR, &mut buf);
        format_unit(self.I, I_STR, &mut buf);
        format_unit(self.Th, TH_STR, &mut buf);
        format_unit(self.N, N_STR, &mut buf);
        format_unit(self.J, J_STR, &mut buf);

        buf.pop(); // Remove trailing cdot

        write!(f, "{buf}")
    }
}

/// Compile-time predicates used as `where`-clause guards on `Quantity` methods.
///
/// Each function returns a `bool` that is then cast to `usize` and used as an
/// array length in a `[(); N]: True` bound, causing a compile error when the
/// predicate is false.
pub(crate) mod checks {
    use super::Dimension;

    /// Returns `true` if `d1` and `d2` have identical exponents.
    ///
    /// Used to enforce that only quantities of matching dimension can be added,
    /// subtracted, or constructed from a particular unit.
    pub const fn eq(d1: Dimension, d2: Dimension) -> bool {
        d1.L == d2.L
            && d1.M == d2.M
            && d1.T == d2.T
            && d1.I == d2.I
            && d1.Th == d2.Th
            && d1.N == d2.N
            && d1.J == d2.J
    }

    /// Returns `true` if every exponent in `d` is evenly divisible by `rhs`.
    ///
    /// Used to enforce that `sqrt` (divisor 2) and `nroot` (divisor N) are
    /// only callable when the resulting exponents would be integers.
    pub const fn can_divide(d: Dimension, rhs: i32) -> bool {
        d.L % rhs == 0
            && d.M % rhs == 0
            && d.T % rhs == 0
            && d.I % rhs == 0
            && d.Th % rhs == 0
            && d.N % rhs == 0
            && d.J % rhs == 0
    }
}
