use std::ops::Index;

use num_traits::NumCast;
pub trait F3lCast<T: NumCast>: Index<usize, Output = T> {
    #[inline]
    fn cast_f32<const D: usize>(&self) -> [f32; D] {
        let mut out = [0f32; D];
        (0..D).for_each(|i| out[i] = self[i].to_f32().unwrap());
        out
    }
    #[inline]
    fn cast_f64<const D: usize>(&self) -> [f64; D] {
        let mut out = [0f64; D];
        (0..D).for_each(|i| out[i] = self[i].to_f64().unwrap());
        out
    }
}

impl<T: NumCast, const D: usize> F3lCast<T> for [T; D] {}
