use f3l_core::{
    apply_both, apply_each, project_vector, BasicFloat, SimpleSliceMath
};
use std::marker::PhantomData;

use super::SacModel;

#[derive(Debug, Clone, Default)]
pub struct SacModelCircle3d<'a, P, T: BasicFloat>
where
    P: Into<[T; 3]> + Clone + Copy
{
    /// - Point on Line
    /// - Direction
    pub coefficients: ([T; 3], [T; 3], T),
    data: Option<&'a Vec<P>>,
    _value_type: PhantomData<T>
}

impl<'a, P, T: BasicFloat> SacModel<'a, P, T> for SacModelCircle3d<'a, P, T>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    type DataType = P;

    type SampleIdxType = [usize; 3];

    type CoefficientsType = ([T; 3], [T; 3], T);

    const NB_SAMPLE:usize = 3;

    const NB_COEFFICIENTS: usize = 7;

    /// 1. compute length of p project to normal
    /// 2. compute proj_point on plane
    /// 3. compute vector of center to proj_plane, and which normalized
    /// 4. use normalized vector move center to round
    /// 5. compute distance between proj_point and point on round
    fn compute_point_to_model(p: P, coefficients: &Self::CoefficientsType) -> T {
        let (c, n, r) = *coefficients;
        let p: [T; 3] = p.into();
        let vec_pc = apply_both(&p, &c, std::ops::Sub::sub);
        let proj_vec = project_vector(&vec_pc, &n);

        // point project to plane of circle
        // let project = apply_both(&p, &proj_vec, std::ops::Add::add);
        let project = apply_both(&p, &proj_vec, std::ops::Sub::sub);
        // compute vector from center to project point
        let project_to_c = apply_both(&project, &c, std::ops::Sub::sub);
        let dir_to_proj = project_to_c.normalized();

        // point from center along dir_to_proj to circle
        let p_to_circle = apply_both(
            &c,
            &apply_each(&dir_to_proj, r, std::ops::Mul::mul),
            std::ops::Add::add);
        apply_both(&p, &p_to_circle, std::ops::Sub::sub).len()
    }

    fn set_data(&mut self, data: &'a Vec<P>) {
        self.data = Some(data);
    }

    fn set_coefficient(&mut self, factor: &Self::CoefficientsType) {
        self.coefficients = *factor;
    }

    fn get_coefficient(&self) -> Self::CoefficientsType {
        self.coefficients
    }

    fn samples(&self) -> &Vec<Self::DataType> {
        self.data.unwrap()
    }

    fn get_random_samples(&self) -> Self::SampleIdxType {
        let sample = self.get_random_sample_id();
        [sample[0], sample[1], sample[2]]
    }

    /// Ref: [Cartesian coordinates from cross- and dot-products](https://en.wikipedia.org/wiki/Circumcircle)
    fn compute_model_coefficients(&self, samples: &Self::SampleIdxType) -> Result<Self::CoefficientsType, String> {
        let [p1, p2, p3] = *samples;
        let (p1, p2, p3): ([T;3],[T;3],[T;3]) = if let Some(data) = self.data {
            (
                data[p1].into(),
                data[p2].into(),
                data[p3].into(),
            )
        } else {
            return Err("Data corrupted.".to_owned());
        };
        let v12 = apply_both(&p1, &p2, std::ops::Sub::sub);
        let v21 = apply_both(&p2, &p1, std::ops::Sub::sub);
        let v13 = apply_both(&p1, &p3, std::ops::Sub::sub);
        let v31 = apply_both(&p3, &p1, std::ops::Sub::sub);
        let v23 = apply_both(&p2, &p3, std::ops::Sub::sub);
        let v32 = apply_both(&p3, &p2, std::ops::Sub::sub);
        
        let normal = v12.cross(&v23);
        let common_divided = T::one() / T::from(2.).unwrap() * normal.len().powi(2);

        let alpha = (v23.len().powi(2) * v12.dot(&v13)) * common_divided;
        let beta = (v13.len().powi(2) * v21.dot(&v23)) * common_divided;
        let gamma = (v12.len().powi(2) * v31.dot(&v32)) * common_divided;

        let mut pc = [T::zero(); 3];
        (0..3).for_each(|i| pc[i] = alpha * p1[i] + beta * p2[i] + gamma * p3[i]);

        let radius = apply_both(&pc, &p1, std::ops::Sub::sub).len();

        Ok(
            (pc.into(), normal.normalized().into(), radius)
        )
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
                .filter(|&i| Self::compute_point_to_model(data[i], coefficients) < tolerance)
                .collect()
        } else {
            vec![]
        }
    }

    fn count_indices_within_tolerance(&self, coefficients: &Self::CoefficientsType, tolerance: T) -> usize {
        if let Some(data) = self.data {
            (0..data.len())
                .filter(|&i| Self::compute_point_to_model(data[i], coefficients) < tolerance)
                .map(|_| 1)
                .sum()
        } else {
            0
        }
    }
}