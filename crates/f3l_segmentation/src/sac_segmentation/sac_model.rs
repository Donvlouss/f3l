use rand::Rng;
use f3l_core::BasicFloat;

mod sac_model_plane;
mod sac_model_line;
mod sac_model_circle3d;
pub use sac_model_plane::*;
pub use sac_model_line::*;
pub use sac_model_circle3d::*;

#[derive(Debug, Default, Clone, Copy)]
pub enum SacModelType {
    #[default]
    SacModelPlane,
    SacModelLine,
    SacModelCircle3d,
    SacModelSphere,
}

pub trait SacModel<'a, P, T: BasicFloat> {
    type DataType;
    type SampleIdxType;
    type CoefficientsType;

    const NB_SAMPLE:usize;
    const NB_COEFFICIENTS: usize;

    fn compute_point_to_model(p: P, coefficients: &Self::CoefficientsType) -> T;
    fn set_data(&mut self, data: &'a Vec<P>);
    fn set_coefficient(&mut self, factor: &Self::CoefficientsType);
    fn get_coefficient(&self) -> Self::CoefficientsType;

    fn samples(&self) -> &Vec<Self::DataType>;
    fn data_len(&self) -> usize {
        self.samples().len()
    }
    fn get_random_sample_id(&self) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let nb = self.data_len();
        use std::collections::HashSet;
        let mut set = HashSet::new();
        while set.len() < Self::NB_SAMPLE {
            set.insert(rng.gen_range(0..nb));
        }
        set.into_iter().map(|v| v).collect()
    }
    fn get_random_samples(&self) -> Self::SampleIdxType;
    fn compute_model_coefficients(&self, samples: &Self::SampleIdxType) -> Result<Self::CoefficientsType, String>;
    fn get_distance_to_model(&self, coefficients: &Self::CoefficientsType) -> Vec<T>;
    fn select_indices_within_tolerance(&self, coefficients: &Self::CoefficientsType, tolerance: T) -> Vec<usize>;
    fn count_indices_within_tolerance(&self, coefficients: &Self::CoefficientsType, tolerance: T) -> usize;
}