use std::ops::{
    Index,
    IndexMut
};

use num_traits::{
    Float,
    NumAssign,
    NumOps
};

pub trait BasicFloat: Float + NumOps + NumAssign + Send + Sync + Copy + Clone {}
impl BasicFloat for f32 {}
impl BasicFloat for f64 {}

pub trait SimpleSliceMath<T: BasicFloat, U, const D: usize>
where
    Self: Index<U, Output = T> + IndexMut<U, Output = T> + Copy,
    U: From<usize>
{
    #[inline]
    fn dot(&self, other: &Self) -> T {
        (0..D)
            .fold(T::zero(), |acc, i| acc + self[i.into()] * other[i.into()])
    }
    fn cross(&self, other: &Self) -> Self;
    #[inline]
    fn compute_angle(&self, other: &Self) -> T {
        (self.dot(other) / (self.len() * other.len())).acos()
    }
    #[inline]
    fn len(&self) -> T {
        (0..D)
            .fold(T::zero(), |acc, i| acc + self[i.into()].powi(2))
            .sqrt()
    }
    #[inline]
    fn normalize(&mut self) {
        let d = T::one() / self.len();
        (0..D)
            .for_each(|i| {
                self[i.into()]  = self[i.into()] * d;
            })
    }
    #[inline]
    fn normalized(&self) -> Self {
        let mut out = *self;
        let d = T::one() / self.len();
        (0..D)
            .for_each(|i| {
                out[i.into()]  = self[i.into()] * d;
            });
        out
    }
}

impl <T: BasicFloat> SimpleSliceMath<T, usize, 2> for [T; 2] {
    #[inline]
    fn cross(&self, _other: &Self) -> Self {
        [T::zero(); 2]
    }
}

impl <T: BasicFloat> SimpleSliceMath<T, usize, 3> for [T; 3] {
    #[inline]
    fn cross(&self, other: &Self) -> Self {
        [
            self[1] * other[2] - self[2] * other[1],
            self[2] * other[0] - self[0] * other[2],
            self[0] * other[1] - self[1] * other[0]
        ]
    }
}

impl <T: BasicFloat> SimpleSliceMath<T, usize, 4> for [T; 4] {
    #[inline]
    fn cross(&self, other: &Self) -> Self {
        [
            self[1] * other[2] - self[2] * other[1],
            self[2] * other[0] - self[0] * other[2],
            self[0] * other[1] - self[1] * other[0],
            T::one()
        ]
    }
}