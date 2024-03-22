use f3l_core::{
    compute_covariance_matrix, get_minmax,
    glam::{Mat3A, Vec2, Vec3A},
    jacobi_eigen_square_n, BasicFloat, EigenSet, F3lCast,
};

#[inline]
pub fn aabb<P, T: BasicFloat, const D: usize>(cloud: &[P]) -> (P, P)
where
    P: Into<[T; D]> + Clone + Copy,
    [T; D]: Into<P>,
{
    get_minmax(cloud)
}

#[derive(Debug, Clone, Copy)]
/// Primary: Largest eigenvector of eigenvalue
/// Secondary: second one
/// Tertiary: third one
pub struct OBB<T: BasicFloat, const D: usize, P>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync + std::ops::Index<usize, Output = T>,
    [T; D]: Into<P>,
{
    pub center: P,
    pub primary: P,
    pub secondary: P,
    pub tertiary: P,
    pub length: P,
}

impl<T: BasicFloat, const D: usize, P> OBB<T, D, P>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync + std::ops::Index<usize, Output = T>,
    [T; D]: Into<P>,
{
    pub fn compute(cloud: &[P]) -> Self {
        assert!(D <= 3);

        let (cov, _) = compute_covariance_matrix(&cloud);

        let eigen = EigenSet(jacobi_eigen_square_n(cov));

        let major: Vec3A = if D == 3 {
            eigen[0].eigenvector.cast_f32::<3>().into()
        } else {
            Vec3A::from(Vec2::from(eigen[0].eigenvector.cast_f32::<2>()).extend(0f32))
        }
        .normalize();
        let second: Vec3A = if D == 3 {
            eigen[1].eigenvector.cast_f32::<3>().into()
        } else {
            Vec3A::from(Vec2::from(eigen[1].eigenvector.cast_f32::<2>()).extend(0f32))
        }
        .normalize();

        let mut third = if D == 3 {
            major.cross(second)
        } else {
            Vec3A::Z
        }
        .normalize();
        let mut second = third.cross(major);
        let mut major = second.cross(third);

        (0..D - 1).for_each(|i| {
            major[i] *= -1f32;
            second[i] *= -1f32;
            third[i] *= -1f32;
        });

        let mat = Mat3A::from_cols(major, second, third).inverse();

        let align = cloud
            .iter()
            .map(|&p| {
                if D == 3 {
                    mat.mul_vec3a(Vec3A::new(
                        p[0].to_f32().unwrap(),
                        p[1].to_f32().unwrap(),
                        p[2].to_f32().unwrap(),
                    ))
                } else {
                    mat.mul_vec3a(Vec3A::new(
                        p[0].to_f32().unwrap(),
                        p[1].to_f32().unwrap(),
                        0f32,
                    ))
                }
            })
            .collect::<Vec<Vec3A>>();
        let (min, max) = aabb::<Vec3A, f32, 3>(&align);
        let aligned_center = mat.mul_vec3a((max + min) * 0.5f32);

        let mut center = [T::zero(); D];
        let mut primary = [T::zero(); D];
        let mut secondary = [T::zero(); D];
        let mut tertiary = [T::zero(); D];
        let mut length = [T::zero(); D];
        (0..D).for_each(|i| {
            center[i] = T::from(aligned_center[i]).unwrap();
            primary[i] = T::from(major[i]).unwrap();
            secondary[i] = T::from(second[i]).unwrap();
            tertiary[i] = T::from(third[i]).unwrap();
            length[i] = T::from((max[i] - min[i]) / 2f32).unwrap().abs();
        });

        Self {
            center: center.into(),
            primary: primary.into(),
            secondary: secondary.into(),
            tertiary: tertiary.into(),
            length: length.into(),
        }
    }
}

#[test]
fn obb_2d() {
    use f3l_core::round_slice_n;
    let cloud = vec![[1f32, 0.], [0., 1.], [3., 2.], [2., 3.]];
    let result = OBB::compute(&cloud);

    assert_eq!([1.5f32, 1.5], round_slice_n(result.center, 4));
    assert_eq!([0.7071, 1.4142], round_slice_n(result.length, 4));
}
