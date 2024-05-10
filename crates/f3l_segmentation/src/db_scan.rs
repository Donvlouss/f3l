use std::ops::Index;

use f3l_core::{rayon::iter::FromParallelIterator, BasicFloat, serde::{self, Deserialize, Serialize}};
use f3l_search_tree::{KdTree, SearchBy, TreeRadiusResult, TreeResult};

use crate::{F3lCluster, F3lClusterParameter};

/// Euclidean Cluster Extractor
/// use [`KdTree`] to search neighbors of radius.
///
/// # Examples
/// ```
/// let vertices = load_ply("../../data/table_remove_plane.ply");
///
/// let parameter = F3lClusterParameter {
///     tolerance: 0.02f32,
///     nb_in_tolerance: 20,
///     min_nb_data: 100,
///     max_nb_data: vertices.len(),
///     max_nb_cluster: 5,
/// };
/// let mut extractor = DBScan::with_data(parameter, &vertices);
/// let clusters = extractor.extract();
/// let clusters = (0..clusters.len())
///     .map(|i| extractor.at(i).unwrap())
///     .collect::<Vec<_>>();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate="self::serde")]
pub struct DBScan<'a, T, P, const D: usize>
where
    T: BasicFloat,
    P: Into<[T; D]> + Clone + Copy + Index<usize, Output = T>,
{
    pub parameter: F3lClusterParameter<T>,
    tree: KdTree<'a, T, P>,
    clusters: Vec<Vec<usize>>,
}

impl<'a, T, P, const D: usize> DBScan<'a, T, P, D>
where
    T: BasicFloat,
    P: Into<[T; D]> + Clone + Copy + Send + Sync + Index<usize, Output = T>,
    [T; D]: Into<P>,
{
    pub fn new(parameter: F3lClusterParameter<T>) -> Self {
        Self {
            parameter,
            tree: KdTree::<T, P>::new(D),
            clusters: vec![],
        }
    }

    pub fn with_data(parameter: F3lClusterParameter<T>, data: &'a Vec<P>) -> Self {
        let mut entity = Self::new(parameter);
        entity.set_data(data);
        entity
    }
}

impl<'a, T, P, const D: usize> F3lCluster<'a, T, P> for DBScan<'a, T, P, D>
where
    T: BasicFloat,
    P: Into<[T; D]> + Clone + Copy + Send + Sync + Index<usize, Output = T>,
    [T; D]: Into<P>,
{
    fn set_parameter(&mut self, parameter: F3lClusterParameter<T>) {
        self.parameter = parameter;
    }

    fn parameter(&self) -> F3lClusterParameter<T> {
        self.parameter
    }

    fn set_data(&mut self, data: &'a Vec<P>) {
        self.tree.set_data(data);
    }

    fn clusters(&self) -> usize {
        self.clusters.len()
    }

    fn extract(&mut self) -> Vec<Vec<usize>> {
        if !self.apply_extract() {
            return vec![];
        }

        self.clusters.clone()
    }

    fn apply_extract(&mut self) -> bool {
        self.tree.build();
        let data = &self.tree.data;

        let radius = self.parameter.tolerance.to_f32().unwrap();
        let radius = radius * radius;
        let mut result = TreeRadiusResult::new(radius);
        let mut result_inner = TreeRadiusResult::new(radius);
        let by = SearchBy::Radius(radius);
        let mut visited = vec![-2_i32; data.len()];

        let mut clusters: Vec<Vec<usize>> = vec![];
        (0..data.len()).for_each(|i| {
            if visited[i] >= -1 {
                return;
            }
            result.clear();
            self.tree.search(data[i], by, &mut result);

            if result.data.len() < self.parameter.nb_in_tolerance {
                visited[i] = -1;
                return;
            }
            let mut cluster =
                std::collections::BTreeSet::<usize>::from_par_iter(result.data.clone());
            visited[i] = clusters.len() as i32;

            let mut ptr = 0_usize;
            while ptr < result.data.len() {
                if visited[result.data[ptr]] >= -1 {
                    ptr += 1;
                    continue;
                }

                result_inner.clear();
                self.tree
                    .search(data[result.data[ptr]], by, &mut result_inner);
                if result_inner.data.len() < self.parameter.nb_in_tolerance {
                    visited[result.data[ptr]] = -1;
                    ptr += 1;
                    continue;
                }

                visited[result.data[ptr]] = clusters.len() as i32;
                cluster.insert(result.data[ptr]);

                result_inner.data.iter().for_each(|&iii| {
                    if visited[iii] < -1 && !cluster.contains(&iii) {
                        result.data.push(iii);
                    }
                });
                ptr += 1;
            }
            if cluster.len() >= self.parameter.min_nb_data
                && cluster.len() <= self.parameter.max_nb_data
            {
                clusters.push(cluster.into_iter().collect());
            }
        });
        clusters.sort_by(|a, b| b.len().partial_cmp(&a.len()).unwrap());
        self.clusters = clusters
            .into_iter()
            .take(self.parameter.max_nb_cluster)
            .collect();

        true
    }

    fn at(&self, id: usize) -> Result<Vec<P>, String> {
        if id > self.clusters.len() {
            return Err(format!(
                "Out of Range, available to {}",
                self.clusters.len() - 1
            ));
        }
        let cluster = &self.clusters[id];
        let data = cluster.iter().map(|&i| self.tree.data[i]).collect::<Vec<_>>();
        Ok(data)
    }

    fn max_cluster(&self) -> Vec<P> {
        self.at(0).unwrap()
    }
}
