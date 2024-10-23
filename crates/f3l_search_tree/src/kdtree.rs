mod kd_features;
mod kd_leaf;
pub use kd_features::KdFeature;
pub use kd_leaf::KdLeaf;

#[cfg(all(feature = "pure", not(feature = "core")))]
use crate::{
    serde::{self, Deserialize, Serialize},
    BasicFloat,
};
use crate::{SearchBy, TreeHeapElement, TreeKnnResult, TreeRadiusResult, TreeResult, TreeSearch};
#[cfg(all(feature = "core", not(feature = "pure")))]
use f3l_core::{
    rayon,
    serde::{self, Deserialize, Serialize},
    BasicFloat,
};
use std::{borrow::Cow, cmp::Reverse, collections::BinaryHeap, ops::Index};

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
/// let data = (0..10).map(|i| [i as f32]).collect::<Vec<_>>();
/// let mut tree = KdTree::with_data(&data);
/// tree.build();
/// let result = tree.search_knn(&[5.1f32], 1);
/// let nearest_data = result[0].0[0];
/// let nearest_distance = result[0].1;
///
/// assert_relative_eq!(nearest_data, 5f32);
/// assert_relative_eq!(nearest_distance, 0.1f32);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct KdTree<'a, T: BasicFloat, P>
where
    P: Index<usize, Output = T> + Clone + Copy,
{
    pub root: Option<Box<KdLeaf>>,
    pub dim: usize,
    pub data: Cow<'a, Vec<P>>,
    pub ignores: Vec<usize>,
    pub enable_ignore: bool,
}

impl<'a, T: BasicFloat, P> KdTree<'a, T, P>
where
    P: Index<usize, Output = T> + Clone + Copy + Send + Sync,
{
    pub fn new(dim: usize) -> Self {
        Self {
            root: None,
            dim,
            data: Cow::Owned(vec![]),
            ignores: vec![],
            enable_ignore: false,
        }
    }

    pub fn with_data(dim: usize, data: &'a Vec<P>) -> Self {
        Self {
            root: None,
            dim,
            data: Cow::Borrowed(data),
            ignores: vec![],
            enable_ignore: false,
        }
    }

    pub fn clear(&mut self) {
        // self.data.clear();
        self.root = None;
    }

    pub fn set_data(&mut self, data: &'a Vec<P>) {
        self.clear();
        self.data = Cow::Borrowed(data);
    }

    pub fn build(&mut self) {
        let n = self.data.len();
        self.root = Some(self.build_recursive(&mut (0..n).collect::<Vec<usize>>()));
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

        (node.left, node.right) = rayon::join(
            || Some(self.build_recursive(&mut data_l)),
            || Some(self.build_recursive(&mut data_r)),
        );
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
            while left < right && self.data[indices[right]][split_dim] > split_val {
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

    pub fn search<R: TreeResult>(&self, data: P, by: SearchBy, result: &mut R) {
        let mut search_queue =
            BinaryHeap::with_capacity(std::cmp::max(10, (self.data.len() as f32).sqrt() as usize));

        if self.root.is_none() {
            return;
        }
        if let Some(root) = &self.root {
            self.search_(
                result,
                root,
                &data,
                by,
                if result.is_farthest() { f32::MAX } else { 0.0 },
                &mut search_queue,
            );

            while let Some(Reverse(node)) = search_queue.pop() {
                self.search_(result, node.raw, &data, by, node.order, &mut search_queue)
            }
        };
    }

    fn search_<R: TreeResult>(
        &self,
        result: &mut R,
        node: &'a KdLeaf,
        data: &P,
        by: SearchBy,
        min_dist: f32,
        // queue: &mut BinaryHeap<SearchQueue<TreeHeapElement<&'a Box<KdLeaf>, f32>>>,
        queue: &mut BinaryHeap<Reverse<TreeHeapElement<&'a Box<KdLeaf>, f32>>>,
    ) {
        let is_farthest = result.is_farthest();
        if match is_farthest {
            true => result.worst() > min_dist,
            false => result.worst() < min_dist,
        } {
            return;
        }
        // let p: [T; D] = (*data).into();
        let p = data;

        let near;
        let far;

        let d: T;
        match node.feature {
            KdFeature::Leaf(leaf) => {
                if self.enable_ignore && self.ignores.contains(&leaf) {
                    return;
                }
                let dist = distance(self.data[leaf], *p, self.dim);
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
        let (near, far) = if is_farthest {
            (far, near)
        } else {
            (near, far)
        };

        if let Some(far) = far {
            let add_far = match by {
                SearchBy::Count(_) => {
                    if !result.is_full() {
                        true
                    } else {
                        match is_farthest {
                            true => d * d > T::from(result.worst() + f32::EPSILON).unwrap(),
                            false => d * d < T::from(result.worst() + f32::EPSILON).unwrap(),
                        }
                    }
                }
                SearchBy::Radius(r) => d * d <= T::from(r).unwrap(),
            };
            if add_far {
                let node = TreeHeapElement {
                    raw: far,
                    order: min_dist + (d * d).to_f32().unwrap(),
                };
                queue.push(Reverse(node));
            }
        }

        if let Some(near) = near {
            self.search_(result, near, data, by, min_dist, queue);
        }
    }
}

#[inline]
fn distance<T: BasicFloat, P>(a: P, b: P, dim: usize) -> T
where
    P: Index<usize, Output = T> + Copy,
{
    (0..dim).fold(T::zero(), |acc, i| acc + (a[i] - b[i]).powi(2))
}

impl<'a, T: BasicFloat, P> TreeSearch<P> for KdTree<'a, T, P>
where
    P: Send + Sync + Clone + Copy + Index<usize, Output = T>,
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
            .map(|&(i, d)| (self.data[i], d.sqrt()))
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
        result.data.iter().map(|&i| self.data[i]).collect()
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

    fn add_ignore(&mut self, idx: usize) {
        self.ignores.push(idx);
    }

    fn add_ignores(&mut self, idx: &[usize]) {
        idx.iter().for_each(|&i| self.ignores.push(i));
    }

    fn set_ignore(&mut self, enable: bool) {
        self.enable_ignore = enable;
    }
}
