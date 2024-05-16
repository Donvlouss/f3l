use f3l_core::{
    apply_both, apply_each, find_circle, project_vector,
    serde::{self, Deserialize, Serialize},
    BasicFloat, SimpleSliceMath,
};
use std::marker::PhantomData;

use super::{ModelCoefficient, SacModel};

/// Compute a 3d circle, not sphere.
#[derive(Debug, Clone, Default)]
pub struct SacModelCircle3d<'a, P, T: BasicFloat>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    /// - Center of Circle
    /// - Normal of Circle plane
    /// - Radius
    pub coefficients: ([T; 3], [T; 3], T),
    data: Option<&'a [P]>,
    _value_type: PhantomData<T>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct Circle3dCoefficient<T: BasicFloat> {
    pub coefficients: ([T; 3], [T; 3], T),
}

impl<T: BasicFloat> ModelCoefficient for Circle3dCoefficient<T> {
    type CoefficientsType = ([T; 3], [T; 3], T);

    fn coe(&self) -> Self::CoefficientsType {
        self.coefficients
    }
}

impl<'a, P, T: BasicFloat> SacModelCircle3d<'a, P, T>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    pub fn new() -> Self {
        Self {
            coefficients: ([T::zero(); 3], [T::zero(); 3], T::zero()),
            data: None,
            _value_type: PhantomData,
        }
    }

    pub fn with_data(data: &'a [P]) -> Self {
        Self {
            coefficients: ([T::zero(); 3], [T::zero(); 3], T::zero()),
            data: Some(data),
            _value_type: PhantomData,
        }
    }
}

impl<'a, P, T: BasicFloat> SacModel<'a, P, T> for SacModelCircle3d<'a, P, T>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    type SampleIdxType = [usize; 3];

    type CoefficientsType = ([T; 3], [T; 3], T);

    /// 3 points be a circle3d.
    const NB_SAMPLE: usize = 3;
    /// center XYZ, normal XYZ, radius.
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
            std::ops::Add::add,
        );
        apply_both(&p, &p_to_circle, std::ops::Sub::sub).len()
    }

    fn set_data(&mut self, data: &'a [P]) {
        self.data = Some(data);
    }

    fn set_coefficient(&mut self, factor: &Self::CoefficientsType) {
        self.coefficients = *factor;
    }

    fn get_coefficient(&self) -> Self::CoefficientsType {
        self.coefficients
    }

    fn samples(&self) -> &[P] {
        self.data.unwrap()
    }

    fn get_random_samples(&self) -> Self::SampleIdxType {
        let sample = self.get_random_sample_id();
        [sample[0], sample[1], sample[2]]
    }

    /// Ref: [Cartesian coordinates from cross- and dot-products](https://en.wikipedia.org/wiki/Circumcircle)
    fn compute_model_coefficients(
        &self,
        samples: &Self::SampleIdxType,
    ) -> Result<Self::CoefficientsType, String> {
        let [p1, p2, p3] = *samples;
        let (p1, p2, p3): ([T; 3], [T; 3], [T; 3]) = if let Some(data) = self.data {
            (data[p1].into(), data[p2].into(), data[p3].into())
        } else {
            return Err("Data corrupted.".to_owned());
        };

        let (pc, normal, radius) = find_circle(&[p1, p2, p3]);

        Ok((pc, normal.normalized(), radius))
    }

    fn get_distance_to_model(&self, coefficients: &Self::CoefficientsType) -> Vec<T> {
        if let Some(data) = self.data {
            data.iter()
                .map(|&p| Self::compute_point_to_model(p, coefficients))
                .collect()
        } else {
            vec![]
        }
    }
}
