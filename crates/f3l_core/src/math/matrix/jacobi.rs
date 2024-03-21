use std::ops::{Index, IndexMut};
use crate::{BasicFloat, Eigen};


const  MAX_ROTATION: usize = 20;

fn jacobi_rotate<T, M, const D: usize>(m: &mut M, i:usize, j: usize, k: usize, l: usize, s: T, tau: T) -> (T, T)
where
    T: BasicFloat,
    M: Into<[[T; D]; D]> + Index<usize, Output = [T; D]> + IndexMut<usize, Output = [T; D]>
{
    // let mut m = m;
    let g = m[i][j];
    let h = m[k][l];
    m[i][j] = g - s * (h + g * tau);
    m[k][l] = h + s * (g - h * tau);
    (
        g, h
    )
}

/// Ref: vtk/Math/Core/vtkMath/vtkJacobiN
pub fn jacobi_eigen_square_n<T, M, const D: usize>(mat: M) -> [Eigen<T, D>; D]
where
    T: BasicFloat,
    M: Into<[[T; D]; D]> + Index<usize, Output = [T; D]> + IndexMut<usize, Output = [T; D]> + Clone
{

    let mut a = mat;
    let mut z_space = [T::zero(); D];
    let mut b_space = [T::zero(); D];
    let mut eigenvalues = [T::zero(); D];
    
    let mut eigenvectors = [[T::zero(); D]; D];
    (0..D).for_each(|i| eigenvectors[i][i] = T::one());
    (0..D).for_each(|i| {
        b_space[i] = a[i][i];
        eigenvalues[i] = a[i][i];
    });

    let mut run = 0_usize;
    for i in 0..MAX_ROTATION {
        run += 1;
        let mut sm = T::zero();
        (0..D-1).for_each(|ip| {
            (ip+1..D).for_each(|iq| {
                sm += a[ip][iq].abs();
            });
        });

        if sm == T::zero() {
            break;
        }

        let trash = 
            if i < 3 {T::from(0.2f32).unwrap() * sm / T::from(D * D).unwrap()}
            else {T::zero()};
        
        (0..D-1).for_each(|ip| {
            (ip+1..D).for_each(|iq| {
                let mut g = T::from(100f32).unwrap() * a[ip][iq].abs();

                if i > 3
                    && eigenvalues[ip].abs()+g == eigenvalues[ip].abs()
                    && eigenvalues[iq].abs()+g == eigenvalues[iq].abs()
                {
                    a[ip][iq] = T::zero();
                } else if a[ip][iq].abs() > trash {

                    let h = eigenvalues[iq] - eigenvalues[ip];
                    let mut t;
                    if h.abs() + g == h.abs() {
                        t = a[ip][iq] / h;
                    } else {
                        let theta = T::from(0.5f32).unwrap() * h / a[ip][iq];
                        t = T::one() / (theta.abs() + (T::one() + theta * theta).sqrt());
                        if theta < T::zero() {
                            t = -t;
                        }
                    }

                    let c = T::one() / (T::one() + t * t).sqrt();
                    let s = t * c;
                    let tau = s / (T::one() + c);
                    let mut h = t * a[ip][iq];
                    z_space[ip] -= h;
                    z_space[iq] += h;
                    eigenvalues[ip] -= h;
                    eigenvalues[iq] += h;
                    a[ip][iq] = T::zero();

                    if ip != 0 {
                        (0..=ip-1).for_each(|j| {
                            (g, h) = jacobi_rotate(&mut a, j, ip, j, iq, s, tau);
                        });
                    }
                    if iq != 0 {
                        (ip+1..=iq-1).for_each(|j| {
                            (g, h) = jacobi_rotate(&mut a, ip, j, j, iq, s, tau);
                        });
                    }
                    (iq+1..D).for_each(|j| {
                        (g, h) = jacobi_rotate(&mut a, ip, j, iq, j, s, tau);
                    });
                    (0..D).for_each(|j| {
                        (g, h) = jacobi_rotate(&mut eigenvectors, j, ip, j, iq, s, tau);
                    });
                }
            });
        });

        (0..D).for_each(|ip| {
            b_space[ip] += z_space[ip];
            eigenvalues[ip] = b_space[ip];
            z_space[ip] = T::zero();
        });
    };

    assert!(run <= MAX_ROTATION);

    (0..D-1).for_each(|j| {
        let mut k = j;
        let mut tmp = eigenvalues[k];
        (j+1..D).for_each(|i| {
            if eigenvalues[i] >= tmp {
                k = i;
                tmp = eigenvalues[k];
            }
        });
        if k != j {
            eigenvalues[k] = eigenvalues[j];
            eigenvalues[j] = tmp;
            (0..D).for_each(|i| {
                tmp = eigenvectors[i][j];
                eigenvectors[i][j] = eigenvectors[i][k];
                eigenvectors[i][k] = tmp;
            });
        }
    });

    let call_half_n = (D >> 1) + (D & 1);
    (0..D).for_each(|j| {
        let mut num_pos = 0_usize;
        (0..D).for_each(|i| {
            if eigenvectors[i][j] >= T::zero() {
                num_pos += 1;
            }
        });
        if num_pos < call_half_n {
            (0..D).for_each(|i| {
                eigenvectors[i][j] *= -T::one();
            });
        }
    });

    let mut out = [Eigen::<T, D>::default(); D];
    (0..D).for_each(|i| {
        let mut v = [T::zero(); D];
        (0..D).for_each(|ii| {
            v[ii] = eigenvectors[ii][i];
        });
        out[i] = Eigen {
            eigenvalue: eigenvalues[i],
            eigenvector: v
        };
    });
    out
}
