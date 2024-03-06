use f3l_core::{
    BasicFloat,
    SimpleSliceMath
};
use std::marker::PhantomData;

use super::SacModel;

#[derive(Debug, Clone, Default)]
pub struct SacModelLine<'a, P, T: BasicFloat>
where
    P: Into<[T; 3]> + Clone + Copy
{
    /// - Point on Line
    /// - Direction
    pub coefficients: ([T; 3], [T; 3]),
    data: Option<&'a Vec<P>>,
    _value_type: PhantomData<T>
}

impl<'a, P, T: BasicFloat> SacModelLine<'a, P, T>
where
    P: Into<[T; 3]> + Clone + Copy
{
    pub fn new() -> Self {
        Self {
            coefficients: ([T::zero(); 3], [T::zero(); 3]),
            data: None,
            _value_type: PhantomData::default()
        }
    }

    pub fn with_data(data: &'a Vec<P>) -> Self {
        Self {
            coefficients: ([T::zero(); 3], [T::zero(); 3]),
            data: Some(data),
            _value_type: PhantomData::default()
        }
    }
}

impl<'a, P, T: BasicFloat> SacModel<'a, P, T> for SacModelLine<'a, P, T>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    type DataType = P;

    type SampleIdxType = [usize; 2];

    type CoefficientsType = ([T; 3], [T; 3]);

    const NB_SAMPLE:usize = 2;

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
        let (p0, (p1, dir)): ([T; 3], ([T;3], [T;3])) = (p.into(), (coefficients.0.into(), coefficients.1.into()));
        let p_dir = [p1[0]-p0[0], p1[1]-p0[1], p1[2]-p0[2]];
        let dir = dir.normalized();     
        p_dir.cross(&dir).len()
    }

    fn set_data(&mut self, data: &'a Vec<P>) {
        self.data = Some(data);
    }

    fn set_coefficient(&mut self, factor: &Self::CoefficientsType) {
        self.coefficients = *factor;
    }

    fn get_coefficient(&self) -> Self::CoefficientsType {
        (
            self.coefficients.0.into(),
            self.coefficients.1.into(),
        )
    }

    fn samples(&self) -> &Vec<Self::DataType> {
        self.data.unwrap()
    }

    fn get_random_samples(&self) -> Self::SampleIdxType {
        let sample = self.get_random_sample_id();
        [sample[0], sample[1]]
    }

    /// Samples 0 as Point of line. norm(samples 1-samples 0) as direction
    fn compute_model_coefficients(&self, samples: &Self::SampleIdxType) -> Result<Self::CoefficientsType, String> {
        let [p0, p1] = samples;
        let (p0, p1): Self::CoefficientsType = if let Some(data) = self.data {
            (
                data[*p0].into(),
                data[*p1].into()
            )
        } else {
            return Err("Data corrupted".to_owned());
        };
        let dir = [p1[0]-p0[0], p1[1]-p0[1], p1[2]-p0[2]].normalized();
        Ok((p0.into(), dir.into()))
    }

    fn get_distance_to_model(&self, coefficients: &Self::CoefficientsType) -> Vec<T> {
        if let Some(data) = self.data {
            data.iter()
                .map(|&p| {
                    Self::compute_point_to_model(p, coefficients)
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn select_indices_within_tolerance(&self, coefficients: &Self::CoefficientsType, tolerance: T) -> Vec<usize> {
        if let Some(data) = self.data {
            (0..data.len())
                .filter(|&i| {
                    let d = Self::compute_point_to_model(data[i], coefficients);
                    d  <= tolerance
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn count_indices_within_tolerance(&self, coefficients: &Self::CoefficientsType, tolerance: T) -> usize {
        if let Some(data) = self.data {
            (0..data.len())
                .filter(|&i| Self::compute_point_to_model(data[i], coefficients) <= tolerance)
                .sum()
        } else {
            0
        }
    }
}