use std::{ops::Index, process::Output};

use super::Eigenvectors3;
use f3l_core::{
    BasicFloat,
    glam::Vec3,
    matrix3x3::*,
    rayon::prelude::*,
};
use f3l_search_tree::*;

pub struct NormalEstimation<'a, P, T: BasicFloat, const D: usize>
where
    P:Into<[T; D]> + Clone + Copy,
    [T; D]: Into<P>
{
    pub radius: T,
    data: Option<&'a Vec<P>>,
    tree: KdTree<T, D>,
    normals: Vec<Vec3>
}

fn ortho(v: Vec3) {
    let mut out = Vec3::ZERO;

}

impl<'a, P, T:BasicFloat, const D: usize> NormalEstimation<'a, P, T, D>
where
    P:Into<[T; D]> + Clone + Copy + Send + Sync + Index<usize, Output=T>,
    [T; D]: Into<P>
{
    pub fn new(radius: T) -> Self {
        Self {
            radius,
            data: None,
            tree: KdTree::<T, D>::new(),
            normals: vec![]
        }
    }

    pub fn with_data(radius: T, data: &'a Vec<P>) -> Self {
        let mut entity = Self::new(radius);
        entity.set_data(data);
        entity
    }

    pub fn set_data(&mut self, data: &'a Vec<P>) {
        self.data = Some(data);
        self.tree.set_data(data);
    }

    pub fn normals(&self) -> Vec<Vec3> {
        self.normals.clone()
    }

    pub fn compute(&mut self) -> bool {
        if self.tree.data.is_empty() {
            return false;
        }
        self.tree.build();
        let data = self.data.unwrap();
        let radius = self.radius.to_f32().unwrap();
        
        let normals = (0..data.len())
            // .into_par_iter()
            .into_iter()
            .map(|i| {
                let cloud = self.tree.search_radius(&data[i], radius);
                let cov = compute_covariance_matrix(&cloud);

                let max_v = cov.to_cols_array().iter().max_by(|&&a, &b| a.partial_cmp(b).unwrap()).unwrap().to_owned();
                let mat = 1. / max_v * cov;

                let eigenvalues = compute_eigenvalues::<f32>(mat);

                let eigenvector =
                    if eigenvalues[1] - eigenvalues[0] > f32::EPSILON {
                        compute_eigenvector(mat, eigenvalues[0])
                    } else if eigenvalues[2] - eigenvalues[0] > f32::EPSILON {
                        compute_eigenvector(mat, eigenvalues[0]).any_orthonormal_vector()
                    } else {
                        Vec3::Z
                    };


                (i, eigenvector)
            }).collect::<Vec<_>>();
        self.normals = vec![Vec3::ZERO; data.len()];
        normals.into_iter()
            .for_each(|(i, n)| self.normals[i]=n);
        true
    }

}