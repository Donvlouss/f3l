use f3l_core::BasicFloat;
use rand::Rng;

mod sac_model_circle3d;
mod sac_model_line;
mod sac_model_plane;
mod sac_model_sphere;
pub use sac_model_circle3d::*;
pub use sac_model_line::*;
pub use sac_model_plane::*;
pub use sac_model_sphere::*;

#[derive(Debug, Default, Clone, Copy)]
pub enum SacModelType {
    #[default]
    SacModelPlane,
    SacModelLine,
    SacModelCircle3d,
    SacModelSphere,
}

pub trait SacModel<'a, P: Copy, T: BasicFloat> {
    type SampleIdxType;
    type CoefficientsType;

    const NB_SAMPLE: usize;
    const NB_COEFFICIENTS: usize;

    fn set_data(&mut self, data: &'a Vec<P>);
    fn set_coefficient(&mut self, factor: &Self::CoefficientsType);
    fn get_coefficient(&self) -> Self::CoefficientsType;

    fn samples(&self) -> &Vec<P>;
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
    fn get_distance_to_model(&self, coefficients: &Self::CoefficientsType) -> Vec<T> {
        self.samples()
            .iter()
            .map(|&p| Self::compute_point_to_model(p, coefficients))
            .collect()
    }
    fn select_indices_within_tolerance(
        &self,
        coefficients: &Self::CoefficientsType,
        tolerance: T,
    ) -> Vec<usize> {
        let data = self.samples();
        (0..data.len())
            .filter(|&i| Self::compute_point_to_model(data[i], coefficients) < tolerance)
            .collect()
    }
    fn count_indices_within_tolerance(
        &self,
        coefficients: &Self::CoefficientsType,
        tolerance: T,
    ) -> usize {
        let data = self.samples();
        (0..data.len())
            .filter(|&i| Self::compute_point_to_model(data[i], coefficients) < tolerance)
            .map(|_| 1)
            .sum()
    }

    fn compute_point_to_model(p: P, coefficients: &Self::CoefficientsType) -> T;
    fn get_random_samples(&self) -> Self::SampleIdxType;
    fn compute_model_coefficients(
        &self,
        samples: &Self::SampleIdxType,
    ) -> Result<Self::CoefficientsType, String>;
}
