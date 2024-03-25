mod kd_features;
mod kd_leaf;
pub use kd_features::KdFeature;
pub use kd_leaf::KdLeaf;

use crate::{SearchBy, TreeHeapElement, TreeKnnResult, TreeRadiusResult, TreeResult, TreeSearch};
use f3l_core::BasicFloat;
use std::{cmp::Reverse, collections::BinaryHeap};

/// KD-Tree Implement
///
/// Use for any dimension of data.
/// Allow type which implement `Into<[T; D]>`
/// See more in `tests`.
///
/// `let mut tree = KdTree::<f32, 1>::new();`
/// Input:
/// * element type (like f32 or f64.. )
/// * Dimension: usize
///
/// # Examples
/// ```
/// use approx::assert_relative_eq;
/// use f3l_core::glam::{Vec2, Vec3};
/// use f3l_search_tree::*;
///
/// let mut tree = KdTree::<f32, 1>::new();
/// tree.set_data(&(0..10).map(|i| [i as f32]).collect::<Vec<_>>());
/// tree.build();
/// let result = tree.search_knn(&[5.1f32], 1);
/// let nearest_data = result[0].0[0];
/// let nearest_distance = result[0].1;
///
/// assert_relative_eq!(nearest_data, 5f32);
/// assert_relative_eq!(nearest_distance, 0.1f32);
/// ```
#[derive(Debug, Clone, Default)]
pub struct KdTree<T: BasicFloat, const D: usize> {
    pub root: Option<Box<KdLeaf>>,
    pub dim: usize,
    pub data: Vec<[T; D]>,
}

impl<T: BasicFloat, const D: usize> KdTree<T, D> {
    pub fn new() -> Self {
        Self {
            root: None,
            dim: D,
            data: vec![],
        }
    }

    pub fn set_data<P>(&mut self, data: &[P])
    where
        P: Into<[T; D]> + Clone + Copy,
    {
        self.data = data.iter().map(|p| (*p).into()).collect::<Vec<[T; D]>>();
    }

    pub fn build(&mut self) {
        if self.data.is_empty() {
            return;
        }
        self.root = Some(self.build_recursive(&mut (0..self.data.len()).collect::<Vec<usize>>()));
    }

    fn build_recursive(&self, indices: &mut [usize]) -> Box<KdLeaf> {
        let mut node = Box::<KdLeaf>::default();
        if indices.len() == 1 {
            node.feature = KdFeature::Leaf(indices[0]);
            return node;
        }
        let (split, index) = self.mean_split(indices);

        let mut data_l = indices[..index].to_owned();
        let mut data_r = indices[index..].to_owned();

        f3l_core::rayon::scope(|s| {
            s.spawn(|_| {
                node.left = Some(self.build_recursive(&mut data_l));
            });
            s.spawn(|_| {
                node.right = Some(self.build_recursive(&mut data_r));
            });
        });
        node.feature = split;

        node
    }

    fn mean_split(&self, indices: &mut [usize]) -> (KdFeature, usize) {
        // Compute mean value per dimension
        let factor = T::from(1.0f32 / indices.len() as f32).unwrap();
        let mut mean = vec![T::zero(); self.dim];
        indices.iter().for_each(|&i| {
            (0..self.dim).for_each(|j| {
                mean[j] += self.data[i][j] * factor;
            })
        });

        // Compute variance per dimension
        let mut var = vec![T::zero(); self.dim];
        indices.iter().for_each(|&i| {
            (0..self.dim).for_each(|j| {
                let dist = self.data[i][j] - mean[j];
                var[j] += dist * dist;
            })
        });
        // Choose the max variance dimension
        let mut split_dim = 0;
        (1..self.dim).for_each(|i| {
            if var[i] > var[split_dim] {
                split_dim = i;
            }
        });

        let split_val = mean[split_dim];
        let (lim1, lim2) = self.plane_split(indices, split_dim, split_val);

        let mut index: usize;
        let mid = indices.len() / 2;
        if lim1 > mid {
            index = lim1;
        } else if lim2 < mid {
            index = lim2;
        } else {
            index = mid;
        }
        if lim1 == indices.len() || lim2 == 0 {
            index = mid;
        }

        (
            KdFeature::Split((split_dim, split_val.to_f32().unwrap())),
            index,
        )
    }

    fn plane_split(&self, indices: &mut [usize], split_dim: usize, split_val: T) -> (usize, usize) {
        let mut left = 0;
        let mut right = indices.len() - 1;

        loop {
            while left <= right && self.data[indices[left]][split_dim] < split_val {
                left += 1;
            }
            while left < right && self.data[indices[right]][split_dim] >= split_val {
                right -= 1;
            }
            if left >= right {
                break;
            }
            indices.swap(left, right);
            left += 1;
            right -= 1;
        }
        let lim1 = left;
        right = indices.len() - 1;
        loop {
            while left <= right && self.data[indices[left]][split_dim] <= split_val {
                left += 1;
            }
            while left <= right && self.data[indices[right]][split_dim] > split_val {
                right -= 1;
            }
            if left >= right {
                break;
            }
            indices.swap(left, right);
            left += 1;
            right -= 1;
        }
        (lim1, left)
    }

    pub fn search<R: TreeResult, P>(&self, data: P, by: SearchBy, result: &mut R)
    where
        P: Into<[T; D]> + Clone + Copy,
    {
        let mut search_queue =
            BinaryHeap::with_capacity(std::cmp::max(10, (self.data.len() as f32).sqrt() as usize));
        if self.root.is_none() {
            return;
        }
        if let Some(root) = &self.root {
            self.search_(result, root, &data, by, 0.0, &mut search_queue);

            // Use Binary Heap to search the minimal node first
            while let Some(Reverse(node)) = search_queue.pop() {
                self.search_(result, node.raw, &data, by, node.order, &mut search_queue);
            }
        };
    }

    fn search_<'a, R: TreeResult, P>(
        &self,
        result: &mut R,
        node: &'a KdLeaf,
        data: &P,
        by: SearchBy,
        min_dist: f32,
        queue: &mut BinaryHeap<Reverse<TreeHeapElement<&'a KdLeaf, f32>>>,
    ) where
        P: Into<[T; D]> + Clone + Copy,
    {
        if result.worst() < min_dist {
            return;
        }
        let p: [T; D] = (*data).into();

        let near;
        let far;

        let d: T;
        match node.feature {
            KdFeature::Leaf(leaf) => {
                let dist = distance(&self.data[leaf], &p);
                result.add(leaf, dist.to_f32().unwrap());
                return;
            }
            KdFeature::Split((sp_dim, sp_val)) => {
                d = p[sp_dim] - T::from(sp_val).unwrap();
                if d < T::zero() {
                    near = &node.left;
                    far = &node.right;
                } else {
                    near = &node.right;
                    far = &node.left;
                }
            }
        };

        if let Some(far) = far {
            let add_far = match by {
                SearchBy::Count(_) => {
                    if !result.is_full() {
                        true
                    } else {
                        d * d < T::from(result.worst() + f32::EPSILON).unwrap()
                    }
                }
                SearchBy::Radius(r) => d * d <= T::from(r).unwrap(),
            };
            if add_far {
                queue.push(Reverse(TreeHeapElement {
                    raw: far,
                    order: min_dist + (d * d).to_f32().unwrap(),
                }));
            }
        }

        if let Some(near) = near {
            self.search_(result, near, data, by, min_dist, queue);
        }
    }
}

#[inline]
fn distance<T: BasicFloat, const D: usize>(a: &[T; D], b: &[T; D]) -> T {
    a.iter()
        .zip(b)
        .fold(T::zero(), |acc, (a, b)| acc + (*a - *b).powi(2))
}
impl<P, T: BasicFloat, const D: usize> TreeSearch<P> for KdTree<T, D>
where
    P: Into<[T; D]> + Send + Sync + Clone + Copy,
    [T; D]: Into<P>,
{
    fn search_knn(&self, point: &P, k: usize) -> Vec<(P, f32)> {
        let by = if k == 0 {
            SearchBy::Count(1)
        } else {
            SearchBy::Count(k)
        };
        let mut result = TreeKnnResult::new(k);
        self.search(*point, by, &mut result);
        result
            .result()
            .iter()
            .map(|&(i, d)| (self.data[i].into(), d.sqrt()))
            .collect::<Vec<(P, f32)>>()
    }

    fn search_radius(&self, point: &P, radius: f32) -> Vec<P> {
        let by = if radius == 0.0 {
            SearchBy::Count(1)
        } else {
            SearchBy::Radius(radius * radius)
        };
        let mut result = TreeRadiusResult::new(radius * radius);
        self.search(*point, by, &mut result);
        result.data.iter().map(|&i| self.data[i].into()).collect()
    }

    fn search_knn_ids(&self, point: &P, k: usize) -> Vec<usize> {
        let by = if k == 0 {
            SearchBy::Count(1)
        } else {
            SearchBy::Count(k)
        };
        let mut result = TreeKnnResult::new(k);
        self.search(*point, by, &mut result);
        result.data.iter().map(|&(i, _)| i).collect()
    }

    fn search_radius_ids(&self, point: &P, radius: f32) -> Vec<usize> {
        let by = if radius == 0.0 {
            SearchBy::Count(1)
        } else {
            SearchBy::Radius(radius * radius)
        };
        let mut result = TreeRadiusResult::new(radius * radius);
        self.search(*point, by, &mut result);
        result.data
    }
}
