use f3l_glam::{F3lMatrix, glam};

pub mod one_polynomial;
pub mod n_polynomial;


pub trait MatrixLinAlg: F3lMatrix
where
    Self: Copy,
{
    /// solve: ax = b
    fn solve(&self, b: Self::RowType) -> Self::RowType
    where <Self as F3lMatrix>::RowType: F3lMatrix {
        n_polynomial::gaussian_elimination(self, &b)
    }
}

impl MatrixLinAlg for glam::Mat2 {}
impl MatrixLinAlg for glam::Mat3 {}
impl MatrixLinAlg for glam::Mat3A {}
impl MatrixLinAlg for glam::Mat4 {}

#[cfg(test)]
mod matrix_lin_alg_impl {
    use super::*;
    use glam::{Mat3, Vec3};
    use crate::round_slice_n;

    #[test]
    fn has_solution() {
        let a = Mat3::from_cols(
            Vec3::new(5., 3., 2.),
            Vec3::new(-6., -2., 4.),
            Vec3::new(-7., 5., -3.),
        );
        let b = Vec3::new(7., -17., 29.);

        let x: [f32; 3] = a.solve(b).into();

        assert_eq!(
            round_slice_n([2f32, 4f32, -3f32], 4),
            round_slice_n(x, 4)
        );
    }
}