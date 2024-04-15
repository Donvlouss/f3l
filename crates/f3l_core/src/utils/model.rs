use crate::{apply_both, BasicFloat, SimpleSliceMath};
use std::ops::Index;

pub fn find_circle<T: BasicFloat, const D: usize, P>(points: &[P; 3]) -> ([T; 3], [T; 3], T)
where
    P: Into<[T; D]> + Index<usize, Output = T>,
{
    assert!(D >= 2 && D <= 4);
    let mut mat = [[T::zero(); 3]; 3];
    let n = match D {
        2 => 2_usize,
        3 => 3,
        4 => 3,
        // This could not happened.
        _ => panic!("Out of Range."),
    };
    (0..n).for_each(|i| {
        (0..n).for_each(|ii| {
            mat[i][ii] = points[i][ii];
        });
    });

    let v12 = apply_both(&mat[0], &mat[1], std::ops::Sub::sub);
    let v21 = apply_both(&mat[1], &mat[0], std::ops::Sub::sub);
    let v13 = apply_both(&mat[0], &mat[2], std::ops::Sub::sub);
    let v31 = apply_both(&mat[2], &mat[0], std::ops::Sub::sub);
    let v23 = apply_both(&mat[1], &mat[2], std::ops::Sub::sub);
    let v32 = apply_both(&mat[2], &mat[1], std::ops::Sub::sub);

    let normal = v12.cross(&v23);
    let common_divided = T::one() / (T::from(2.).unwrap() * normal.len().powi(2));

    let alpha = (v23.len().powi(2) * v12.dot(&v13)) * common_divided;
    let beta = (v13.len().powi(2) * v21.dot(&v23)) * common_divided;
    let gamma = (v12.len().powi(2) * v31.dot(&v32)) * common_divided;

    let mut pc = [T::zero(); 3];
    (0..3).for_each(|i| pc[i] = alpha * mat[0][i] + beta * mat[1][i] + gamma * mat[2][i]);

    let radius = apply_both(&pc, &mat[0], std::ops::Sub::sub).len();
    (pc, normal, radius)
}
