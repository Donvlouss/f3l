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
}

impl Default for SacAlgorithmParameter {
    fn default() -> Self {
        Self {
            probability: 0.99,
            threshold: 0.1,
            max_iterations: 100,
        }
    }
}

pub trait SacAlgorithm<'a, P, T, R>
where
    T: BasicFloat,
    R: SacModel<'a, P, T>
{
    fn get_inliers(&self) -> &Vec<usize>;
    fn compute(&mut self, model: &mut R) -> bool;
}