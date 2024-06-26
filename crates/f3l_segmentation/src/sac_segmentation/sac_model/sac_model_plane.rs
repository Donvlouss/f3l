use super::{ModelCoefficient, SacModel};
use f3l_core::{
    apply_both,
    serde::{self, Deserialize, Serialize},
    BasicFloat, SimpleSliceMath,
};

/// Compute a 3d plane model.
/// Any 3 points(not overlay or parallel) span a plane.
///
/// Coefficients: `coefficients_0` x + `coefficients_1` y + `coefficients_2` z + `coefficients_3` = 0
#[derive(Debug, Clone, Default)]
pub struct SacModelPlane<'a, P, T: BasicFloat>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    pub coefficients: [T; 4],
    data: Option<&'a [P]>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct PlaneCoefficient<T: BasicFloat> {
    pub coefficients: [T; 4],
}

impl<T: BasicFloat> ModelCoefficient for PlaneCoefficient<T> {
    type CoefficientsType = [T; 4];

    fn coe(&self) -> Self::CoefficientsType {
        self.coefficients
    }
}

impl<'a, P, T: BasicFloat> SacModelPlane<'a, P, T>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    pub fn new() -> Self {
        Self {
            coefficients: [T::zero(); 4],
            data: None,
        }
    }

    pub fn with_data(data: &'a [P]) -> Self {
        Self {
            coefficients: [T::zero(); 4],
            data: Some(data),
        }
    }
}

impl<'a, P, T: BasicFloat> SacModel<'a, P, T> for SacModelPlane<'a, P, T>
where
    P: Into<[T; 3]> + Clone + Copy,
{
    type SampleIdxType = [usize; 3];

    type CoefficientsType = [T; 4];

    /// Any 3 points(not overlay or parallel) span a plane.
    const NB_SAMPLE: usize = 3;

    /// `coefficients_0` x + `coefficients_1` y + `coefficients_2` z + `coefficients_3` = 0
    const NB_COEFFICIENTS: usize = 4;

    fn compute_point_to_model(p: P, coefficients: &Self::CoefficientsType) -> T {
        let p: [T; 3] = p.into();
        let p = [p[0], p[1], p[2], T::one()];
        (p.dot(coefficients)).abs()
    }

    fn set_data(&mut self, data: &'a [P]) {
        self.data = Some(data);
    }

    fn set_coefficient(&mut self, factor: &Self::CoefficientsType) {
        self.coefficients = *factor;
    }

    fn get_coefficient(&self) -> Self::CoefficientsType {
        [
            self.coefficients[0],
            self.coefficients[1],
            self.coefficients[2],
            self.coefficients[3],
        ]
    }

    fn samples(&self) -> &[P] {
        self.data.unwrap()
    }

    fn get_random_samples(&self) -> Self::SampleIdxType {
        let sample = self.get_random_sample_id();
        [sample[0], sample[1], sample[2]]
    }

    fn compute_model_coefficients(
        &self,
        samples: &Self::SampleIdxType,
    ) -> Result<Self::CoefficientsType, String> {
        let [p0, p1, p2] = *samples;
        let [p0, p1, p2]: [[T; 3]; 3] = if let Some(data) = self.data {
            [data[p0].into(), data[p1].into(), data[p2].into()]
        } else {
            return Err("Data Empty".to_owned());
        };
        let p1p0 = apply_both(&p1, &p0, std::ops::Sub::sub);
        let p2p0 = apply_both(&p2, &p0, std::ops::Sub::sub);
        let dpp = apply_both(&p2p0, &p1p0, std::ops::Div::div);
        if dpp[0] == dpp[1] && dpp[1] == dpp[2] {
            return Err("Parallel or overlay".to_owned());
        }
        let coefficient = [
            p1p0[1] * p2p0[2] - p1p0[2] * p2p0[1],
            p1p0[2] * p2p0[0] - p1p0[0] * p2p0[2],
            p1p0[0] * p2p0[1] - p1p0[1] * p2p0[0],
            T::zero(),
        ];
        let mut coefficient = coefficient.normalized();
        coefficient[3] =
            -T::one() * (coefficient[0] * p0[0] + coefficient[1] * p0[1] + coefficient[2] * p0[2]);

        Ok(coefficient)
    }
}
