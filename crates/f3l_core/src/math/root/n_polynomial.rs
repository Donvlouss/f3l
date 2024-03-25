use f3l_glam::F3lMatrix;

/// Ref: [Gaussian elimination (wiki)](https://en.wikipedia.org/wiki/Gaussian_elimination)
pub fn gaussian_elimination<M: F3lMatrix + Copy, O: F3lMatrix + Copy>(a: &M, b: &O) -> O {
    assert!(a.rows() == b.rows());
    assert!(a.rows() <= a.cols());

    let mut a = *a;
    let mut b = *b;

    (0..a.rows()).for_each(|lead| {
        (0..a.rows()).for_each(|r| {
            let d = a.at(lead, lead);
            let m = a.at(r, lead) / d;

            (0..a.cols()).for_each(|c| {
                if r == lead {
                    a.set_element((r, c), a.at(r, c) / d);
                } else {
                    a.set_element((r, c), a.at(r, c) - a.at(lead, c) * m);
                }
            });
            if r == lead {
                b.set_element((r, 0), b.at(r, 0) / d);
            } else {
                b.set_element((r, 0), b.at(r, 0) - b.at(lead, 0) * m);
            }
        });
    });
    b
}

#[cfg(test)]
mod gaussian_elimination {
    use super::gaussian_elimination;
    use crate::{is_slice_ok, round_slice_n};
    use f3l_glam::glam::{Mat3, Vec3};

    #[test]
    fn has_solution() {
        let a = Mat3::from_cols(
            Vec3::new(5., 3., 2.),
            Vec3::new(-6., -2., 4.),
            Vec3::new(-7., 5., -3.),
        );
        let b = Vec3::new(7., -17., 29.);

        let x: [f32; 3] = gaussian_elimination(&a, &b).into();

        assert_eq!(round_slice_n([2f32, 4f32, -3f32], 4), round_slice_n(x, 4));
    }

    #[test]
    fn parallel() {
        let a = Mat3::from_cols(
            Vec3::new(5., 3., 3.),
            Vec3::new(-6., -2., -2.),
            Vec3::new(-7., 5., 5.),
        );
        let b = Vec3::new(7., -17., 29.);

        let x: [f32; 3] = gaussian_elimination(&a, &b).into();

        assert!(!is_slice_ok(x));
    }

    #[test]
    fn nan_solution() {
        let a = Mat3::from_cols(
            Vec3::new(2., 3., -1.),
            Vec3::new(4., -6., 2.),
            Vec3::new(6., 9., -3.),
        );
        let b = Vec3::new(5., 10., 15.);

        let x: [f32; 3] = gaussian_elimination(&a, &b).into();

        assert!(!is_slice_ok(x));
    }
}
