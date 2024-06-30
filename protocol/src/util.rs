use std::fmt;

use num_traits::Zero;
pub use ordered_float::NotNan;

pub fn not_nan(f: f64) -> NotNan<f64> {
    match NotNan::new(f) {
        Ok(x) => x,
        Err(_) => NotNan::zero(),
    }
}

#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct Percent {
    inner: UnrestrictedPercent,
}

impl Percent {
    pub const ZERO: Self = Self {
        inner: UnrestrictedPercent::ZERO,
    };
    pub const MIN: Self = Self::ZERO;
    pub const MAX: Self = Self {
        inner: UnrestrictedPercent {
            inner: unsafe { NotNan::new_unchecked(100.0) },
        },
    };

    fn new(overload: UnrestrictedPercent) -> Self {
        Self {
            inner: overload.clamp(Self::MIN.inner, Self::MAX.inner),
        }
    }

    pub fn try_new(percent: f64) -> Option<Self> {
        UnrestrictedPercent::try_new(percent).map(Self::new)
    }

    pub fn of(num: f64, total: f64) -> Option<Self> {
        UnrestrictedPercent::of(num, total).map(Self::new)
    }

    pub fn as_f64(self) -> f64 {
        self.inner.as_f64()
    }
}

impl fmt::Debug for Percent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Percent").field(&self.as_f64()).finish()
    }
}

impl fmt::Display for Percent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Default for Percent {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct PositivePercent {
    inner: UnrestrictedPercent,
}

impl PositivePercent {
    pub const ZERO: Self = PositivePercent {
        inner: UnrestrictedPercent::ZERO,
    };
    pub const MIN: Self = Self::ZERO;

    fn new(overload: UnrestrictedPercent) -> Self {
        Self {
            inner: overload.max(Self::MIN.inner),
        }
    }

    pub fn try_new(percent: f64) -> Option<Self> {
        UnrestrictedPercent::try_new(percent).map(Self::new)
    }

    pub fn of(num: f64, total: f64) -> Option<Self> {
        UnrestrictedPercent::of(num, total).map(Self::new)
    }

    pub fn as_f64(self) -> f64 {
        self.inner.as_f64()
    }
}

impl fmt::Debug for PositivePercent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PositivePercent")
            .field(&self.as_f64())
            .finish()
    }
}

impl fmt::Display for PositivePercent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Default for PositivePercent {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct UnrestrictedPercent {
    inner: NotNan<f64>,
}

impl UnrestrictedPercent {
    pub const ZERO: Self = UnrestrictedPercent {
        inner: unsafe { NotNan::new_unchecked(0.0) },
    };

    pub fn try_new(percent: f64) -> Option<Self> {
        if percent.is_finite() {
            Some(Self {
                inner: NotNan::new(percent).expect("not nan"),
            })
        } else {
            None
        }
    }

    pub fn of(num: f64, total: f64) -> Option<Self> {
        Self::try_new(100.0 * num / total)
    }

    pub fn as_f64(self) -> f64 {
        *self.inner
    }

    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self {
            inner: self.inner.clamp(min.inner, max.inner),
        }
    }

    pub fn max(self, other: Self) -> Self {
        Self {
            inner: self.inner.max(other.inner),
        }
    }
}

impl fmt::Debug for UnrestrictedPercent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("UnrestrictedPercent")
            .field(&self.as_f64())
            .finish()
    }
}

impl fmt::Display for UnrestrictedPercent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.as_f64())
    }
}

impl Default for UnrestrictedPercent {
    fn default() -> Self {
        Self::ZERO
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn divide_by_zero() {
        assert!(Percent::of(5.0, 0.0).is_none());
        assert!(UnrestrictedPercent::of(5.0, 0.0).is_none());
    }

    #[test]
    fn from_nan() {
        assert!(Percent::try_new(f64::NAN).is_none());
        assert!(UnrestrictedPercent::try_new(f64::NAN).is_none());
    }

    #[test]
    fn outside_valid_range() {
        assert_eq!(Some(PositivePercent::MIN), PositivePercent::try_new(-1.0));
        assert_eq!(Some(Percent::MAX), Percent::try_new(200.0));
    }
}
