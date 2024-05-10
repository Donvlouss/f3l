use f3l_core::{BasicFloat, SimpleSliceMath, serde::{self, Deserialize, Serialize}};
use std::marker::PhantomData;

use super::{ModelCoefficient, SacModel};

/// Compute a sphere
#[derive(Debug, Clone, Default)]
pub struct SacModelSphere<'a, P, T: BasicFloat>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    /// - center
    /// - radius
    pub coefficients: ([T; 3], T),
    data: Option<&'a [P]>,
    _value_type: PhantomData<T>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(crate="self::serde")]
pub struct SphereCoefficient<T: BasicFloat> {
    pub coefficients: ([T; 3], T),
}

impl<T: BasicFloat> ModelCoefficient for SphereCoefficient<T> {
    type CoefficientsType = ([T; 3], T);

    fn coe(&self) -> Self::CoefficientsType {
        self.coefficients
    }
}

impl<'a, P, T: BasicFloat> SacModelSphere<'a, P, T>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    pub fn new() -> Self {
        Self {
            coefficients: ([T::zero(); 3], T::zero()),
            data: None,
            _value_type: PhantomData,
        }
    }

    pub fn with_data(data: &'a [P]) -> Self {
        Self {
            coefficients: ([T::zero(); 3], T::zero()),
            data: Some(data),
            _value_type: PhantomData,
        }
    }
}

impl<'a, P, T: BasicFloat> SacModel<'a, P, T> for SacModelSphere<'a, P, T>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    type SampleIdxType = [usize; 4];

    type CoefficientsType = ([T; 3], T);

    /// 4 points be a tetrahedron.
    const NB_SAMPLE: usize = 4;
    /// center XYZ, radius
    const NB_COEFFICIENTS: usize = 4;

    fn compute_point_to_model(p: P, coefficients: &Self::CoefficientsType) -> T {
        let p: [T; 3] = p.into();
        (p.distance_between(&coefficients.0) - coefficients.1).abs()
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
        [sample[0], sample[1], sample[2], sample[3]]
    }

    /// Ref: [wiki](https://en.wikipedia.org/wiki/Sphere),
    /// [Wolfram](https://mathworld.wolfram.com/Circumsphere.html)
    fn compute_model_coefficients(
        &self,
        samples: &Self::SampleIdxType,
    ) -> Result<Self::CoefficientsType, String> {
        use f3l_core::glam::{Mat4, Vec4};

        let (mut mat, common_col) = if let Some(data) = self.data {
            let [p1, p2, p3, p4]: [[T; 3]; 4] = [
                data[samples[0]].into(),
                data[samples[1]].into(),
                data[samples[2]].into(),
                data[samples[3]].into(),
            ];
            let mat = Mat4::from_cols_array(&[
                p1[0].to_f32().unwrap(),
                p1[1].to_f32().unwrap(),
                p1[2].to_f32().unwrap(),
                1.,
                p2[0].to_f32().unwrap(),
                p2[1].to_f32().unwrap(),
                p2[2].to_f32().unwrap(),
                1.,
                p3[0].to_f32().unwrap(),
                p3[1].to_f32().unwrap(),
                p3[2].to_f32().unwrap(),
                1.,
                p4[0].to_f32().unwrap(),
                p4[1].to_f32().unwrap(),
                p4[2].to_f32().unwrap(),
                1.,
            ])
            .transpose();

            (
                mat,
                Vec4::new(
                    p1.len_squared().to_f32().unwrap(),
                    p2.len_squared().to_f32().unwrap(),
                    p3.len_squared().to_f32().unwrap(),
                    p4.len_squared().to_f32().unwrap(),
                ),
            )
        } else {
            return Err("Data corrupted.".to_owned());
        };
        // [p1, p2, p3, p4] transpose to
        // | p1.x p1.y p1.z 1 |
        //  ...
        let x = mat.x_axis;
        let y = mat.y_axis;
        let z = mat.z_axis;

        let common_div = 1. / mat.determinant();

        // | common_col.0 p1.y p1.z 1|
        mat.x_axis = common_col;
        let dx = mat.determinant();

        // | common_col.0 p1.x p1.z 1|
        mat.y_axis = x;
        let dy = -mat.determinant();

        // | common_col.0 p1.x p1.y 1|
        mat.z_axis = y;
        let dz = mat.determinant();

        // | common_col p1.x p1.y p1.z|
        mat.w_axis = z;
        let dp = mat.determinant();

        let center = [
            T::from(dx * common_div * 0.5).unwrap(),
            T::from(dy * common_div * 0.5).unwrap(),
            T::from(dz * common_div * 0.5).unwrap(),
        ];
        // let radius = center.distance_between(&p1);
        let radius = (center.len_squared() - T::from(dp * common_div).unwrap()).sqrt();

        Ok((center, radius))
    }
}
