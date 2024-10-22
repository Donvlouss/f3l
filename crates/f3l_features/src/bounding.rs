use f3l_core::{
    compute_covariance_matrix, get_minmax,
    glam::{Mat3A, Vec2, Vec3A},
    jacobi_eigen_square_n,
    serde::{self, Deserialize, Serialize},
    BasicFloat, EigenSet, F3lCast,
};

/// Compute AABB. A wrapper of [`get_minmax`].
#[inline]
pub fn aabb<P, T: BasicFloat, const D: usize>(cloud: &[P]) -> (P, P)
where
    P: Into<[T; D]> + Clone + Copy,
    [T; D]: Into<P>,
{
    get_minmax(cloud)
}

/// Compute Oriented Bounding Box
///
/// 1. Compute covariance of data.
/// 2. Compute eigenvalues and eigenvectors of covariance.
/// 3. Make each direction orthogonal by cross product.
/// 4. Multiply inverse of eigenvector to align data to orthogonal.
/// 5. Compute AABB of aligned data as `Length`
///
/// # Examples
///
/// ```
/// let vertices = load_ply("../../data/table_voxel_down.ply");
/// let obb = OBB::compute(&vertices);
/// ```
///
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct OBB<T: BasicFloat, const D: usize, P>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync + std::ops::Index<usize, Output = T>,
    [T; D]: Into<P>,
{
    /// Center of OBB
    pub center: P,
    /// Largest eigenvector of eigenvalue
    pub primary: P,
    /// second one
    pub secondary: P,
    /// Smallest one
    pub tertiary: P,
    /// Length of 3 directions.
    pub length: P,
}

impl<T: BasicFloat, const D: usize, P> OBB<T, D, P>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync + std::ops::Index<usize, Output = T>,
    [T; D]: Into<P>,
{
    pub fn compute(cloud: &[P]) -> Self {
        assert!(D == 2 || D == 3);

        let (cov, _) = compute_covariance_matrix(cloud);

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

        let third = if D == 3 {
            major.cross(second)
        } else {
            Vec3A::Z
        }
        .normalize();
        let second = third.cross(major);
        let major = second.cross(third);

        let mat = Mat3A::from_cols(major, second, third);//.inverse();

        let mut mean = Vec3A::ZERO;
        let mut min = Vec3A::MAX;
        let mut max = Vec3A::MIN;

        cloud.iter().for_each(|p| {
            for i in 0..D {
                mean[i] += p[i].to_f32().unwrap();
            }
        });
        mean /= cloud.len() as f32;

        cloud.iter().for_each(|p| {
            let mut slice = [0f32, 0., 0.];
            for i in 0..D {
                slice[i] = p[i].to_f32().unwrap();
            }
            let p: Vec3A = slice.into();
            let x = (p - mean).dot(major);
            let y = (p - mean).dot(second);
            let z = (p - mean).dot(third);

            min.x = min.x.min(x);
            min.y = min.y.min(y);
            min.z = min.z.min(z);
            max.x = max.x.max(x);
            max.y = max.y.max(y);
            max.z = max.z.max(z);
        });
        let shift = (max + min) * 0.5f32;

        let aligned_center = mean + mat.mul_vec3a(shift);
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
            length[i] = T::from((max[i] - min[i]) * 0.5f32).unwrap().abs();
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
    assert_eq!([1.4142, 0.7071], round_slice_n(result.length, 4));
}
