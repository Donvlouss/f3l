use std::ops::{Deref, DerefMut};

use f3l_core::matrix3x3::EigenVector;


#[derive(Debug, Clone, Copy, Default)]
pub struct Eigenvectors3(pub [EigenVector; 3]);

impl Deref for Eigenvectors3 {
    type Target = [EigenVector; 3];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Eigenvectors3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Eigenvectors3 {
    pub fn sort(&mut self) {
        self.sort_by(|a, b| a.eigenvalues.partial_cmp(&b.eigenvalues).unwrap())
    }

    pub fn largest_id(&self) -> usize {
        self.iter()
            .enumerate()
            .max_by(|(_, &a), (_, &b)| a.eigenvalues.partial_cmp(&b.eigenvalues).unwrap())
            .map(|(idx,_)| idx)
            .unwrap()
    }

    pub fn largest(&self) -> EigenVector {
        self.iter()
            .max_by(|&a, &b| a.eigenvalues.partial_cmp(&b.eigenvalues).unwrap())
            .map(|entry| *entry)
            .unwrap()
    }
}


