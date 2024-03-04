use f3l_core::BasicFloat;
use f3l_search_tree::{
    KdTree,
    TreeRadiusResult,
    TreeResult,
    SearchBy
};

use crate::{
    F3lCluster,
    F3lClusterParameter,
};

pub struct EuclideanClusterExtractor<'a, T, P, const D: usize>
where
    T: BasicFloat,
    P: Into<[T; D]> + Clone + Copy
{
    pub parameter: F3lClusterParameter<T>,
    data: Option<&'a Vec<P>>,
    tree: KdTree<T, D>,
    clusters: Vec<Vec<usize>>
}

impl<'a, T, P, const D: usize> EuclideanClusterExtractor<'a, T, P, D>
where
    T: BasicFloat,
    P: Into<[T; D]> + Clone + Copy + Send + Sync,
    [T; D]: Into<P>
{
    pub fn new(parameter: F3lClusterParameter<T>) -> Self {
        Self {
            parameter,
            data: None,
            tree: KdTree::<T, D>::new(),
            clusters: vec![]
        }
    }

    pub fn with_data(parameter: F3lClusterParameter<T>, data: &'a Vec<P>) -> Self {
        let mut entity = Self::new(parameter);
        entity.set_data(data);
        entity
    }
}

impl<'a, T, P, const D: usize> F3lCluster<'a, T, P> for EuclideanClusterExtractor<'a, T, P, D>
where
    T: BasicFloat,
    P: Into<[T; D]> + Clone + Copy + Send + Sync,
    [T; D]: Into<P>
{
    fn set_parameter(&mut self, parameter: F3lClusterParameter<T>) {
        self.parameter = parameter;
    }

    fn parameter(&self) -> F3lClusterParameter<T> {
        self.parameter
    }

    fn set_data(&mut self, data: &'a Vec<P>) {
        self.data = Some(data);
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
        let data = if let Some(data) = self.data {
            data
        } else {
            return false;
        };
        self.tree.build();

        let radius = self.parameter.tolerance.to_f32().unwrap();
        let radius = radius * radius;
        let mut result = TreeRadiusResult::new(radius);
        let by = SearchBy::Radius(radius);
        let mut visited = vec![false; data.len()];
        (0..data.len())
            .for_each(|i| {
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
                        pts+=1;
                        continue;
                    }
                    ids.into_iter()
                        .for_each(|id| {
                            if visited[id] {
                                return;
                            }
                            cluster.push(id);
                            visited[id] = true;
                        });
                    pts+=1;
                }

                if cluster.len() >= self.parameter.min_nb_data &&
                    cluster.len() < self.parameter.max_nb_data
                {
                    cluster.sort();
                    self.clusters.push(cluster);
                }
            });
        self.clusters.sort_by(|a, b| b.len().partial_cmp(&a.len()).unwrap());
        true
    }

    fn at(&self, id: usize) -> Result<Vec<P>, String> {
        if id > self.clusters.len() {
            return Err(format!("Out of Range, available to {}", self.clusters.len()-1));
        }
        let cluster = &self.clusters[id];
        let data = if let Some(data) = self.data {
            cluster.iter()
                .map(|&i| data[i]).collect::<Vec<_>>()
        } else {
            return Err("Data corrupted".to_owned());
        };
        Ok(data)
    }

    fn max_cluster(&self) -> Vec<P> {
        self.at(0).unwrap()
    }
}
