use f3l_core::{rayon::iter::FromParallelIterator, BasicFloat};
use f3l_search_tree::{KdTree, SearchBy, TreeRadiusResult, TreeResult};

use crate::{F3lCluster, F3lClusterParameter};

pub struct DBScan<'a, T, P, const D: usize>
where
    T: BasicFloat,
    P: Into<[T; D]> + Clone + Copy,
{
    pub parameter: F3lClusterParameter<T>,
    data: Option<&'a [P]>,
    tree: KdTree<T, D>,
    clusters: Vec<Vec<usize>>,
}

impl<'a, T, P, const D: usize> DBScan<'a, T, P, D>
where
    T: BasicFloat,
    P: Into<[T; D]> + Clone + Copy + Send + Sync,
    [T; D]: Into<P>,
{
    pub fn new(parameter: F3lClusterParameter<T>) -> Self {
        Self {
            parameter,
            data: None,
            tree: KdTree::<T, D>::new(),
            clusters: vec![],
        }
    }

    pub fn with_data(parameter: F3lClusterParameter<T>, data: &'a [P]) -> Self {
        let mut entity = Self::new(parameter);
        entity.set_data(data);
        entity
    }
}

impl<'a, T, P, const D: usize> F3lCluster<'a, T, P> for DBScan<'a, T, P, D>
where
    T: BasicFloat,
    P: Into<[T; D]> + Clone + Copy + Send + Sync,
    [T; D]: Into<P>,
{
    fn set_parameter(&mut self, parameter: F3lClusterParameter<T>) {
        self.parameter = parameter;
    }

    fn parameter(&self) -> F3lClusterParameter<T> {
        self.parameter
    }

    fn set_data(&mut self, data: &'a [P]) {
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
        let data = if let Some(data) = self.data {
            cluster.iter().map(|&i| data[i]).collect::<Vec<_>>()
        } else {
            return Err("Data corrupted".to_owned());
        };
        Ok(data)
    }

    fn max_cluster(&self) -> Vec<P> {
        self.at(0).unwrap()
    }
}
