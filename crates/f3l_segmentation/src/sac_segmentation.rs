use f3l_core::{
    serde::{self, Deserialize, Serialize},
    BasicFloat,
};

use crate::sac_algorithm::{SacAlgorithm, SacAlgorithmGetSet};

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
#[serde(crate = "self::serde")]
pub struct SacSegment<M: sac_model::ModelCoefficient> {
    pub model: M,
    pub algorithm: sac_algorithm::SacAlgorithmType,
    pub algorithm_parameter: sac_algorithm::SacAlgorithmParameter,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub inliers: Vec<usize>,
}

macro_rules! impl_compute {
    ($segment:ident, $model_type:ty, $model_struct:ident) => {
        impl<T: BasicFloat> $segment<$model_type> {
            pub fn compute<'a, P>(&mut self, data: &'a [P]) -> bool
            where
                P: Into<[T; 3]> + Clone + Copy + Sync,
            {
                let mut algorithm = match self.algorithm {
                    sac_algorithm::SacAlgorithmType::RANSAC => {
                        sac_algorithm::SacRansac::with_parameter(self.algorithm_parameter)
                    }
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

    let ps = vec![[0f32, 0., 0.], [1., 0., 0.], [0., 1., 0.]];
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
    let ps = vec![[0f32, 0., 0.], [1., 0., 0.]];
    let mut sac = SacSegment {
        model: LineCoefficient::<f32>::default(),
        ..Default::default()
    };
    sac.compute(&ps);

    let coe = sac.model.coefficients;

    assert!(
        (round_slice_n(coe.0, 4) == ps[0] && round_slice_n(coe.1, 4) == ps[1])
            || (round_slice_n(coe.0, 4) == ps[1] && round_slice_n(coe.1, 4) == [-1., 0., 0f32])
    )
}

#[test]
fn segment_circle3d() {
    use f3l_core::{round_n, round_slice_n};

    let ps = vec![[-5f32, 0., 0.], [5., 0., 0.], [0., 5., 0.]];
    let mut sac = SacSegment {
        model: Circle3dCoefficient::<f32>::default(),
        ..Default::default()
    };
    sac.compute(&ps);

    let (center, normal, radius) = sac.model.coefficients;

    assert_eq!(round_slice_n(center, 4), [0f32, 0., 0.]);
    assert!(
        round_slice_n(normal, 4) == [0f32, 0., 1.] || round_slice_n(normal, 4) == [0f32, 0., -1.]
    );
    assert_eq!(round_n(radius, 4), radius);
}

#[test]
fn segment_sphere() {
    use f3l_core::{round_n, round_slice_n};

    let ps = vec![[-5f32, 0., 0.], [5., 0., 0.], [0., 5., 0.], [0., 0., 5.]];
    let mut sac = SacSegment {
        model: SphereCoefficient::<f32>::default(),
        ..Default::default()
    };
    sac.compute(&ps);

    let (center, radius) = sac.model.coefficients;

    assert_eq!(round_slice_n(center, 4), [0f32, 0., 0.]);
    assert_eq!(round_n(radius, 4), radius);
}

mod test_serde {
    #[allow(unused_imports)]
    use super::*;

    #[allow(unused_macros)]
    macro_rules! serde_convert {
        ($target: expr, $text: ident, $type: ty) => {
            let sac = $target;
            let sac_serde: $type = serde_json::from_str($text).unwrap();

            assert_eq!(sac.model.coefficients, sac_serde.model.coefficients);
            assert_eq!(sac.algorithm, sac_serde.algorithm);
            assert_eq!(sac.algorithm_parameter, sac_serde.algorithm_parameter);
        };
    }

    #[test]
    fn serde_plane() {
        let text = r#"{
            "model":{
                "coefficients":[0.0,0.0,0.0,0.0]},
                "algorithm":"RANSAC",
                "algorithm_parameter":{
                    "probability":0.99,
                    "threshold":0.1,
                    "max_iterations":1000,
                    "threads":1
                }
            }"#;

        serde_convert!(
            SacSegment {
                model: PlaneCoefficient::<f32>::default(),
                ..Default::default()
            },
            text,
            SacSegment<PlaneCoefficient<f32>>
        );
    }

    #[test]
    fn serde_line() {
        let text = r#"{
            "model":{
                "coefficients":[
                    [0.0,0.0,0.0],
                    [0.0,0.0,0.0]
                    ]
                },
                "algorithm":"RANSAC",
                "algorithm_parameter":{
                    "probability":0.99,
                    "threshold":0.1,
                    "max_iterations":1000,
                    "threads":1
                }
            }"#;

        serde_convert!(
            SacSegment {
                model: LineCoefficient::<f32>::default(),
                ..Default::default()
            },
            text,
            SacSegment<LineCoefficient<f32>>
        );
    }

    #[test]
    fn serde_circle3d() {
        let text = r#"{
            "model":{
                "coefficients":[
                    [0.0,0.0,0.0],
                    [0.0,0.0,0.0],
                    0.0
                    ]
                },
                "algorithm":"RANSAC",
                "algorithm_parameter":{
                    "probability":0.99,
                    "threshold":0.1,
                    "max_iterations":1000,
                    "threads":1
                }
            }"#;
        serde_convert!(
            SacSegment {
                model: Circle3dCoefficient::<f32>::default(),
                ..Default::default()
            },
            text,
            SacSegment<Circle3dCoefficient<f32>>
        );
    }

    #[test]
    fn serde_sphere() {
        let text = r#"{
            "model":{
                "coefficients":[
                    [0.0,0.0,0.0],
                    0.0]
                },
                "algorithm":"RANSAC",
                "algorithm_parameter":{
                    "probability":0.99,
                    "threshold":0.1,
                    "max_iterations":1000,
                    "threads":1
                }
            }"#;
        serde_convert!(
            SacSegment {
                model: SphereCoefficient::<f32>::default(),
                ..Default::default()
            },
            text,
            SacSegment<SphereCoefficient<f32>>
        );
    }
}
