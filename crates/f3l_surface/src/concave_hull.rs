use f3l_core::{apply_each, SimpleSliceMath};
use f3l_segmentation::{sac_algorithm::{SacAlgorithm, SacAlgorithmParameter}, sac_model::SacModel};

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
/// let mut concave2d = ConcaveHull::new_2d(&p2);
/// let d2 = concave2d.compute(1.0);
/// assert_eq!(d2[0].mesh[0].point, [2, 0, 1]);
/// 
/// let mut concave3d = ConcaveHull::new_3d(&p3);
/// let d3 = concave3d.compute(1.0);
/// assert_eq!(d3[0].mesh[0].point, [2, 0, 1]);
/// ```
#[derive(Debug, Clone)]
pub struct ConcaveHull<'a, T: f3l_core::BasicFloat, P, const D: usize>
where
    P: Into<[T; D]> + Copy + std::ops::Index<usize, Output = T>,
{
    data: &'a [P],
}

impl<'a, T: f3l_core::BasicFloat, P> ConcaveHull<'a, T, P, 2>
where
    P: Into<[T; 2]> + Copy + std::ops::Index<usize, Output = T>,
    [T; 2]: Into<P>,
{
    pub fn new_2d(data: &'a [P]) -> Self {
        Self {
            data
        }
    }

    pub fn compute(&mut self, alpha: T) -> Vec<Delaunay2DShape> {
        let mut delaunay = Delaunay2D::new(self.data);
        delaunay.compute(alpha);
        delaunay.shapes
    }
}

impl<'a, T: f3l_core::BasicFloat, P> ConcaveHull<'a, T, P, 3>
where
    P: Into<[T; 3]> + Copy + std::ops::Index<usize, Output = T> + Send + Sync,
    [T; 3]: Into<P>,
{
    pub fn new_3d(data: &'a [P]) -> Self {
        Self {
            data
        }
    }

    /// Compute with default [`SacAlgorithmParameter`].
    /// 
    /// See [`ConcaveHull::compute_with_parameter`]
    pub fn compute(&mut self, alpha: T) -> Vec<Delaunay2DShape> {
        self.compute_with_parameter(alpha, SacAlgorithmParameter::default())
    }

    /// Compute concave hull with parameter of plane.
    /// 
    /// 1. Find plane of data.
    /// 2. Compute `Axis Angle` of `Normal` to `+Z`, then rotate data, get `XY`, ignore `Z`.
    /// 3. Compute Concave Hull of 2d.
    pub fn compute_with_parameter(&mut self, alpha: T, parameter: SacAlgorithmParameter) -> Vec<Delaunay2DShape> {
        use f3l_core::glam::{Vec3, Vec3A, Mat3A};
        let mut model = f3l_segmentation::sac_model::SacModelPlane::with_data(&self.data);
        let mut algorithm = f3l_segmentation::sac_algorithm::SacRansac { parameter, inliers: vec![], };
        algorithm.compute(&mut model);
        let mut coefficients = model.get_coefficient();
        if coefficients[2] < T::zero() {
            coefficients = apply_each(&coefficients, -T::one(), std::ops::Mul::mul);
        }
        let axis = [coefficients[0], coefficients[1], coefficients[2]].cross(&[T::zero(), T::zero(), T::one()]);
        let angle = [coefficients[0], coefficients[1], coefficients[2]].compute_angle(&[T::zero(), T::zero(), T::one()]);

        let mat = Mat3A::from_axis_angle(
            Vec3::new(axis[0].to_f32().unwrap(), axis[1].to_f32().unwrap(), axis[2].to_f32().unwrap()),
            angle.to_f32().unwrap()
        );
        let points = self.data.iter().map(|p| {
            let p = mat * Vec3A::new(p[0].to_f32().unwrap(), p[1].to_f32().unwrap(), p[2].to_f32().unwrap());
            [p[0], p[1]]
        }).collect::<Vec<_>>();

        let mut delaunay = Delaunay2D::new(&points);
        delaunay.compute(alpha.to_f32().unwrap());
        delaunay.shapes
    }
}

#[test]
fn test() {
    use f3l_core::glam::Vec2;
    let p2 = [Vec2::ZERO, Vec2::X, Vec2::Y];
    let p3 = [[1_f32, 0., 0.], [0., 1., 0.], [0., 0., 1.]];

    let mut concave2d = ConcaveHull::new_2d(&p2);
    let d2 = concave2d.compute(1.0);
    assert_eq!(d2[0].mesh[0].point, [2, 0, 1]);
    
    let mut concave3d = ConcaveHull::new_3d(&p3);
    let d3 = concave3d.compute(1.0);
    assert_eq!(d3[0].mesh[0].point, [2, 0, 1]);
}