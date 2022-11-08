use num_traits::Zero;
use ordered_float::NotNan;

pub fn not_nan_or_zero(f: f64) -> NotNan<f64> {
    match NotNan::new(f) {
        Ok(x) => x,
        Err(_) => NotNan::zero(),
    }
}
