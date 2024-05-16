use std::ops::Index;

use f3l_core::{
    serde::{self, Deserialize, Serialize},
    BasicFloat,
};
use f3l_search_tree::{KdTree, SearchBy, TreeRadiusResult, TreeResult};

use crate::{F3lCluster, F3lClusterParameter};

/// Euclidean Cluster Extractor
/// use [`KdTree`] to search neighbors of radius.
///
/// # Examples
/// ```
/// let vertices = load_ply("../../data/table_remove_plane.ply");
/// let parameter = F3lClusterParameter {
///     tolerance: 0.02f32,
///     nb_in_tolerance: 1,
///     min_nb_data: 100,
///     max_nb_data: 25000,
///     max_nb_cluster: 5,
/// };
/// let mut extractor = EuclideanClusterExtractor::with_data(parameter, &vertices);
/// let clusters = extractor.extract();
/// let clusters = (0..clusters.len())
///     .map(|i| extractor.at(i).unwrap())
///     .collect::<Vec<_>>();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct EuclideanClusterExtractor<'a, T, P, const D: usize>
where
    T: BasicFloat,
    P: Into<[T; D]> + Clone + Copy + Index<usize, Output = T>,
{
    pub parameter: F3lClusterParameter<T>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    tree: KdTree<'a, T, P>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    clusters: Vec<Vec<usize>>,
}

impl<'a, T, P, const D: usize> EuclideanClusterExtractor<'a, T, P, D>
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
}

impl<'a, T, P, const D: usize> F3lCluster<'a, T, P> for EuclideanClusterExtractor<'a, T, P, D>
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

    fn clusters(&self) -> usize {
        self.clusters.len()
    }

    fn extract(&mut self, data: &'a Vec<P>) -> Vec<Vec<usize>> {
        if data.is_empty() {
            return vec![];
        }
        if !self.apply_extract(data) {
            return vec![];
        }

        self.clusters.clone()
    }

    fn apply_extract(&mut self, data: &'a Vec<P>) -> bool {
        if self.tree.dim != D {
            self.tree = KdTree::<T, P>::new(D);
        }
        self.tree.set_data(data);
        self.tree.build();
        let data = &self.tree.data;

        let radius = self.parameter.tolerance.to_f32().unwrap();
        let radius = radius * radius;
        let mut result = TreeRadiusResult::new(radius);
        let by = SearchBy::Radius(radius);
        let mut visited = vec![false; data.len()];
        (0..data.len()).for_each(|i| {
            if visited[i] {
                return;
            }

            let mut pts = 0usize;
            let mut cluster = vec![i];
            visited[i] = true;

            while pts < cluster.len() {
                result.clear();
                self.tree.search(data[cluster[pts]], by, &mut result);
                let ids = result.result();
                if ids.is_empty() {
                    pts += 1;
                    continue;
                }
                ids.into_iter().for_each(|id| {
                    if visited[id] {
                        return;
                    }
                    cluster.push(id);
                    visited[id] = true;
                });
                pts += 1;
            }

            if cluster.len() >= self.parameter.min_nb_data
                && cluster.len() < self.parameter.max_nb_data
            {
                cluster.sort();
                self.clusters.push(cluster);
            }
        });
        self.clusters
            .sort_by(|a, b| b.len().partial_cmp(&a.len()).unwrap());
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
        let data = cluster
            .iter()
            .map(|&i| self.tree.data[i])
            .collect::<Vec<_>>();
        Ok(data)
    }

    fn max_cluster(&self) -> Vec<P> {
        self.at(0).unwrap()
    }
}

#[test]
fn serde() {
    let cluster: EuclideanClusterExtractor<f32, [f32; 3], 3> =
        EuclideanClusterExtractor::new(F3lClusterParameter::default());
    let text = r#"{
        "parameter":{
            "tolerance":0.0,
            "nb_in_tolerance":0,
            "min_nb_data":0,
            "max_nb_data":0,
            "max_nb_cluster":0
        }
    }"#;
    let cluster_serde: EuclideanClusterExtractor<f32, [f32; 3], 3> =
        serde_json::from_str(&text).unwrap();
    assert_eq!(cluster.parameter, cluster_serde.parameter);
}
