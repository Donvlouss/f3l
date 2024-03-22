use crate::BasicFloat;

pub fn rref<T: BasicFloat, const R: usize, const C: usize, M: Into<[[T; C]; R]>>(matrix: M) -> M
where
    [[T; C]; R]: Into<M>,
{
    let mut matrix: [[T; C]; R] = matrix.into();
    let mut lead = 0_usize;

    (0..R).for_each(|r| {
        if lead >= C {
            return;
        }
        let mut i = r;
        while matrix[i][lead] == T::zero() {
            i += 1;
            if i == R {
                i = r;
                lead += 1;
                if lead == C {
                    return;
                }
            }
        }
        (matrix[i], matrix[r]) = (matrix[r], matrix[i]);
        let lv = matrix[r][lead];
        (0..C).for_each(|i| {
            matrix[r][i] /= lv;
        });

        (0..R).filter(|&i| i != r).for_each(|i| {
            let lv = matrix[i][lead];
            (0..C).for_each(|ii| {
                matrix[i][ii] -= lv * matrix[r][ii];
            });
        });
        lead += 1;
    });
    matrix.into()
}

#[cfg(test)]
mod test_rref {
    use super::*;

    #[test]
    fn mat33() {
        let matrix = [
            [1f32, 2., 0., 1., 0., 0.],
            [0., 0., 0., 3., 0., 0.],
            [0., 0., 1., 0., 1., 0.],
        ];
        let src = [
            [1f32, 2.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        ];

        let target = rref(matrix);
        assert_eq!(src, target);
    }
}
