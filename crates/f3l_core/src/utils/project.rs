use crate::{BasicFloat, SimpleSliceMath};

/// Compute distance of a project on b
#[inline]
pub fn project_len<T: BasicFloat, const D: usize>(a: &[T; D], b: &[T; D]) -> T {
    (*a).dot(b) / b.len().powi(2)
}

/// Compute `scaled` vector of a project on b <br>
/// - Length:       length of vector
/// - Direction: normalize of vector
pub fn project_vector<T: BasicFloat, const D: usize>(a: &[T; D], b: &[T; D]) -> [T; D] {
    let mut out = *b;
    let factor = project_len(a, b);
    for v in out.iter_mut() {
        *v *= factor;
    }
    out
}
