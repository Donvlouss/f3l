use f3l_core::{
    serde::{self, Deserialize, Serialize},
    BasicFloat,
};

mod db_scan;
mod euclidean_cluster_extraction;
mod sac_segmentation;

/// Cluster Extractor parameter
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(crate = "self::serde")]
pub struct F3lClusterParameter<T: BasicFloat> {
    /// `K`-NN or `Radius` search
    pub tolerance: T,
    /// K-`NN` or `points` in Radius search
    pub nb_in_tolerance: usize,
    /// Add to clusters when numbers of cluster more than this
    pub min_nb_data: usize,
    /// Add to clusters when numbers of cluster smaller than this
    pub max_nb_data: usize,
    /// Set maximum numbers of clusters
    pub max_nb_cluster: usize,
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

/// A trait fo cluster methods.
pub trait F3lCluster<'a, T: BasicFloat, P> {
    /// Set [`F3lClusterParameter`]
    fn set_parameter(&mut self, parameter: F3lClusterParameter<T>);
    /// Get [`F3lClusterParameter`]
    fn parameter(&self) -> F3lClusterParameter<T>;

    // fn set_data(&mut self, data: &'a Vec<P>);
    fn clusters(&self) -> usize;
    /// vector of points of clusters
    fn extract(&mut self, data: &'a [P]) -> Vec<Vec<usize>>;
    /// Use `extract` directly, not call this.
    fn apply_extract(&mut self, data: &'a [P]) -> bool;
    /// Get data from Target cluster
    fn at(&self, id: usize) -> Result<Vec<P>, String>;
    /// Get maximum data one of clusters
    fn max_cluster(&self) -> Vec<P>;
}

pub use db_scan::*;
pub use euclidean_cluster_extraction::EuclideanClusterExtractor;
pub use sac_segmentation::*;
