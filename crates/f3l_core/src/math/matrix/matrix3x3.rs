use std::ops::Index;

use f3l_glam::F3lMatrix;
use nalgebra::ComplexField;
use crate::{apply_each, Eigen, EigenSet};
use crate::glam::{Mat3, Vec3};

use crate::{one_polynomial::{root2, root3}, BasicFloat};


pub fn compute_covariance_matrix<P, T: BasicFloat>(points: &[P]) -> Mat3
where
    P: Index<usize, Output = T> + Copy
{
    let means = points.iter()
        .fold([T::zero(); 3], |acc, p| {
            [
                acc[0] + p[0],
                acc[1] + p[1],
                acc[2] + p[2],
            ]
        });
    let means = apply_each(&means, T::one() / T::from(points.len()).unwrap(), std::ops::Mul::mul);
    let mut cov = [[T::zero(); 3]; 3];
    points.iter().for_each(|p| {
        let xp = (0..3).map(|i| p[i] - means[i]).collect::<Vec<T>>();
        (0..3).for_each(|i| {
            cov[0][i] += xp[0] * xp[i];
            cov[1][i] += xp[1] * xp[i];
            cov[2][i] += xp[2] * xp[i];
        });
    });
    let mut out = Mat3::ZERO;
    (0..3).for_each(|r|{
        (0..3).for_each(|c| {
            out.set_element((r, c), cov[r][c].to_f32().unwrap() / (points.len() as f32));
        });
    });
    out
}

pub fn compute_eigenvalues_by_points<P, T: BasicFloat>(points: &[P]) -> [T; 3]
where
    P: Index<usize, Output = T> + Copy
{
    let cov = compute_covariance_matrix(points);
    compute_eigenvalues(cov)
}

pub fn compute_eigenvalues<T: BasicFloat>(cov: Mat3) -> [T; 3] {
    assert!(cov.to_cols_array().iter().all(|&v| v.is_finite()));
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

    if !(lambda_0.is_finite() || lambda_1.is_finite() || lambda_2.is_finite()) {
        return [T::zero(); 3];
    }

    let root = root3([lambda_2, lambda_1, lambda_0]);
    let mut eigenvalues = [
        T::from(root[0]).unwrap(),
        T::from(root[1]).unwrap(),
        T::from(root[2]).unwrap(),
    ];
    eigenvalues.sort_by(|a, &b| b.partial_cmp(a).unwrap());
    if eigenvalues[0] <= T::zero() {
        let root = root2([lambda_2, lambda_1]);
        eigenvalues[1] = T::from(root[0]).unwrap();
        eigenvalues[2] = T::from(root[1]).unwrap();
    }
    eigenvalues
}

/// Compute 3 x 3 eigenvectors<br>
/// M = (A - lambda * I)<br>
/// take any 2 rows, become:<br>
/// |   i   j   k  |<br>
/// | m00 m01 m02  |<br>
/// | m10 m11 m12  |<br>
/// get eigenvector from product 2 rows
pub fn compute_eigenvector(cov: Mat3, eigenvalue: f32) -> Vec3 {
    let mat = cov - eigenvalue * Mat3::IDENTITY;
    let cross_product = [
        mat.row(0).cross(mat.row(1)),
        mat.row(0).cross(mat.row(2)),
        mat.row(1).cross(mat.row(2)),
    ];
    let id = [
            cross_product[0].length().abs(),
            cross_product[1].length().abs(),
            cross_product[2].length().abs(),
        ]
        .into_iter()
        .enumerate()
        .max_by(|&(_,a), (_,b)| {
            let v = a.partial_cmp(b);
            match v {
                Some(v) => v,
                None => panic!(""),
            }
        })
        .map(|(i,_)| i)
        .unwrap();
    1. / cross_product[id].length() * cross_product[id]
}

pub fn compute_eigenvectors<T: BasicFloat, V: Into<[f32; 3]>>(cov: Mat3, eigenvalues: V) -> [Vec3; 3] {
    let mut out = [Vec3::ZERO; 3];
    let vs: [f32; 3] = eigenvalues.into();
    (0..3)
        .for_each(|i| {
            let lambda = vs[i as usize];
            out[i] = compute_eigenvector(cov, lambda);
        });
    out
}

pub fn compute_eigen(cov: Mat3) -> EigenSet<f32, 3, 3> {
    let mut out = [Eigen::default(); 3];
    let max_v = cov.to_cols_array().iter().max_by(|&&a, &b| a.partial_cmp(b).unwrap()).unwrap().to_owned();
    let mat = cov.mul_scalar(1. /max_v);
    compute_eigenvalues::<f32>(mat).into_iter()
        .enumerate()
        .for_each(|(i, v)| {
            // let b = mat - Mat3::IDENTITY.mul_scalar(v * max_v);
            out[i] = Eigen {
                eigenvalue: v * max_v,
                // eigenvector: b.row(0).cross(b.row(1)).normalize()
                eigenvector: compute_eigenvector(mat, v).into()
            };
        });
    EigenSet(out)
}

fn unit_orthogonal(v: Vec3) -> Vec3 {
    let mut out = Vec3::ZERO;
    if !(v.x <= v.z * f32::EPSILON)
        || !(v.y <= v.z * f32::EPSILON)
    {
        let factor = 1f32 / v.truncate().length();
        out.x = -v.y.conjugate() * factor;
        out.y = v.x.conjugate() * factor;
    } else {
        let factor = 1f32 / f3l_glam::glam::Vec2::new(v.y, v.z).length();
        out.y = -v.z.conjugate() * factor;
        out.z = v.y.conjugate() * factor;
    }
    out
}

pub fn compute_eigen_rigorous(cov: Mat3) -> EigenSet<f32, 3, 3> {
    let max_v = cov.to_cols_array().iter().max_by(|&&a, &b| a.partial_cmp(b).unwrap()).unwrap().to_owned();
    let mat = cov.mul_scalar(1. /max_v);

    let mut eigenvalues = compute_eigenvalues::<f32>(mat);
    // set to increase
    eigenvalues.sort_by(|&a, b| a.abs().partial_cmp(&b.abs()).unwrap());

    let mut eigenvectors = Mat3::IDENTITY;
    if (eigenvalues[2] - eigenvalues[0]) < f32::EPSILON {
        // all equal
    } else if (eigenvalues[1] - eigenvalues[0]) < f32::EPSILON {
        // first and second equal
        eigenvectors.z_axis = compute_eigenvector(mat, eigenvalues[2]);
        // eigenvectors.y_axis = eigenvectors.z_axis.any_orthonormal_vector();
        eigenvectors.y_axis = unit_orthogonal(eigenvectors.z_axis);
        eigenvectors.x_axis = eigenvectors.y_axis.cross(eigenvectors.z_axis);
    } else if (eigenvalues[2] - eigenvalues[1]) < f32::EPSILON {
        // second and third equal
        eigenvectors.z_axis = compute_eigenvector(mat, eigenvalues[0]);
        // eigenvectors.y_axis = eigenvectors.z_axis.any_orthonormal_vector();
        eigenvectors.y_axis = unit_orthogonal(eigenvectors.z_axis);
        eigenvectors.x_axis = eigenvectors.y_axis.cross(eigenvectors.z_axis);
    } else {
        let mut vector_lens = [0f32; 3];
        eigenvalues.iter()
            .enumerate()
            .for_each(|(i, &v)| {
                let vec = compute_eigenvector(mat, v);
                *eigenvectors.col_mut(i) = vec;
                vector_lens[i] = vec.length();
            });
        let min_id = eigenvalues.iter().enumerate()
            .min_by(|&(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i).unwrap();
        let max_id = eigenvalues.iter().enumerate()
            .max_by(|&(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i).unwrap();
        let mid_id = 3 - max_id - min_id;

        *eigenvectors.col_mut(min_id) = eigenvectors.col((min_id+1)%3).cross(eigenvectors.col((min_id+2)%3)).normalize();
        *eigenvectors.col_mut(mid_id) = eigenvectors.col((mid_id+1)%3).cross(eigenvectors.col((mid_id+2)%3)).normalize();
    }
    let mut out = [Eigen::default(); 3];
    (0..3).for_each(|i| {
        out[i] = Eigen {
            eigenvalue: eigenvalues[i] * max_v,
            eigenvector: eigenvectors.col(i).into()
        };
    });
    EigenSet(out)
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
        round_slice_n([6.1439, 1.38445, -1.52835f32], 3),
        round_slice_n(eigenvalues, 3)
    );
}

#[test]
fn covariance() {
    use f3l_glam::ArrayRowMajor;
    let points = vec![
        Vec3::new(-0.12531, -0.28153, -1.9203),
        Vec3::new(-0.11903, -0.28154, -1.9204),
        Vec3::new(-0.10804, -0.282605, -1.9203501),
        Vec3::new(-0.081356, -0.28155, -1.9205),
        Vec3::new(-0.068798, -0.28153, -1.9203),
        Vec3::new(-0.056241, -0.2815, -1.9201),
        Vec3::new(-0.12217, -0.27958, -1.9207),
        Vec3::new(-0.11275, -0.2796, -1.9208),
        Vec3::new(-0.10542667, -0.27833664, -1.921),
        Vec3::new(-0.076647, -0.27959, -1.9207),
    ];
    let target = Mat3::from_rows_slice(&[
        0.000547356, -0.0000049326613, 0.000002223934,
        -0.0000049326613, 0.0000016343753, -0.00000029845418,
        0.000002223934, -0.00000029845418, 0.00000006901876,
    ]);
    let cov = compute_covariance_matrix(&points);
    assert_eq!(target, cov);
}
