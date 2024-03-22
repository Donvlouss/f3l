use std::ops::{Index, IndexMut};

use num_traits::{Float, Num, NumAssign, NumOps};

pub trait BasicFloat: Float + NumOps + NumAssign + Send + Sync + Copy + Clone {}
impl BasicFloat for f32 {}
impl BasicFloat for f64 {}

pub trait SimpleSliceMath<T: BasicFloat, const D: usize>
where
    Self: Index<usize, Output = T> + IndexMut<usize, Output = T> + Copy,
{
    #[inline]
    fn dot(&self, other: &Self) -> T {
        (0..D).fold(T::zero(), |acc, i| acc + self[i] * other[i])
    }
    fn cross(&self, other: &Self) -> Self;
    #[inline]
    fn compute_angle(&self, other: &Self) -> T {
        let v = self.dot(other) / (self.len() * other.len());
        let v = v.max(-T::one());
        let v = v.min(T::one());

        v.acos()
    }
    #[inline]
    fn len(&self) -> T {
        (0..D)
            .fold(T::zero(), |acc, i| acc + self[i].powi(2))
            .sqrt()
    }
    #[inline]
    fn len_squared(&self) -> T {
        (0..D).fold(T::zero(), |acc, i| acc + self[i].powi(2))
    }
    #[inline]
    fn normalize(&mut self) {
        let d = T::one() / self.len();
        (0..D).for_each(|i| {
            self[i] = self[i] * d;
        })
    }
    #[inline]
    fn normalized(&self) -> Self {
        let mut out = *self;
        let d = T::one() / self.len();
        (0..D).for_each(|i| {
            out[i] = self[i] * d;
        });
        out
    }
    #[inline]
    fn distance_between(&self, other: &Self) -> T {
        self.distance_square_between(other).sqrt()
    }
    #[inline]
    fn distance_square_between(&self, other: &Self) -> T {
        (0..D).fold(T::zero(), |acc, i| acc + (self[i] - other[i]).powi(2))
    }
}

pub trait F3lSlice<T> {
    type EXPAND;
    type TRUNCATE;

    fn truncate(&self) -> Self::TRUNCATE;
    fn expand(&self, v: T) -> Self::EXPAND;
    fn head(&self, n: usize) -> &[T];
    fn tail(&self, n: usize) -> &[T];
}

impl<T: BasicFloat, const D: usize> SimpleSliceMath<T, D> for [T; D]
where
    Self: Index<usize, Output = T> + IndexMut<usize, Output = T> + Copy,
{
    fn cross(&self, other: &Self) -> Self {
        let mut out = [T::zero(); D];
        match D {
            3 => {
                out[0] = self[1] * other[2] - self[2] * other[1];
                out[1] = self[2] * other[0] - self[0] * other[2];
                out[2] = self[0] * other[1] - self[1] * other[0];
            }
            4 => {
                out[0] = self[1] * other[2] - self[2] * other[1];
                out[1] = self[2] * other[0] - self[0] * other[2];
                out[2] = self[0] * other[1] - self[1] * other[0];
                out[3] = T::one();
            }
            _ => {}
        };
        out
    }
}

#[test]
fn test_dot() {
    let a = [1f32, 2.];
    let b = [2f32, 3.];
    println!("{}", a.dot(&b));
}

impl<T: Num + Copy> F3lSlice<T> for [T; 2] {
    type EXPAND = [T; 3];

    type TRUNCATE = [T; 1];

    fn truncate(&self) -> Self::TRUNCATE {
        [self[0]]
    }

    fn expand(&self, v: T) -> Self::EXPAND {
        [self[0], self[1], v]
    }

    fn head(&self, n: usize) -> &[T] {
        let n = match n {
            0 => 0,
            1..=2 => n - 1,
            _ => 1,
        };
        &self[..(n - 1)]
    }

    fn tail(&self, n: usize) -> &[T] {
        let n = match n {
            0 => 0,
            1..=2 => n - 1,
            _ => 1,
        };
        &self[n..]
    }
}

impl<T: Num + Copy> F3lSlice<T> for [T; 3] {
    type EXPAND = [T; 4];

    type TRUNCATE = [T; 2];

    fn truncate(&self) -> Self::TRUNCATE {
        [self[0], self[1]]
    }

    fn expand(&self, v: T) -> Self::EXPAND {
        [self[0], self[1], self[2], v]
    }

    fn head(&self, n: usize) -> &[T] {
        let n = match n {
            0 => 0,
            1..=3 => n - 1,
            _ => 2,
        };
        &self[..(n - 1)]
    }

    fn tail(&self, n: usize) -> &[T] {
        let n = match n {
            0 => 0,
            1..=3 => n - 1,
            _ => 2,
        };
        &self[n..]
    }
}

impl<T: Num + Copy> F3lSlice<T> for [T; 4] {
    type EXPAND = [T; 5];

    type TRUNCATE = [T; 3];

    fn truncate(&self) -> Self::TRUNCATE {
        [self[0], self[1], self[2]]
    }

    fn expand(&self, v: T) -> Self::EXPAND {
        [self[0], self[1], self[2], self[3], v]
    }

    fn head(&self, n: usize) -> &[T] {
        let n = match n {
            0 => 0,
            1..=4 => n - 1,
            _ => 3,
        };
        &self[..(n - 1)]
    }

    fn tail(&self, n: usize) -> &[T] {
        let n = match n {
            0 => 0,
            1..=4 => n - 1,
            _ => 3,
        };
        &self[n..]
    }
}
