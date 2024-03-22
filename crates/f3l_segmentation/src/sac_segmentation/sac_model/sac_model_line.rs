use f3l_core::{BasicFloat, SimpleSliceMath};
use std::marker::PhantomData;

use super::SacModel;

#[derive(Debug, Clone, Default)]
pub struct SacModelLine<'a, P, T: BasicFloat>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    /// - Point on Line
    /// - Direction
    pub coefficients: ([T; 3], [T; 3]),
    data: Option<&'a [P]>,
    _value_type: PhantomData<T>,
}

impl<'a, P, T: BasicFloat> SacModelLine<'a, P, T>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    pub fn new() -> Self {
        Self {
            coefficients: ([T::zero(); 3], [T::zero(); 3]),
            data: None,
            _value_type: PhantomData,
        }
    }

    pub fn with_data(data: &'a Vec<P>) -> Self {
        Self {
            coefficients: ([T::zero(); 3], [T::zero(); 3]),
            data: Some(data),
            _value_type: PhantomData,
        }
    }
}

impl<'a, P, T: BasicFloat> SacModel<'a, P, T> for SacModelLine<'a, P, T>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    type SampleIdxType = [usize; 2];

    type CoefficientsType = ([T; 3], [T; 3]);

    const NB_SAMPLE: usize = 2;

    const NB_COEFFICIENTS: usize = 6;

    /// Compute point to target line.<br>
    /// * P0: target point
    /// * P1: point of line
    /// * Dir: direction of line ( p2 - p1 )
    /// * PDir: norm(p1 - p0)<br>
    /// by: Parallelogram,
    /// cause ||PDir x Dir|| = ||Dir|| * ||PDir|| * sin(theta) = ||Dir|| * distance(P0 to line)<br>
    /// distance = ||PDir x Dir|| / ||Dir||, let ||Dir|| = 1, then distance = ||PDir x Dir||
    fn compute_point_to_model(p: P, coefficients: &Self::CoefficientsType) -> T {
        let (p0, (p1, dir)): ([T; 3], ([T; 3], [T; 3])) =
            (p.into(), (coefficients.0, coefficients.1));
        let p_dir = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let dir = dir.normalized();
        p_dir.cross(&dir).len()
    }

    fn set_data(&mut self, data: &'a [P]) {
        self.data = Some(data);
    }

    fn set_coefficient(&mut self, factor: &Self::CoefficientsType) {
        self.coefficients = *factor;
    }

    fn get_coefficient(&self) -> Self::CoefficientsType {
        (self.coefficients.0, self.coefficients.1)
    }

    fn samples(&self) -> &[P] {
        self.data.unwrap()
    }

    fn get_random_samples(&self) -> Self::SampleIdxType {
        let sample = self.get_random_sample_id();
        [sample[0], sample[1]]
    }

    /// Samples 0 as Point of line. norm(samples 1-samples 0) as direction
    fn compute_model_coefficients(
        &self,
        samples: &Self::SampleIdxType,
    ) -> Result<Self::CoefficientsType, String> {
        let [p0, p1] = samples;
        let (p0, p1): Self::CoefficientsType = if let Some(data) = self.data {
            (data[*p0].into(), data[*p1].into())
        } else {
            return Err("Data corrupted".to_owned());
        };
        let dir = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]].normalized();
        Ok((p0, dir))
    }
}
