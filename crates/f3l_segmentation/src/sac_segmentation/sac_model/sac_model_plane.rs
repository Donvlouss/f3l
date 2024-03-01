use f3l_core::BasicFloat;
use std::marker::PhantomData;
use glam::{
    Vec3,
    Vec4
};

use super::SacModel;



#[derive(Debug, Clone, Default)]
pub struct SacModelPlane<'a, P, T: BasicFloat>
where
    P: Into<[T; 3]> + Clone + Copy
{
    pub coefficients: Vec4,
    data: Option<&'a Vec<P>>,
    _value_type: PhantomData<T>
}

impl<'a, P, T: BasicFloat> SacModelPlane<'a, P, T>
where
    P: Into<[T; 3]> + Clone + Copy
{
    pub fn new() -> Self {
        Self {
            coefficients: Vec4::ZERO,
            data: None,
            _value_type: PhantomData::default()
        }
    }

    pub fn with_data(data: &'a Vec<P>) -> Self {
        Self {
            coefficients: Vec4::ZERO,
            data: Some(data),
            _value_type: PhantomData::default()
        }
    }
}

impl<'a, P, T: BasicFloat> SacModel<'a, P, T> for SacModelPlane<'a, P, T>
where
    P: Into<[T; 3]> + Clone + Copy + Into<Vec3>,
{
    type DataType = P;

    type SampleIdxType = [usize; 3];

    type CoefficientsIdxType = [f32; 4];

    const NB_SAMPLE:usize = 3;

    const NB_COEFFICIENTS: usize = 4;

    fn set_data(&mut self, data: &'a Vec<P>) {
        self.data = Some(data);
    }

    fn set_coefficient(&mut self, factor: &Self::CoefficientsIdxType) {
        self.coefficients = (*factor).into();
    }

    fn get_coefficient(&self) -> Self::CoefficientsIdxType {
        [
            self.coefficients[0],
            self.coefficients[1],
            self.coefficients[2],
            self.coefficients[3],
        ]
    }

    fn samples(&self) -> &Vec<Self::DataType> {
        self.data.unwrap()
    }
    
    fn get_random_samples(&self) -> Self::SampleIdxType {
        let sample = self.get_random_sample_id();
        [sample[0], sample[1], sample[2]]
    }

    fn compute_model_coefficients(&self, samples: &Self::SampleIdxType) -> Result<Self::CoefficientsIdxType, String> {
        let [p0, p1, p2] = *samples;
        let (p0, p1, p2) = if let Some(data) = self.data {
            let p0: Vec3 = data[p0].into();
            let p1: Vec3 = data[p1].into();
            let p2: Vec3 = data[p2].into();
            (p0, p1, p2)
        } else {
            return Err("Data Empty".to_owned());
        };
        let p1p0 = p1 - p0;
        let p2p0 = p2 - p0;
        let dpp = p2p0 / p1p0;
        if dpp[0] == dpp[1] && dpp[1] == dpp[2] {
            return Err("Parallel or overlay".to_owned());
        }
        let coefficient = Vec4::new(
            p1p0[1] * p2p0[2] - p1p0[2] * p2p0[1],
            p1p0[2] * p2p0[0] - p1p0[0] * p2p0[2],
            p1p0[0] * p2p0[1] - p1p0[1] * p2p0[0],
            0.
        );
        let mut coefficient = coefficient.normalize();
        coefficient[3] = -1. * (
            coefficient[0] * p0[0] +
            coefficient[1] * p0[1] +
            coefficient[2] * p0[2]
        );

        let v: [f32; 4] = coefficient.into();
        Ok(v)
    }

    fn get_distance_to_model(&self, coefficients: &Self::CoefficientsIdxType) -> Vec<T> {
        let factor: Vec4 = (*coefficients).into();
        if let Some(data) = self.data {
            data.iter()
                .map(|p| {
                    let p: Vec4 = (<P as Into<Vec3>>::into(*p), 0f32).into();
                    T::from(factor.dot(p).abs()).unwrap()
                })
                .collect::<Vec<_>>()
        } else{
            vec![]
        }
    }

    fn select_indices_within_tolerance(&self, coefficients: &Self::CoefficientsIdxType, tolerance: T) -> Vec<usize> {
        let factor: Vec4 = (*coefficients).into();
        if let Some(data) = self.data {
            data.iter()
                .enumerate()
                .filter_map(|(i, p)| {
                    let p: Vec4 = (<P as Into<Vec3>>::into(*p), 0f32).into();
                    if T::from(factor.dot(p).abs()).unwrap() < tolerance {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    }

    fn count_indices_within_tolerance(&self, coefficients: &Self::CoefficientsIdxType, tolerance: T) -> usize {
        let factor: Vec4 = (*coefficients).into();
        if let Some(data) = self.data {
            data.iter()
                .map(|p| {
                    let p: Vec4 = (<P as Into<Vec3>>::into(*p), 1f32).into();
                    let v = T::from(factor.dot(p).abs()).unwrap();
                    if v < tolerance {
                        1
                    } else {
                        0
                    }
                })
                .sum()
        } else {
            0
        }
    }
}