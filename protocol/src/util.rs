use derivative::Derivative;
use std::{fmt, marker::PhantomData};

use num_traits::Zero;
pub use ordered_float::NotNan;

pub fn not_nan(f: f64) -> NotNan<f64> {
    NotNan::new(f).unwrap_or_else(|_| NotNan::zero())
}

pub trait PercentKind {
    fn restrict(num: NotNan<f64>) -> Result<NotNan<f64>, NotNan<f64>>;
}

pub struct Unrestricted;

impl PercentKind for Unrestricted {
    fn restrict(num: NotNan<f64>) -> Result<NotNan<f64>, NotNan<f64>> {
        Ok(num)
    }
}

pub struct Normal;

impl PercentKind for Normal {
    fn restrict(num: NotNan<f64>) -> Result<NotNan<f64>, NotNan<f64>> {
        let clamped = num.clamp(NotNan::zero(), NotNan::new(100.0).unwrap());
        if clamped == num {
            Ok(clamped)
        } else {
            Err(clamped)
        }
    }
}

pub struct Positive;

impl PercentKind for Positive {
    fn restrict(num: NotNan<f64>) -> Result<NotNan<f64>, NotNan<f64>> {
        if num < NotNan::zero() {
            Err(NotNan::zero())
        } else {
            Ok(num)
        }
    }
}

#[derive(Derivative)]
#[derivative(Copy(bound = ""))]
#[derivative(Clone(bound = ""))]
#[derivative(PartialEq(bound = ""))]
#[derivative(Eq(bound = ""))]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = ""))]
#[serde(bound(deserialize = ""))]
pub struct Percent<Kind = Unrestricted> {
    _kind: PhantomData<Kind>,
    num: NotNan<f64>,
}

impl<Kind> Percent<Kind>
where
    Kind: PercentKind,
{
    /// Creates a percentage out of a float. Returns None if it is NaN or is outside of
    /// range of the kind.
    pub fn try_new(num: f64) -> Option<Self> {
        NotNan::new(num)
            .ok()
            .and_then(|f| Kind::restrict(f).ok())
            .map(|f| Self {
                _kind: PhantomData,
                num: f,
            })
    }

    /// Create a percentage out of a float. Return None if it is NaN. Clamps the value to
    /// the allowed range of the kind.
    pub fn new(num: f64) -> Option<Self> {
        let Ok(num) = NotNan::new(num) else {
            return None;
        };
        let num = Kind::restrict(num).map_or_else(|x| x, |x| x);
        Some(Self {
            _kind: PhantomData,
            num,
        })
    }

    pub fn as_f64(self) -> f64 {
        *self.num
    }

    pub fn of(num: f64, total: f64) -> Option<Self> {
        if total == 0.0 {
            return None;
        }
        Self::new(100.0 * num / total)
    }
}

impl<Kind: PercentKind> fmt::Debug for Percent<Kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Percent").field(&self.as_f64()).finish()
    }
}

impl<Kind: PercentKind> fmt::Display for Percent<Kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.as_f64())
    }
}

impl<Kind: PercentKind> Default for Percent<Kind> {
    fn default() -> Self {
        Self::new(0.0).expect("all kinds accept 0")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn divide_by_zero() {
        assert!(Percent::<Normal>::of(5.0, 0.0).is_none());
        assert!(Percent::<Unrestricted>::of(5.0, 0.0).is_none());
    }

    #[test]
    fn outside_valid_range() {
        assert_eq!(
            Some(Percent::<Positive>::try_new(0.0).unwrap()),
            Percent::<Positive>::new(-1.0)
        );
        assert_eq!(
            Some(Percent::<Normal>::try_new(100.0).unwrap()),
            Percent::<Normal>::new(200.0)
        );
    }

    #[test]
    fn unrestricted_extremes() {
        assert!(Percent::<Unrestricted>::try_new(f64::INFINITY).is_some());
        assert!(Percent::<Unrestricted>::try_new(f64::NEG_INFINITY).is_some());
        assert!(Percent::<Unrestricted>::try_new(f64::NAN).is_none());
        assert!(Percent::<Unrestricted>::try_new(0.0).is_some());
    }

    #[test]
    fn normal_extremes() {
        assert!(Percent::<Normal>::try_new(f64::INFINITY).is_none());
        assert!(Percent::<Normal>::try_new(f64::NEG_INFINITY).is_none());
        assert!(Percent::<Normal>::try_new(f64::NAN).is_none());
        assert!(Percent::<Normal>::try_new(0.0).is_some());
    }

    #[test]
    fn positive_extremes() {
        assert!(Percent::<Positive>::try_new(f64::INFINITY).is_some());
        assert!(Percent::<Positive>::try_new(f64::NEG_INFINITY).is_none());
        assert!(Percent::<Positive>::try_new(f64::NAN).is_none());
        assert!(Percent::<Positive>::try_new(0.0).is_some());
    }
}
