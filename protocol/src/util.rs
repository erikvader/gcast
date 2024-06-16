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

    pub fn try_new(percent: f64) -> Option<Percent> {
        if percent.is_finite() && percent >= 0.0 {
            Some(Percent {
                inner: NotNan::new(percent).expect("not nan"),
            })
        } else {
            None
        }
    }

    pub fn as_f64(self) -> f64 {
        *self.inner
    }
}
