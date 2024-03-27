mod oc_features;
mod oc_leaf;

use std::ops::Index;

use f3l_core::{get_minmax, BasicFloat};
pub use oc_features::*;
pub use oc_leaf::*;

pub struct OcTree<'a, T: BasicFloat, P>
where
    P: Into<[T; 3]> + Index<usize, Output = T> + Clone + Copy
{
    pub bounds: Option<(P, P)>,
    pub data: Option<&'a [P]>,
    pub max_points: usize,
    pub depth: usize,
    pub nodes: Vec<OcLeaf<T>>,
}

impl<'a, T: BasicFloat, P> OcTree<'a, T, P>
where
    P: Into<[T; 3]> + Index<usize, Output = T> + Clone + Copy + 'a,
    [T; 3]: Into<P>
{
    pub fn new(max_points: usize, depth: usize) -> Self {
        Self {
            bounds: None,
            data: None,
            max_points,
            depth,
            nodes: Vec::with_capacity(8_usize.pow(3)),
        }
    }

    pub fn with_data(data: &'a [P], max_points: usize, depth: usize) -> Self {
        Self {
            bounds: None,
            data: Some(data),
            max_points,
            depth,
            nodes: Vec::with_capacity(8_usize.pow(depth as u32))
        }
    }

    pub fn set_data(&mut self, data: &'a [P])
    {
        if !data.is_empty() {
            self.data = Some(data);
        }
    }

    fn compute_bounds(&mut self) {
        if let Some(data) = self.data {
            let bdx = get_minmax(data);
            self.bounds = Some(bdx);
        };
    }

    pub fn build(&mut self) {
        let data = if let Some(data) = self.data {
            data
        } else {
            return;
        };

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

        (0..data.len()).for_each(|i| {
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
                let id = self.nodes[i_node].locate_at(self.data.unwrap()[i_point]).unwrap();
                let nodes = if let Some(nodes) = nodes {
                    nodes
                } else {
                    let ids = self.create_8_nodes(i_node, (self.nodes[i_node].lower, self.nodes[i_node].upper));
                    let node = &mut self.nodes[i_node];
                    node.feature = OcFeature::Split(Some(ids));
                    ids
                };
                self.insert(i_point, depth+1, nodes[id]);

            },
            OcFeature::Leaf => {
                let full = self.nodes[i_node].points.len() >= self.max_points;
                if full {
                    // Set this node to `Split`, clone the vector of points and remove it.
                    let ids = self.create_8_nodes(i_node, (self.nodes[i_node].lower, self.nodes[i_node].upper));
                    let node = &mut self.nodes[i_node];
                    
                    // Max number of node is reached. Transfer this node from `Leaf` to `Split`
                    node.feature = OcFeature::Split(Some(ids));
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
            },
        };
    }

    fn create_8_nodes(&mut self, i_root: usize, bounds: ([T; 3], [T; 3])) -> [usize; 8] {
        let n = self.nodes.len();
        let out = [
            n, n+1, n+2, n+3, n+4, n+5, n+6, n+7
        ];
        
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
            root: Some(i_root), position: self.nodes.len(),
            lower: bounds.0,
            upper: mid,
            feature: OcFeature::Leaf, points: vec![],
        });
        self.nodes.push(OcLeaf {
            root: Some(i_root), position: self.nodes.len(),
            lower: [mid_0[0], mid_0[1] - half[1], mid_0[2]],
            upper: [mid[0] + half[0], mid[1], mid[2]],
            feature: OcFeature::Leaf, points: vec![],
        });
        self.nodes.push(OcLeaf {
            root: Some(i_root), position: self.nodes.len(),
            lower: [mid_0[0] - half[0], mid_0[1], mid_0[2]],
            upper: [mid[0], mid[1] + half[1], mid[2]],
            feature: OcFeature::Leaf, points: vec![],
        });
        self.nodes.push(OcLeaf {
            root: Some(i_root), position: self.nodes.len(),
            lower: mid_0,
            upper: [mid[0] + half[0], mid[1] + half[1], mid[2]],
            feature: OcFeature::Leaf, points: vec![],
        });

        self.nodes.push(OcLeaf {
            root: Some(i_root), position: self.nodes.len(),
            lower: [mid[0] - half[0], mid[1] - half[1], mid[2]],
            upper: mid_1,
            feature: OcFeature::Leaf, points: vec![],
        });
        self.nodes.push(OcLeaf {
            root: Some(i_root), position: self.nodes.len(),
            lower: [mid[0], mid[1] - half[1], mid[2]],
            upper: [mid_1[0] + half[0], mid_1[1], mid_1[2]],
            feature: OcFeature::Leaf, points: vec![],
        });
        self.nodes.push(OcLeaf {
            root: Some(i_root), position: self.nodes.len(),
            lower: [mid[0] - half[0], mid[1], mid[2]],
            upper: [mid_1[0], mid_1[1] + half[1], mid_1[2]],
            feature: OcFeature::Leaf, points: vec![],
        });
        self.nodes.push(OcLeaf {
            root: Some(i_root), position: self.nodes.len(),
            lower: mid,
            upper: [mid_1[0] + half[0], mid_1[1] + half[1], mid_1[2]],
            feature: OcFeature::Leaf, points: vec![],
        });

        out
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
    println!("{}",tree.nodes.len());
}