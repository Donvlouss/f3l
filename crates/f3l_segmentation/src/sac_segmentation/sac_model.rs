use f3l_core::{BasicFloat, serde::{self, Deserialize, Serialize}};
use rand::Rng;

mod sac_model_circle3d;
mod sac_model_line;
mod sac_model_plane;
mod sac_model_sphere;
pub use sac_model_circle3d::*;
pub use sac_model_line::*;
pub use sac_model_plane::*;
pub use sac_model_sphere::*;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate="self::serde")]
pub enum SacModelType {
    #[default]
    SacModelPlane,
    SacModelLine,
    SacModelCircle3d,
    SacModelSphere,
}

pub trait ModelCoefficient {
    type CoefficientsType;

    fn coe(&self) -> Self::CoefficientsType;
}

/// Represent any model.
///
/// Implement this to Customize model.
/// Currently support [`SacModelPlane`], [`SacModelSphere`],
/// [`SacModelLine`], [`SacModelCircle3d`]
pub trait SacModel<'a, P: Copy, T: BasicFloat> {
    type SampleIdxType;
    type CoefficientsType;

    const NB_SAMPLE: usize;
    const NB_COEFFICIENTS: usize;

    fn set_data(&mut self, data: &'a [P]);
    /// Set `NB_COEFFICIENTS` array.
    fn set_coefficient(&mut self, factor: &Self::CoefficientsType);
    /// Get `NB_COEFFICIENTS` array.
    fn get_coefficient(&self) -> Self::CoefficientsType;

    /// Get random sample points.
    fn samples(&self) -> &[P];
    /// Numbers of data
    fn data_len(&self) -> usize {
        self.samples().len()
    }
    /// Random numbers of indices by `NB_SAMPLE`.
    fn get_random_sample_id(&self) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let nb = self.data_len();
        use std::collections::HashSet;
        let mut set = HashSet::new();
        while set.len() < Self::NB_SAMPLE {
            set.insert(rng.gen_range(0..nb));
        }
        set.into_iter().collect()
    }
    /// Returns a distance list and uses `coefficients` to calculate the distance from data to the model.
    fn get_distance_to_model(&self, coefficients: &Self::CoefficientsType) -> Vec<T> {
        self.samples()
            .iter()
            .map(|&p| Self::compute_point_to_model(p, coefficients))
            .collect()
    }
    /// Return indices of distance between point and `coefficients` smaller than `tolerance`.
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
    /// Return numbers which between point and `coefficients` smaller than `tolerance`.
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
    /// Return distance between target `point` and `coefficients`.
    fn compute_point_to_model(p: P, coefficients: &Self::CoefficientsType) -> T;
    /// Get array of indices of samples.
    fn get_random_samples(&self) -> Self::SampleIdxType;
    /// Return `CoefficientsType` of samples.
    ///
    /// # Err
    /// * Numbers of data smaller than `NB_SAMPLE`.
    /// * Samples could not be computed.
    /// (ex: samples are overlay or parallel each other.)
    fn compute_model_coefficients(
        &self,
        samples: &Self::SampleIdxType,
    ) -> Result<Self::CoefficientsType, String>;
}
