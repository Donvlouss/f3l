mod sac_ransac;
pub use sac_ransac::*;

use f3l_core::BasicFloat;

use super::sac_model::SacModel;

#[derive(Debug, Default, Clone, Copy)]
pub enum SacAlgorithmType {
    #[default]
    RANSAC,
}


#[derive(Debug, Clone, Copy)]
pub struct SacAlgorithmParameter {
    pub probability: f32,
    pub threshold: f32,
    pub max_iterations: usize,
    pub threads: usize
}

impl Default for SacAlgorithmParameter {
    fn default() -> Self {
        Self {
            probability: 0.99,
            threshold: 0.1,
            max_iterations: 1000,
            threads: 1,
        }
    }
}

pub trait SacAlgorithmGetter {
    fn get_inliers(&self) -> &Vec<usize>;
}

pub trait SacAlgorithm<'a, P, T, R>: SacAlgorithmGetter
where
    T: BasicFloat,
    R: SacModel<'a, P, T>
{
    fn compute(&mut self, model: &mut R) -> bool;
}