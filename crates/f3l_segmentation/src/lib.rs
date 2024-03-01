use f3l_core::BasicFloat;

mod euclidean_cluster_extraction;
mod sac_segmentation;

#[derive(Debug, Clone, Copy, Default)]
pub struct F3lClusterParameter<T: BasicFloat> {
    pub tolerance: T,
    pub nb_in_tolerance: usize,
    pub min_nb_data: usize,
    pub max_nb_data: usize,
    pub max_nb_cluster: usize
}

impl<T: BasicFloat> F3lClusterParameter<T> {
    pub fn set_min_data_in_cluster(&mut self, nb: usize) {
        self.min_nb_data = nb;
    }
    pub fn set_max_data_in_cluster(&mut self, nb: usize) {
        self.max_nb_data = nb;
    }
    pub fn set_max_clusters(&mut self, nb: usize) {
        self.max_nb_cluster = nb;
    }
}

pub trait F3lCluster<'a, T: BasicFloat, P> {
    fn set_parameter(&mut self, parameter: F3lClusterParameter<T>);
    fn parameter(&self) -> F3lClusterParameter<T>;

    fn set_data(&mut self, data: &'a Vec<P>);
    fn clusters(&self) -> usize;
    /// vector of points of cluster nb
    fn extract(&mut self) -> Vec<Vec<usize>>;
    fn apply_extract(&mut self) -> bool;
    fn at(&self, id: usize) -> Result<Vec<P>, String>;
    fn max_cluster(&self) -> Vec<P>;
}

pub use euclidean_cluster_extraction::EuclideanClusterExtractor;
pub use sac_segmentation::*;