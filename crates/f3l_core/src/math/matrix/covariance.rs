use crate::{apply_each, BasicFloat};
use std::ops::Index;

pub fn compute_covariance_matrix<P, T: BasicFloat, const D: usize>(
    points: &[P],
) -> ([[T; D]; D], [T; D])
where
    P: Index<usize, Output = T> + Copy + Into<[T; D]>,
{
    let means = points.iter().fold([T::zero(); D], |acc, p| {
        let mut acc = acc;
        (0..D).for_each(|i| acc[i] += p[i]);
        acc
    });
    let factor = T::one() / T::from(points.len()).unwrap();
    let means = apply_each(&means, factor, std::ops::Mul::mul);
    let mut cov = [[T::zero(); D]; D];
    points.iter().for_each(|p| {
        let xp = (0..D).map(|i| p[i] - means[i]).collect::<Vec<T>>();
        (0..D).for_each(|c| {
            (0..D).for_each(|r| {
                cov[r][c] += xp[r] * xp[c];
            });
        });
    });
    (0..D).for_each(|r| {
        (0..D).for_each(|c| {
            cov[r][c] *= factor;
        });
    });
    (cov, means)
}

#[test]
fn covariance_3d() {
    use f3l_glam::glam::Vec3;
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
    let target = [
        [0.000547356_f32, -0.0000049326613, 0.000002223934],
        [-0.0000049326613, 0.0000016343753, -0.00000029845418],
        [0.000002223934, -0.00000029845418, 0.00000006901876],
    ];
    // let cov = compute_covariance_matrix(&points);
    let cov = compute_covariance_matrix(&points);
    assert_eq!(target, cov.0);
}

#[test]
fn covariance_2d() {
    let points = [
        [3.7f32, 1.7],
        [4.1, 3.8],
        [4.7, 2.9],
        [5.2, 2.8],
        [6.0, 4.0],
        [6.3, 3.6],
        [9.7, 6.3],
        [10.0, 4.9],
        [11.0, 3.6],
        [12.5, 6.4],
    ];
    let cov = compute_covariance_matrix(&points);
    let target = [[9.083601, 3.3650002], [3.3650002, 2.0160003_f32]];
    assert_eq!(target, cov.0);
}
