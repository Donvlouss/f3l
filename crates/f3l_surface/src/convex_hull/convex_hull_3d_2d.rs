use f3l_core::{
    compute_covariance_matrix,
    glam::{Mat3A, Vec3A},
    jacobi_eigen_square_n, BasicFloat, EigenSet, F3lCast, SimpleSliceMath,
};
use std::ops::Index;

use crate::{Convex, ConvexHull2D, ConvexHullId};

/// This structure is using to process 3D data which is near a plane.
#[derive(Debug, Clone)]
pub struct ConvexHull3D2D<'a, T: BasicFloat, P>
where
    P: Into<[T; 3]> + Clone + Copy + Send + Sync + Index<usize, Output = T>,
{
    pub data: &'a [P],
    pub hulls: ConvexHullId,
}

impl<'a, T: BasicFloat, P> Convex<'a, P> for ConvexHull3D2D<'a, T, P>
where
    P: Into<[T; 3]> + Clone + Copy + Send + Sync + Index<usize, Output = T>,
{
    fn new(data: &'a [P]) -> Self {
        Self {
            data,
            hulls: ConvexHullId::D2(vec![]),
        }
    }

    /// 1. Compute `eigenvector` by using [`compute_covariance_matrix`] and [`jacobi_eigen_square_n`].
    /// 2. Using Eigenvector to align data to XY Plane.
    /// 3. Using [`ConvexHull2D`] to compute aligned points.
    fn compute(&mut self) {
        let (cov, _) = compute_covariance_matrix(self.data);
        let eigen = EigenSet(jacobi_eigen_square_n(cov));

        let major: Vec3A = eigen[0].eigenvector.cast_f32::<3>().normalized().into();
        let second: Vec3A = eigen[1].eigenvector.cast_f32::<3>().normalized().into();

        let mut third = major.cross(second).normalize();
        let mut second = third.cross(major);
        let mut major = second.cross(third);

        (0..2).for_each(|i| {
            major[i] *= -1f32;
            second[i] *= -1f32;
            third[i] *= -1f32;
        });

        let mat = Mat3A::from_cols(major, second, third).inverse();

        let align = self
            .data
            .iter()
            .map(|&p| {
                let align_p = mat.mul_vec3a(Vec3A::new(
                    p[0].to_f32().unwrap(),
                    p[1].to_f32().unwrap(),
                    p[2].to_f32().unwrap(),
                ));
                [align_p[0], align_p[1]]
            })
            .collect::<Vec<_>>();

        let mut cvh = ConvexHull2D::new(&align);
        cvh.compute();

        self.hulls = ConvexHullId::D2(cvh.hulls);
    }
}
