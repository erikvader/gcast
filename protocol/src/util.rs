use num_traits::Zero;
pub use ordered_float::NotNan;

pub fn not_nan(f: f64) -> NotNan<f64> {
    match NotNan::new(f) {
        Ok(x) => x,
        Err(_) => NotNan::zero(),
    }
}

#[derive(PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct Percent {
    inner: NotNan<f64>,
}

impl Percent {
    pub const ZERO: Self = Percent {
        inner: unsafe { NotNan::new_unchecked(0.0) },
    };

    pub fn try_new(percent: f64) -> Option<Self> {
        if percent.is_finite() {
            Some(Percent {
                inner: NotNan::new(percent.clamp(0.0, 100.0)).expect("not nan"),
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn divide_by_zero() {
        assert!(Percent::of(5.0, 0.0).is_none());
    }
}
