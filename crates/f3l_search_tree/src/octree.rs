mod oc_features;
mod oc_leaf;

#[cfg(all(feature = "pure", not(feature = "core")))]
use crate::{get_minmax, BasicFloat};
#[cfg(all(feature = "core", not(feature = "pure")))]
use f3l_core::{get_minmax, serde, BasicFloat};
pub use oc_features::*;
pub use oc_leaf::*;
use std::{borrow::Cow, cmp::Reverse, collections::BinaryHeap, ops::Index};

use crate::{SearchBy, TreeHeapElement, TreeKnnResult, TreeRadiusResult, TreeResult, TreeSearch};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct OcTree<'a, T: BasicFloat, P>
where
    P: Index<usize, Output = T> + Clone + Copy + Serialize,
{
    pub bounds: Option<(P, P)>,
    pub data: Cow<'a, Vec<P>>,
    pub max_points: usize,
    pub depth: usize,
    pub nodes: Vec<OcLeaf<T>>,
    pub ignores: Vec<usize>,
    pub enable_ignore: bool,
}

// Build
impl<'a, T: BasicFloat, P> OcTree<'a, T, P>
where
    P: Into<[T; 3]> + Index<usize, Output = T> + Clone + Copy + Serialize,
    [T; 3]: Into<P>,
{
    pub fn new(max_points: usize, depth: usize) -> Self {
        Self {
            bounds: None,
            data: Cow::Owned(vec![]),
            max_points,
            depth,
            nodes: Vec::with_capacity(8_usize.pow(3)),
            ignores: vec![],
            enable_ignore: false,
        }
    }

    pub fn with_data(data: &'a Vec<P>, max_points: usize, depth: usize) -> Self {
        Self {
            bounds: None,
            data: Cow::Borrowed(data),
            max_points,
            depth,
            nodes: Vec::with_capacity(8_usize.pow(depth as u32)),
            ignores: vec![],
            enable_ignore: false,
        }
    }

    pub fn clear(&mut self) {
        self.bounds = None;
        self.nodes.clear();
    }

    pub fn set_data(&mut self, data: &'a Vec<P>) {
        self.clear();
        if !data.is_empty() {
            self.data = Cow::Borrowed(data);
        }
    }

    fn compute_bounds(&mut self) {
        let bdx = get_minmax(&self.data);
        self.bounds = Some(bdx);
    }

    pub fn build(&mut self) {
        self.compute_bounds();
        let root = OcLeaf {
            root: None,
            position: 0,
            lower: self.bounds.unwrap().0.into(),
            upper: self.bounds.unwrap().1.into(),
            feature: OcFeature::Leaf,
            points: vec![],
        };
        self.nodes.push(root);

        (0..self.data.len()).for_each(|i| {
            self.insert(i, 0, 0);
        });
    }

    ///
    fn insert(&mut self, i_point: usize, depth: usize, i_node: usize) {
        if depth == self.depth {
            let node = &mut self.nodes[i_node];
            node.feature = OcFeature::Leaf;
            node.points.push(i_point);
            return;
        }

        match self.nodes[i_node].feature {
            OcFeature::Split(nodes) => {
                // Unwrap the option, cause this node must be a `Split` type.
                let id = self.nodes[i_node].locate_at(self.data[i_point]).unwrap();
                self.insert(i_point, depth + 1, nodes[id]);
            }
            OcFeature::Leaf => {
                let full = self.nodes[i_node].points.len() >= self.max_points;
                if full {
                    // Set this node to `Split`, clone the vector of points and remove it.
                    let ids = self.create_8_nodes(
                        i_node,
                        (self.nodes[i_node].lower, self.nodes[i_node].upper),
                    );
                    let node = &mut self.nodes[i_node];

                    // Max number of node is reached. Transfer this node from `Leaf` to `Split`
                    node.feature = OcFeature::Split(ids);
                    let points = node.points.clone();
                    node.points = vec![];

                    points.into_iter().for_each(|ii_point| {
                        self.insert(ii_point, depth, i_node);
                    });
                    self.insert(i_point, depth, i_node);
                } else {
                    let node = &mut self.nodes[i_node];
                    node.points.push(i_point);
                }
            }
        };
    }

    fn create_8_nodes(&mut self, i_root: usize, bounds: ([T; 3], [T; 3])) -> [usize; 8] {
        let n = self.nodes.len();
        let out = [n, n + 1, n + 2, n + 3, n + 4, n + 5, n + 6, n + 7];

        let mid = [
            (bounds.1[0] + bounds.0[0]) / T::from(2f32).unwrap(),
            (bounds.1[1] + bounds.0[1]) / T::from(2f32).unwrap(),
            (bounds.1[2] + bounds.0[2]) / T::from(2f32).unwrap(),
        ];
        let half = [
            (bounds.1[0] - bounds.0[0]) / T::from(2f32).unwrap(),
            (bounds.1[1] - bounds.0[1]) / T::from(2f32).unwrap(),
            (bounds.1[2] - bounds.0[2]) / T::from(2f32).unwrap(),
        ];
        let mid_0 = [mid[0], mid[1], mid[2] - half[2]];
        let mid_1 = [mid[0], mid[1], mid[2] + half[2]];

        self.nodes.push(OcLeaf {
            root: Some(i_root),
            position: self.nodes.len(),
            lower: bounds.0,
            upper: mid,
            feature: OcFeature::Leaf,
            points: vec![],
        });
        self.nodes.push(OcLeaf {
            root: Some(i_root),
            position: self.nodes.len(),
            lower: [mid_0[0], mid_0[1] - half[1], mid_0[2]],
            upper: [mid[0] + half[0], mid[1], mid[2]],
            feature: OcFeature::Leaf,
            points: vec![],
        });
        self.nodes.push(OcLeaf {
            root: Some(i_root),
            position: self.nodes.len(),
            lower: [mid_0[0] - half[0], mid_0[1], mid_0[2]],
            upper: [mid[0], mid[1] + half[1], mid[2]],
            feature: OcFeature::Leaf,
            points: vec![],
        });
        self.nodes.push(OcLeaf {
            root: Some(i_root),
            position: self.nodes.len(),
            lower: mid_0,
            upper: [mid[0] + half[0], mid[1] + half[1], mid[2]],
            feature: OcFeature::Leaf,
            points: vec![],
        });

        self.nodes.push(OcLeaf {
            root: Some(i_root),
            position: self.nodes.len(),
            lower: [mid[0] - half[0], mid[1] - half[1], mid[2]],
            upper: mid_1,
            feature: OcFeature::Leaf,
            points: vec![],
        });
        self.nodes.push(OcLeaf {
            root: Some(i_root),
            position: self.nodes.len(),
            lower: [mid[0], mid[1] - half[1], mid[2]],
            upper: [mid_1[0] + half[0], mid_1[1], mid_1[2]],
            feature: OcFeature::Leaf,
            points: vec![],
        });
        self.nodes.push(OcLeaf {
            root: Some(i_root),
            position: self.nodes.len(),
            lower: [mid[0] - half[0], mid[1], mid[2]],
            upper: [mid_1[0], mid_1[1] + half[1], mid_1[2]],
            feature: OcFeature::Leaf,
            points: vec![],
        });
        self.nodes.push(OcLeaf {
            root: Some(i_root),
            position: self.nodes.len(),
            lower: mid,
            upper: [mid_1[0] + half[0], mid_1[1] + half[1], mid_1[2]],
            feature: OcFeature::Leaf,
            points: vec![],
        });

        out
    }
}

// Search
impl<'a, T: BasicFloat, P> OcTree<'a, T, P>
where
    P: Into<[T; 3]> + Index<usize, Output = T> + Clone + Copy + Serialize,
{
    pub fn search<R: TreeResult>(&self, point: P, by: SearchBy, result: &mut R) {
        // let data = if let Some(data) = self.data {
        //     data
        // } else {
        //     return;
        // };
        let data = if !self.data.is_empty() {
            &self.data
        } else {
            return;
        };
        let mut search_queue =
            BinaryHeap::with_capacity(std::cmp::max(10, (data.len() as f32).sqrt() as usize));

        self.search_(result, 0, &point, by, 0.0, &mut search_queue);
        while let Some(Reverse(node)) = search_queue.pop() {
            self.search_(result, node.raw, &point, by, node.order, &mut search_queue);
        }
    }

    fn search_<R: TreeResult>(
        &self,
        result: &mut R,
        node: usize,
        data: &P,
        by: SearchBy,
        min_dist: f32,
        queue: &mut BinaryHeap<Reverse<TreeHeapElement<usize, f32>>>,
    ) {
        if result.worst() < min_dist {
            return;
        }

        let node = &self.nodes[node];
        let clusters = match node.feature {
            OcFeature::Split(children) => {
                let mut ids = (0..8).collect::<Vec<usize>>();
                // If is inside, move target to front.
                if node.is_inside(*data) {
                    let id = node.locate_at(*data).unwrap();
                    ids.swap(0, id);
                }
                ids.into_iter().map(|i| children[i]).collect::<Vec<usize>>()
            }
            OcFeature::Leaf => {
                node.points.iter().for_each(|&i| {
                    if self.enable_ignore && self.ignores.contains(&i) {
                        return;
                    }
                    let p = self.data[i];
                    let distance = Self::distance_square(*data, p);
                    result.add(i, distance);
                });
                return;
            }
        };
        let first = clusters[0];
        clusters.into_iter().skip(1).for_each(|i| {
            let distance_type = self.nodes[i].distance(*data);
            let add_other = match by {
                SearchBy::Count(_) => {
                    if !result.is_full() {
                        true
                    } else {
                        match distance_type {
                            OcDistance::Outside(d) => (d * d).to_f32().unwrap() < result.worst(),
                            _ => true,
                        }
                    }
                }
                SearchBy::Radius(r) => match distance_type {
                    OcDistance::Outside(d) => d.to_f32().unwrap() < r,
                    _ => true,
                },
            };
            if add_other {
                let d = match distance_type {
                    OcDistance::Outside(d) => d * d,
                    OcDistance::Inside => T::zero(),
                };
                queue.push(Reverse(TreeHeapElement {
                    raw: i,
                    order: d.to_f32().unwrap(),
                }));
            }
        });

        self.search_(result, first, data, by, min_dist, queue);
    }

    fn distance_square(a: P, b: P) -> f32 {
        (0..3).fold(0f32, |acc, i| {
            acc + ((b[i] - a[i]).powi(2)).to_f32().unwrap()
        })
    }
}

impl<'a, T: BasicFloat, P> TreeSearch<P> for OcTree<'a, T, P>
where
    P: Into<[T; 3]> + Index<usize, Output = T> + Clone + Copy + Serialize,
{
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

#[test]
fn oc_crate() {
    let points = vec![
        [0., 0., 0.],
        [1., 0., 0.],
        [0., 1., 0.],
        [1., 1., 0.],
        [0., 0., 1.],
        [1., 0., 1.],
        [0., 1., 1.],
        [1., 1., 1.],
    ];

    let mut tree = OcTree::with_data(&points, 1, 3);
    tree.build();
    println!("{}", tree.nodes.len());
}
