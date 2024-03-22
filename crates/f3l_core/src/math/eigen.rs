use std::{
    cmp::Ordering,
    ops::{Deref, DerefMut},
};

use crate::{BasicFloat, SimpleSliceMath};

#[derive(Debug, Clone, Copy)]
pub struct Eigen<T: BasicFloat, const D: usize> {
    pub eigenvalue: T,
    pub eigenvector: [T; D],
}

impl<T: BasicFloat, const D: usize> Default for Eigen<T, D> {
    fn default() -> Self {
        Self {
            eigenvalue: T::zero(),
            eigenvector: [T::zero(); D],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EigenSet<T: BasicFloat, const D: usize, const N: usize>(pub [Eigen<T, D>; N]);

impl<T: BasicFloat, const D: usize, const N: usize> From<[[T; D]; N]> for EigenSet<T, D, N> {
    fn from(value: [[T; D]; N]) -> Self {
        let mut me = Self::default();
        (0..N).for_each(|n| {
            let eigen = value[n];
            me[n] = Eigen {
                eigenvalue: eigen.len(),
                eigenvector: eigen,
            }
        });
        me
    }
}

impl<T: BasicFloat, const D: usize, const N: usize> Default for EigenSet<T, D, N> {
    fn default() -> Self {
        Self([Eigen::default(); N])
    }
}

impl<T: BasicFloat, const D: usize, const N: usize> Deref for EigenSet<T, D, N> {
    type Target = [Eigen<T, D>; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: BasicFloat, const D: usize, const N: usize> DerefMut for EigenSet<T, D, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: BasicFloat, const D: usize, const N: usize> EigenSet<T, D, N> {
    fn compare(a: &Eigen<T, D>, b: &Eigen<T, D>) -> Option<Ordering> {
        a.eigenvalue.abs().partial_cmp(&b.eigenvalue.abs())
    }

    pub fn eigenvalues(&self) -> [T; N] {
        let mut out = [T::zero(); N];
        (0..N).for_each(|i| {
            out[i] = self[i].eigenvalue;
        });
        out
    }

    pub fn eigenvectors_2d_row_major(&self) -> [[T; D]; N] {
        let mut out = [[T::zero(); D]; N];
        (0..N).for_each(|r| {
            (0..D).for_each(|c| {
                out[r][c] = self[r].eigenvector[c];
            });
        });
        out
    }

    pub fn eigenvectors_2d_column_major(&self) -> [[T; N]; D] {
        let mut out = [[T::zero(); N]; D];
        (0..D).for_each(|r| {
            (0..N).for_each(|c| {
                out[r][c] = self[c].eigenvector[r];
            });
        });
        out
    }

    pub fn sort(&mut self) {
        self.sort_by(|a, b| Self::compare(a, b).unwrap())
    }

    pub fn reverse(&mut self) {
        self.sort_by(|a, b| Self::compare(b, a).unwrap())
    }

    pub fn largest_id(&self) -> usize {
        self.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| Self::compare(a, b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap()
    }

    pub fn largest(&self) -> Eigen<T, D> {
        self.iter()
            .max_by(|a, b| Self::compare(a, b).unwrap())
            .copied()
            .unwrap()
    }

    pub fn minimal_id(&self) -> usize {
        self.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| Self::compare(a, b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap()
    }

    pub fn minimal(&self) -> Eigen<T, D> {
        self.iter()
            .min_by(|a, b| Self::compare(a, b).unwrap())
            .copied()
            .unwrap()
    }
}
