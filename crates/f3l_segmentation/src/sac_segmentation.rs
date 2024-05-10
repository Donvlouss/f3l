use f3l_core::{serde::{self, Deserialize, Serialize}, BasicFloat};

use crate::sac_algorithm::{SacAlgorithmGetSet, SacAlgorithm};

pub mod sac_algorithm;
pub mod sac_model;

use sac_model::*;

/// A Factory of Segmentation.
/// 
/// ```rust
/// use f3l_segmentation::sac_algorithm::*;
/// use f3l_segmentation::sac_model::*;
/// use f3l_segmentation::SacSegment;
/// use f3l_core::round_n;
/// 
/// let ps = vec![
///     [0f32, 0., 0.],
///     [1., 0., 0.],
///     [0., 1., 0.],
/// ];
/// let mut sac = SacSegment {
///     model: PlaneCoefficient::<f32>::default(),
///     ..Default::default()
/// };
/// sac.compute(&ps);
/// 
/// let coe = sac.model.coefficients;
/// assert_eq!(round_n(coe[0], 4), 0.);
/// assert_eq!(round_n(coe[1], 4), 0.);
/// assert_eq!(round_n(coe[2], 4).abs(), 1.);
/// assert_eq!(round_n(coe[3], 4), 0.);
/// ```
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(crate="self::serde")]
pub struct SacSegment<M: sac_model::ModelCoefficient> {
    pub model: M,
    pub algorithm: sac_algorithm::SacAlgorithmType,
    pub algorithm_parameter: sac_algorithm::SacAlgorithmParameter,
    pub inliers: Vec<usize>
}

macro_rules! impl_compute {
    ($segment:ident, $model_type:ty, $model_struct:ident) => {
        impl<T: BasicFloat> $segment<$model_type> {
            pub fn compute<'a, P>(&mut self, data: &'a [P]) -> bool
            where
                P: Into<[T; 3]> + Clone + Copy + Sync,
            {
                let mut algorithm = match self.algorithm {
                    sac_algorithm::SacAlgorithmType::RANSAC => 
                        sac_algorithm::SacRansac::with_parameter(self.algorithm_parameter),
                };
                let mut model = $model_struct::with_data(data);
                assert!($model_struct::<'a, P, T>::NB_SAMPLE <= data.len());
                let b = algorithm.compute(&mut model);
                self.model.coefficients = model.get_coefficient();
                b
            }
        }
    };
}

impl_compute!(SacSegment, PlaneCoefficient<T>, SacModelPlane);
impl_compute!(SacSegment, LineCoefficient<T>, SacModelLine);
impl_compute!(SacSegment, Circle3dCoefficient<T>, SacModelCircle3d);
impl_compute!(SacSegment, SphereCoefficient<T>, SacModelSphere);

#[test]
fn segment_plane() {
    use f3l_core::round_n;

    let ps = vec![
        [0f32, 0., 0.],
        [1., 0., 0.],
        [0., 1., 0.],
    ];
    let mut sac = SacSegment {
        model: PlaneCoefficient::<f32>::default(),
        ..Default::default()
    };
    sac.compute(&ps);

    let coe = sac.model.coefficients;
    assert_eq!(round_n(coe[0], 4), 0.);
    assert_eq!(round_n(coe[1], 4), 0.);
    assert_eq!(round_n(coe[2], 4).abs(), 1.);
    assert_eq!(round_n(coe[3], 4), 0.);
}

#[test]
fn segment_line() {
    use f3l_core::round_slice_n;
    let ps = vec![
        [0f32, 0., 0.],
        [1., 0., 0.],
    ];
    let mut sac = SacSegment {
        model: LineCoefficient::<f32>::default(),
        ..Default::default()
    };
    sac.compute(&ps);

    let coe = sac.model.coefficients;

    assert!(
        (
            round_slice_n(coe.0, 4) == ps[0]
            && round_slice_n(coe.1, 4) == ps[1]
        )
        || (
            round_slice_n(coe.0, 4) == ps[1]
            && round_slice_n(coe.1, 4) == [-1., 0., 0f32]
        )
    )
}

#[test]
fn segment_circle3d() {
    use f3l_core::{round_n, round_slice_n};

    let ps = vec![
        [-5f32, 0., 0.],
        [5., 0., 0.],
        [0., 5., 0.],
    ];
    let mut sac = SacSegment {
        model: Circle3dCoefficient::<f32>::default(),
        ..Default::default()
    };
    sac.compute(&ps);

    let (center, normal, radius) = sac.model.coefficients;

    assert_eq!(round_slice_n(center, 4), [0f32, 0., 0.]);
    assert!(
        round_slice_n(normal, 4) == [0f32, 0., 1.]
        || round_slice_n(normal, 4) == [0f32, 0., -1.]
    );
    assert_eq!(round_n(radius, 4), radius);
}

#[test]
fn segment_sphere() {
    use f3l_core::{round_n, round_slice_n};

    let ps = vec![
        [-5f32, 0., 0.],
        [5., 0., 0.],
        [0., 5., 0.],
        [0., 0., 5.],
    ];
    let mut sac = SacSegment {
        model: SphereCoefficient::<f32>::default(),
        ..Default::default()
    };
    sac.compute(&ps);

    let (center, radius) = sac.model.coefficients;

    assert_eq!(round_slice_n(center, 4), [0f32, 0., 0.]);
    assert_eq!(round_n(radius, 4), radius);
}