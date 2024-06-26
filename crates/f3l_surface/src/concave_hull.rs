use f3l_core::{
    apply_each,
    serde::{self, Deserialize, Serialize},
    SimpleSliceMath,
};
use f3l_segmentation::{
    sac_algorithm::{SacAlgorithm, SacAlgorithmParameter},
    sac_model::SacModel,
};

use crate::{Delaunay2D, Delaunay2DShape};

/// Compute Concave Hull of data.
///
/// Generally concave hull is only for 2d. When data is 3d,
/// would use [`f3l_segmentation::sac_model::SacModelPlane`] to find plane,
/// then use normal of plane rotate cloud to XY dimension and ignore `Z`.
/// Finally, compute 2d concave hull.
///
/// ```rust
/// use f3l_core::glam::Vec2;
/// let p2 = [Vec2::ZERO, Vec2::X, Vec2::Y];
/// let p3 = [[1_f32, 0., 0.], [0., 1., 0.], [0., 0., 1.]];
///
/// let mut concave2d = ConcaveHull::<2>::new();
/// let d2 = concave2d.compute(&p2, 1.0);
/// assert_eq!(d2[0].mesh[0].point, [2, 0, 1]);
///
/// let mut concave3d = ConcaveHull::<3>::new();
/// let d3 = concave3d.compute(&p3, 1.0);
/// assert_eq!(d3[0].mesh[0].point, [2, 0, 1]);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct ConcaveHull<const D: usize> {
    pub dim: usize,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub shapes: Vec<Delaunay2DShape>,
}

impl<const D: usize> ConcaveHull<D> {
    pub fn new() -> Self {
        Self {
            dim: D,
            shapes: vec![],
        }
    }
}

impl ConcaveHull<2> {
    pub fn compute<T: f3l_core::BasicFloat, P>(
        &mut self,
        data: &Vec<P>,
        alpha: T,
    ) -> Vec<Delaunay2DShape>
    where
        P: Into<[T; 2]> + Copy + std::ops::Index<usize, Output = T>,
        [T; 2]: Into<P>,
    {
        let mut delaunay = Delaunay2D::new(data);
        delaunay.compute(alpha);
        delaunay.shapes
    }
}

impl ConcaveHull<3> {
    /// Compute with default [`SacAlgorithmParameter`].
    ///
    /// See [`ConcaveHull::compute_with_parameter`]
    pub fn compute<T: f3l_core::BasicFloat, P>(
        &mut self,
        data: &[P],
        alpha: T,
    ) -> Vec<Delaunay2DShape>
    where
        P: Into<[T; 3]> + Copy + std::ops::Index<usize, Output = T> + Send + Sync,
        [T; 3]: Into<P>,
    {
        self.compute_with_parameter(data, alpha, SacAlgorithmParameter::default())
    }

    /// Compute concave hull with parameter of plane.
    ///
    /// 1. Find plane of data.
    /// 2. Compute `Axis Angle` of `Normal` to `+Z`, then rotate data, get `XY`, ignore `Z`.
    /// 3. Compute Concave Hull of 2d.
    pub fn compute_with_parameter<T: f3l_core::BasicFloat, P>(
        &mut self,
        data: &[P],
        alpha: T,
        parameter: SacAlgorithmParameter,
    ) -> Vec<Delaunay2DShape>
    where
        P: Into<[T; 3]> + Copy + std::ops::Index<usize, Output = T> + Send + Sync,
        [T; 3]: Into<P>,
    {
        use f3l_core::glam::{Mat3A, Vec3, Vec3A};
        let mut model = f3l_segmentation::sac_model::SacModelPlane::with_data(data);
        let mut algorithm = f3l_segmentation::sac_algorithm::SacRansac {
            parameter,
            inliers: vec![],
        };
        algorithm.compute(&mut model);
        let mut coefficients = model.get_coefficient();
        if coefficients[2] < T::zero() {
            coefficients = apply_each(&coefficients, -T::one(), std::ops::Mul::mul);
        }
        let axis = [coefficients[0], coefficients[1], coefficients[2]].cross(&[
            T::zero(),
            T::zero(),
            T::one(),
        ]);
        let angle = [coefficients[0], coefficients[1], coefficients[2]].compute_angle(&[
            T::zero(),
            T::zero(),
            T::one(),
        ]);

        let mat = Mat3A::from_axis_angle(
            Vec3::new(
                axis[0].to_f32().unwrap(),
                axis[1].to_f32().unwrap(),
                axis[2].to_f32().unwrap(),
            ),
            angle.to_f32().unwrap(),
        );
        let points = data
            .iter()
            .map(|p| {
                let p = mat
                    * Vec3A::new(
                        p[0].to_f32().unwrap(),
                        p[1].to_f32().unwrap(),
                        p[2].to_f32().unwrap(),
                    );
                [p[0], p[1]]
            })
            .collect::<Vec<_>>();

        let mut delaunay = Delaunay2D::new(&points);
        delaunay.compute(alpha.to_f32().unwrap());
        delaunay.shapes
    }
}

#[test]
fn test() {
    use f3l_core::glam::Vec2;
    let p2 = vec![Vec2::ZERO, Vec2::X, Vec2::Y];
    let p3 = vec![[1_f32, 0., 0.], [0., 1., 0.], [0., 0., 1.]];

    let mut concave2d = ConcaveHull::<2>::new();
    let d2 = concave2d.compute(&p2, 1.0);
    assert_eq!(d2[0].mesh[0].point, [2, 0, 1]);

    let mut concave3d = ConcaveHull::<3>::new();
    let d3 = concave3d.compute(&p3, 1.0);
    assert_eq!(d3[0].mesh[0].point, [2, 0, 1]);
}

#[test]
fn serde() {
    use f3l_core::glam::Vec2;
    let p2 = vec![Vec2::ZERO, Vec2::X, Vec2::Y];
    let p3 = vec![[1_f32, 0., 0.], [0., 1., 0.], [0., 0., 1.]];

    let mut concave2d: ConcaveHull<2> = serde_json::from_str(r#"{"dim":2}"#).unwrap();
    let mut concave3d: ConcaveHull<3> = serde_json::from_str(r#"{"dim":3}"#).unwrap();

    let d2 = concave2d.compute(&p2, 1.0);
    assert_eq!(d2[0].mesh[0].point, [2, 0, 1]);

    let d3 = concave3d.compute(&p3, 1.0);
    assert_eq!(d3[0].mesh[0].point, [2, 0, 1]);
}
