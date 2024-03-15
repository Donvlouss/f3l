use std::ops::Index;

use f3l_glam::ArrayRowMajor;
use crate::glam::{Mat3, Vec3};

use crate::{one_polynomial::root3, rref, BasicFloat, MatrixLinAlg};

#[derive(Debug, Clone, Copy, Default)]
pub struct EigenVector {
    pub eigenvalues: f32,
    pub eigenvector: Vec3
}

pub fn compute_covariance_matrix<P, T: BasicFloat>(points: &[P]) -> Mat3
where
    P: Index<usize, Output = T>
{
    let mut cov = points.iter()
        .fold([0f32; 6], |mut acc, p| {
            let [x, y, z] = [p[0].to_f32().unwrap(), p[1].to_f32().unwrap(), p[2].to_f32().unwrap()];
            acc[0] += x * x;
            acc[1] += y * y;
            acc[2] += z * z;
            acc[3] += x * y;
            acc[4] += x * z;
            acc[5] += y * z;
            acc
        });
        let factor = points.len() as f32;
        for v in cov.iter_mut() {
            *v *= factor;
        }

    Mat3::from_cols_array(&[
        cov[0], cov[3], cov[4],
        cov[3], cov[1], cov[5],
        cov[4], cov[5], cov[2]
    ])
}

pub fn compute_eigenvalues_by_points<P, T: BasicFloat>(points: &[P]) -> [T; 3]
where
    P: Index<usize, Output = T>
{
    let cov = compute_covariance_matrix(points);
    compute_eigenvalues(cov)
}

pub fn compute_eigenvalues<T: BasicFloat>(cov: Mat3) -> [T; 3] {
    let [
        m00, m10, m20,
        m01, m11, m21,
        m02, m12, m22
    ] = cov.to_cols_array();

    let lambda_2 = -m00 - m11 - m22;
    let lambda_1 = m00 * m11
        + m00 * m22
        + m11 * m22
        - m12 * m21
        - m01 * m10
        - m02 * m20;
    let lambda_0 = m12 * m21 * m00
        + m01 * m10 * m22
        + m02 * m20 * m11
        - m00 * m11 * m22
        - m01 * m12 * m20
        - m02 * m10 * m21;
    let root = root3([lambda_2, lambda_1, lambda_0]);
    [
        T::from(root[0]).unwrap(),
        T::from(root[1]).unwrap(),
        T::from(root[2]).unwrap(),
    ]
}

/// Compute 3 x 3 eigenvectors<br>
/// M = (A - lambda * I)<br>
/// take any 2 rows, become:<br>
/// |   i   j   k  |<br>
/// | m00 m01 m02  |<br>
/// | m10 m11 m12  |<br>
/// get eigenvector from product 2 rows
pub fn compute_eigenvectors<T: BasicFloat, V: Into<[f32; 3]>>(cov: Mat3, eigenvalues: V) -> [Vec3; 3] {
    let mut out = [Vec3::ZERO; 3];
    let vs: [f32; 3] = eigenvalues.into();
    (0..3)
        .for_each(|i| {
            let lambda = vs[i as usize];
            let mat = cov - lambda * Mat3::IDENTITY;
            out[i as usize] = mat.row(0).cross(mat.row(1)).normalize();
        });
    out
}

pub fn compute_eigen(cov: Mat3) -> [EigenVector; 3] {
    let mut out = [EigenVector::default(); 3];
    compute_eigenvalues::<f32>(cov).into_iter()
        .enumerate()
        .for_each(|(i, v)| {
            let mat = cov - v * Mat3::IDENTITY;
            out[i] = EigenVector {
                eigenvalues: v,
                eigenvector: mat.row(0).cross(mat.row(1)).normalize()
            };
        });
    out
}

#[test]
fn test_eigenvalues() {
    use f3l_glam::glam::Vec3;
    use crate::round_slice_n;

    let cov = Mat3::from_cols(
        Vec3::new(1., 2., 3.),
        Vec3::new(2., 2., 1.,),
        Vec3::new(3., 1., 3.)
    );
    
    let eigenvalues = compute_eigenvalues::<f32>(cov);
    
    assert_eq!(
        round_slice_n([-1.52835f32, 1.38445, 6.1439], 3),
        round_slice_n(eigenvalues, 3)
    )
}

#[test]
fn test_eigenvectors() {
    use f3l_glam::glam::Vec3;

    let cov = Mat3::from_cols(
        Vec3::new(1., 2., 3.),
        Vec3::new(2., 2., 1.,),
        Vec3::new(3., 1., 3.)
    );
    
    let eigenvectors = compute_eigen(cov);
    println!("{:?}", eigenvectors[0]);
    println!("{:?}", eigenvectors[1]);
    println!("{:?}", eigenvectors[2]);

    use nalgebra::Matrix3;
    let mat = Matrix3::<f32>::new(1., 3., 3., 2., 2., 1., 3., 1., 3.);
    let solve = mat.symmetric_eigen();
    println!("Eigen values: {}", solve.eigenvalues);
    println!("Eigen vector: {}", solve.eigenvectors);
}
