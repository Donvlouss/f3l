use f3l_core::BasicFloat;

use crate::OcFeature;

pub struct OcLeaf<T: BasicFloat> {
    /// Root Leaf id
    pub root: Option<usize>,
    /// Current Position (Depth, Node: 0-7)
    pub position: usize,
    /// Lower bound
    pub lower: [T; 3],
    /// Upper bound
    pub upper: [T; 3],
    /// `Split` or `Leaf`
    pub feature: OcFeature,
    /// points indices
    pub points: Vec<usize>
}

impl<T: BasicFloat> OcLeaf<T> {
    /// Return point is inside this node or not.
    pub fn is_inside<P>(&self, p: P) -> bool
    where
        P: Into<[T; 3]> + std::ops::Index<usize, Output = T> + Clone + Copy
    {
        p[0] >= self.lower[0]
        && p[1] >= self.lower[1]
        && p[2] >= self.lower[2]
        && p[0] <= self.lower[0]
        && p[1] <= self.lower[1]
        && p[2] <= self.lower[2]
    }

    /// Return `None` if this node is leaf, else `Some(index)` of child
    pub fn locate_at<P>(&self, p: P) -> Option<usize>
    where
        P: Into<[T; 3]> + std::ops::Index<usize, Output = T> + Clone + Copy
    {
        if let OcFeature::Leaf = self.feature {
            return None;
        };
        let mid = [
            (self.upper[0] + self.lower[0]) / T::from(2).unwrap(),
            (self.upper[1] + self.lower[1]) / T::from(2).unwrap(),
            (self.upper[2] + self.lower[2]) / T::from(2).unwrap(),
        ];
        let id = (0..3)
            .fold(0_usize, |acc, i| {
                acc + if p[i] < mid[i] { 0 } else { 1 } * 2_usize.pow(i as u32)
            });
        Some(id)
    }
}

#[test]
fn oc_index_check() {
    let mid = [0.5, 0.5, 0.5];
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
    (0..8).for_each(|i| {
        let p = points[i];
        let id = (0..3)
            .fold(0_usize, |acc, i| {
                acc + if p[i] < mid[i] { 0 } else { 1 } * 2_usize.pow(i as u32)
            });
        assert_eq!(i, id);
    });
}