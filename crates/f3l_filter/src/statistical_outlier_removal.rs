use crate::F3lFilter;
use f3l_core::rayon::prelude::*;
use f3l_core::BasicFloat;
use f3l_search_tree::{KdTree, TreeSearch};

/// Compute k-neighbors of all points, then compute mean and variance
/// filter out mean +- multiply * std
/// 
/// # Examples
/// ```
/// let vertices = load_ply("../../data/table_scene_lms400.ply");
/// // To filter k-neighbor=50 and mean +- 1 * std
/// let mut filter = StatisticalOutlierRemoval::with_data(1., 50, &vertices);
/// filter.set_negative(true);
/// let out = filter.filter_instance();
/// ```
pub struct StatisticalOutlierRemoval<'a, P, T: BasicFloat, const D: usize>
where
    P: Into<[T; D]> + Clone + Copy,
{
    pub negative: bool,
    pub multiply: T,
    pub k_neighbors: usize,
    data: Option<&'a [P]>,
    tree: KdTree<T, D>,
    inlier: Vec<bool>,
}

impl<'a, P, T: BasicFloat, const D: usize> StatisticalOutlierRemoval<'a, P, T, D>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync,
    [T; D]: Into<P>,
{
    pub fn new(multiply: T, k_neighbors: usize) -> Self {
        Self {
            negative: false,
            multiply,
            k_neighbors,
            data: None,
            tree: KdTree::<T, D>::new(),
            inlier: vec![],
        }
    }

    pub fn with_data(multiply: T, k_neighbors: usize, data: &'a [P]) -> Self {
        let mut entity = Self::new(multiply, k_neighbors);
        entity.set_data(data);
        entity
    }

    #[inline]
    fn ok(&self, is_inlier: bool) -> bool {
        (!is_inlier && self.negative) || (is_inlier && !self.negative)
    }
}

impl<'a, P, T: BasicFloat, const D: usize> F3lFilter<'a, P>
    for StatisticalOutlierRemoval<'a, P, T, D>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync,
    [T; D]: Into<P>,
{
    fn set_negative(&mut self, negative: bool) {
        self.negative = negative;
    }

    fn set_data(&mut self, data: &'a [P]) {
        self.data = Some(data);
        self.tree.set_data(data);
    }

    fn filter(&mut self) -> Vec<usize> {
        self.apply_filter();

        self.inlier
            .iter()
            .enumerate()
            .filter(|&(_, f)| self.ok(*f))
            .map(|(i, _)| i)
            .collect()
    }

    fn filter_instance(&mut self) -> Vec<P> {
        self.apply_filter();

        let data = self.data.unwrap();
        self.inlier
            .iter()
            .enumerate()
            .filter(|&(_, f)| self.ok(*f))
            .map(|(i, _)| data[i])
            .collect()
    }

    fn apply_filter(&mut self) -> bool {
        if self.tree.data.is_empty() {
            return false;
        }
        self.tree.build();

        let data = if let Some(data) = self.data {
            data
        } else {
            return false;
        };

        use std::sync::{Arc, Mutex};
        let nb_valid = Arc::new(Mutex::new(0usize));

        let distances = data
            // .iter()
            .par_iter()
            .enumerate()
            .map(|(i, v)| {
                let out = self.tree.search_knn(v, self.k_neighbors);
                if out.is_empty() {
                    return (i, T::zero());
                }
                {
                    let mut lock = nb_valid.lock().unwrap();
                    *lock += 1usize;
                }

                let sum = out.iter().map(|(_, o)| (*o) * (*o)).sum::<f32>();
                (i, T::from(sum).unwrap())
            })
            .collect::<Vec<_>>();
        let nb_valid = *(nb_valid.lock().unwrap());
        let nb_valid_t = T::from(nb_valid).unwrap();

        let (sum, sq_sum) = distances
            .iter()
            .fold((T::zero(), T::zero()), |(sum, sq_sum), &(_, d)| {
                (sum + d, sq_sum + d * d)
            });
        let mean = sum / nb_valid_t;
        let variance = (sq_sum - sum * sum / nb_valid_t) / (nb_valid_t - T::one());
        let std_dev = variance.sqrt();

        let threshold = mean + self.multiply * std_dev;

        self.inlier = vec![false; data.len()];
        distances.iter().for_each(|&(i, d)| {
            if d <= threshold {
                self.inlier[i] = true;
            }
        });

        true
    }
}
