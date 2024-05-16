use std::ops::Index;

use crate::{F3lFilter, F3lFilterInverse};
use f3l_core::rayon::prelude::*;
use f3l_core::{serde::{self, Serialize, Deserialize}, BasicFloat};
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate="self::serde")]
pub struct StatisticalOutlierRemoval<'a, P, T: BasicFloat, const D: usize>
where
    P: Into<[T; D]> + Clone + Copy + Index<usize, Output = T>,
{
    pub negative: bool,
    pub multiply: T,
    pub k_neighbors: usize,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    tree: KdTree<'a, T, P>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    inlier: Vec<bool>,
}

impl<'a, P, T: BasicFloat, const D: usize> StatisticalOutlierRemoval<'a, P, T, D>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync + Index<usize, Output = T>,
    [T; D]: Into<P>,
{
    pub fn new(multiply: T, k_neighbors: usize) -> Self {
        Self {
            negative: false,
            multiply,
            k_neighbors,
            tree: KdTree::<T, P>::new(D),
            inlier: vec![],
        }
    }

    #[inline]
    fn ok(&self, is_inlier: bool) -> bool {
        (!is_inlier && self.negative) || (is_inlier && !self.negative)
    }
}

impl<'a, P, T: BasicFloat, const D: usize> F3lFilterInverse for StatisticalOutlierRemoval<'a, P, T, D>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync + Index<usize, Output = T>,
    [T; D]: Into<P>,
{
    fn set_negative(&mut self, negative: bool) {
        self.negative = negative;
    }
}

impl<'a, P, T: BasicFloat, const D: usize> F3lFilter<'a, P, D>
    for StatisticalOutlierRemoval<'a, P, T, D>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync + Index<usize, Output = T>,
    [T; D]: Into<P>,
{
    fn filter(&mut self, data: &'a Vec<P>) -> Vec<usize> {
        self.apply_filter(data);

        self.inlier
            .iter()
            .enumerate()
            .filter(|&(_, f)| self.ok(*f))
            .map(|(i, _)| i)
            .collect()
    }

    fn filter_instance(&mut self, data: &'a Vec<P>) -> Vec<P> {
        self.apply_filter(data);

        self.inlier
            .iter()
            .enumerate()
            .filter(|&(_, f)| self.ok(*f))
            .map(|(i, _)| data[i])
            .collect()
    }

    fn apply_filter(&mut self, data: &'a Vec<P>) -> bool {
        if data.is_empty() {
            return false;
        }
        // Check Tree dimension correct, cause skip deserialize would be 0.
        if self.tree.dim != D {
            self.tree = KdTree::<T, P>::new(D);
        }
        self.tree.set_data(data);
        
        self.tree.build();

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

#[test]
fn serde() {
    let model = StatisticalOutlierRemoval::<[f32; 3], f32, 3>::new(2f32, 20_usize);
    let content = serde_json::to_string(&model).unwrap();
    println!("{}", content);

    let text = r#"{
        "negative":false,
        "multiply":2.0,
        "k_neighbors":20
    }"#;
    let model_de: StatisticalOutlierRemoval<[f32;3],f32,3> = serde_json::from_str(text).unwrap();
    assert_eq!(model.negative, model_de.negative);
    assert_eq!(model.multiply, model_de.multiply);
    assert_eq!(model.k_neighbors, model_de.k_neighbors);
}