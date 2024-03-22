use std::ops::Index;

use f3l_core::{
    compute_covariance_matrix, glam::Vec3, matrix3x3::*, rayon::prelude::*, BasicFloat, GenericArray
};
use f3l_search_tree::*;

pub struct NormalEstimation<'a, P, T: BasicFloat>
where
    P:Into<[T; 3]> + Clone + Copy,
    [T; 3]: Into<P>
{
    pub method: SearchBy,
    fast: bool,
    data: Option<&'a Vec<P>>,
    tree: KdTree<T, 3>,
    normals: Vec<Option<Vec3>>,
}

impl<'a, P, T:BasicFloat> NormalEstimation<'a, P, T>
where
    P:Into<[T; 3]> + Clone + Copy + Send + Sync + Index<usize, Output=T>,
    [T; 3]: Into<P>
{
    pub fn new(method: SearchBy) -> Self {
        Self {
            method,
            fast: true,
            data: None,
            tree: KdTree::<T, 3>::new(),
            normals: vec![],
        }
    }

    pub fn fast(&self) -> bool {
        self.fast
    }

    pub fn set_fast(&mut self, use_fast: bool) {
        self.fast = use_fast;
    }

    pub fn with_data(method: SearchBy, data: &'a Vec<P>) -> Self {
        let mut entity = Self::new(method);
        entity.set_data(data);
        entity
    }

    pub fn set_data(&mut self, data: &'a Vec<P>) {
        self.data = Some(data);
        self.tree.set_data(data);
    }

    pub fn normals(&self) -> Vec<Option<Vec3>> {
        self.normals.clone()
    }

    pub fn compute(&mut self) -> bool {
        if self.tree.data.is_empty() {
            return false;
        }
        self.tree.build();
        let data = self.data.unwrap();
        
        let normals = (0..data.len())
            .into_par_iter()
            .map(|i| {
                let cloud = match self.method {
                    SearchBy::Count(k) => 
                        self.tree.search_knn(&data[i], k)
                            .iter().map(|&(p, _)| p).collect(),
                    SearchBy::Radius(r) => self.tree.search_radius(&data[i], r),
                };

                if cloud.len() == 1 {
                    return (i, None);
                }
                let cov = compute_covariance_matrix(&cloud);
                let cov = f3l_core::glam::Mat3::cast_from(cov.0);

                let eigen_set = if self.fast {
                    compute_eigen(cov)
                } else {
                    compute_eigen_rigorous(cov)
                };
                let eigenvector: Option<Vec3> = Some(eigen_set.minimal().eigenvector.into());

                (i, eigenvector)
            }).collect::<Vec<_>>();
        self.normals = vec![None; data.len()];
        normals.into_iter()
            .for_each(|(i, n)| {
                self.normals[i]=n;
            });
        true
    }

}
