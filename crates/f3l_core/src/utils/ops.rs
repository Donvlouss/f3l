use num_traits::{Float, Num, Pow};

#[inline]
pub fn round_n<T: Float>(me: T, n: usize) -> T {
    (me * T::from(10.pow(n)).unwrap()).round() / T::from(10.pow(n)).unwrap()
}

#[inline]
pub fn round_slice_n<T: Float, const D: usize>(me: [T; D], n: usize) -> [T; D] {
    let mut out = me;
    for v in out.iter_mut() {
        *v = ((*v) * T::from(10.pow(n)).unwrap()).round() / T::from(10.pow(n)).unwrap();
    }
    out
}

#[inline]
pub fn apply_both<T: Num + Copy, const D: usize, F: FnMut(T, T) -> T>(me: &[T; D], other: &[T], mut ops: F) -> [T; D] {
    let mut out = [T::zero(); D];
    (0..D)
        .for_each(|i| out[i] = ops(me[i], other[i]));
    out
}

#[inline]
pub fn apply_each<T: Num + Copy, const D: usize, F: FnMut(T, T) -> T>(me: &[T; D], other: T, mut ops: F) -> [T; D] {
    let mut out = [T::zero(); D];
    (0..D)
        .for_each(|i| out[i] = ops(me[i], other));
    out
}

#[test]
fn test_ops_unsigned() {
    let a = [0usize, 1, 2];
    let b = [2usize, 3, 4];

    let c = apply_both(&a, &b, std::ops::Add::add);
    assert_eq!(c, [2usize, 4, 6]);
}

#[test]
fn test_ops_float() {
    let a = [0f32, 1., 2.];
    let b = [2f32, 3., 4.];

    let c = apply_both(&a, &b, std::ops::Mul::mul);
    assert_eq!(c, [0f32, 3., 8.]);
}