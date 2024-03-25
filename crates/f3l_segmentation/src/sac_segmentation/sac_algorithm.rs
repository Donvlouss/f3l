mod sac_ransac;
pub use sac_ransac::*;

use f3l_core::BasicFloat;

use super::sac_model::SacModel;

/// Algorithm of Optimization
#[derive(Debug, Default, Clone, Copy)]
pub enum SacAlgorithmType {
    #[default]
    RANSAC,
}

/// Parameter of algorithm of Optimization
#[derive(Debug, Clone, Copy)]
pub struct SacAlgorithmParameter {
    /// Probability: default `0.99`
    pub probability: f32,
    /// Value of threshold
    pub threshold: f32,
    /// If reach `max_iteration`, Optimization will be `terminate`.
    pub max_iterations: usize,
    /// Use parallel. Default `1` (single thread).
    pub threads: usize,
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

/// A trait to get inliers data.
pub trait SacAlgorithmGetter {
    fn get_inliers(&self) -> &Vec<usize>;
}

/// A trait to support algorithm computing. 
pub trait SacAlgorithm<'a, P: Copy, T, R>: SacAlgorithmGetter
where
    T: BasicFloat,
    R: SacModel<'a, P, T>,
{
    fn compute(&mut self, model: &mut R) -> bool;
}
