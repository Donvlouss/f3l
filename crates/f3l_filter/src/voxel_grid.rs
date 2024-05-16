use crate::F3lFilterInverse;

use super::F3lFilter;
use f3l_core::serde::{self, Deserialize, Serialize};
use f3l_core::{get_minmax, BasicFloat};
use std::{collections::HashMap, fmt::Debug};

/// Build a `Dimension-wise` grid, compute mean of points per grid.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(crate = "self::serde")]
pub struct VoxelGridParameter<T: BasicFloat, const D: usize> {
    bound: Vec<(T, T)>,
    inverse_div: Vec<T>,
    nb_dim: Vec<usize>,
}

impl<T: BasicFloat, const D: usize> VoxelGridParameter<T, D> {
    pub fn new() -> Self {
        Self {
            bound: Vec::with_capacity(D),
            inverse_div: Vec::with_capacity(D),
            nb_dim: Vec::with_capacity(D),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct VoxelGrid<T: BasicFloat, const D: usize> {
    pub leaf: Vec<T>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub parameter: VoxelGridParameter<T, D>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    voxel_map: HashMap<usize, Vec<usize>>,
}

impl<T: BasicFloat, const D: usize> Default for VoxelGrid<T, D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: BasicFloat, const D: usize> VoxelGrid<T, D> {
    pub fn new() -> Self {
        Self {
            leaf: vec![T::zero(); D],
            parameter: VoxelGridParameter::new(),
            voxel_map: HashMap::new(),
        }
    }

    pub fn with_data<P: Into<[T; D]> + Copy>(leaf: &P) -> Self {
        let leaf: [T; D] = (*leaf).into();
        Self {
            leaf: leaf.iter().copied().collect(),
            parameter: VoxelGridParameter::<T, D>::new(),
            voxel_map: HashMap::new(),
        }
    }

    pub fn set_leaf<P: Into<[T; D]> + Copy>(&mut self, leaf: &P) {
        let leaf: [T; D] = (*leaf).into();
        self.leaf = leaf.iter().copied().collect();
    }

    pub fn set_leaf_raw(&mut self, leaf: &[T; D]) {
        (0..D).for_each(|i| self.leaf[i] = leaf[i]);
    }

    fn compute_bound<P: Into<[T; D]> + Copy>(&mut self, data: &[P])
    where
        [T; D]: Into<P>,
    {
        if data.is_empty() {
            return;
        }
        let (min, max) = get_minmax(data);
        let (min, max): ([T; D], [T; D]) = (min.into(), max.into());
        self.parameter.bound = min.into_iter().zip(max).collect();
    }

    pub fn leaf_check<P: Into<[T; D]> + Copy>(&mut self, data: &[P]) -> bool
    where
        [T; D]: Into<P>,
    {
        if self.parameter.bound.is_empty() {
            self.compute_bound(data);
        }
        let mut inverse = vec![T::zero(); D];
        let bounds = &self.parameter.bound;
        let mut box_range = [T::zero(); D];
        (0..D).for_each(|i| {
            let dif = bounds[i].1 - bounds[i].0;
            box_range[i] = dif / self.leaf[i];
            inverse[i] = T::one() / self.leaf[i];
        });

        let mut dims = vec![0usize; D];
        let mut count: usize = 0;
        // check number of voxels not overflow
        for (i, v) in box_range.iter().enumerate() {
            match v.to_usize() {
                Some(v) => {
                    // add one to avoid value too small
                    dims[i] = v + 1;
                    if let Some(vv) = count.checked_mul(v + 1) {
                        count = vv;
                    } else {
                        return false;
                    }
                }
                None => return false,
            }
        }
        self.parameter.inverse_div = inverse;
        self.parameter.nb_dim = dims;

        true
    }
}

impl<T: BasicFloat, const D: usize> F3lFilterInverse for VoxelGrid<T, D> {
    fn set_negative(&mut self, _negative: bool) {}
}

impl<'a, P, T: BasicFloat, const D: usize> F3lFilter<'a, P, D> for VoxelGrid<T, D>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync + Debug,
    [T; D]: Into<P>,
{
    /// Get not empty `ids` of `Voxel Grid` not `point id`
    fn filter(&mut self, data: &'a Vec<P>) -> Vec<usize> {
        if !self.apply_filter(data) {
            return vec![];
        }

        let keys = self.voxel_map.keys();
        keys.into_iter().copied().collect()
    }

    /// Get `mean point` of not empty grids
    fn filter_instance(&mut self, data: &'a Vec<P>) -> Vec<P> {
        if !self.apply_filter(data) {
            return vec![];
        }

        let maps = &self.voxel_map;
        maps.iter()
            .map(|(_, pts)| {
                let nb = T::from(pts.len()).unwrap();
                let factor = T::one() / nb;
                let mut sum = [T::zero(); D];
                pts.iter().for_each(|p| {
                    let p: [T; D] = data[*p].into();
                    (0..D).for_each(|i| {
                        sum[i] += p[i] * factor;
                    });
                });
                sum.into()
            })
            .collect()
    }

    fn apply_filter(&mut self, data: &'a Vec<P>) -> bool {
        if !self.leaf_check(data) {
            return false;
        }

        let VoxelGridParameter {
            bound,
            inverse_div: inv_div,
            nb_dim,
        } = &self.parameter;

        let min = bound.iter().map(|(a, _)| *a).collect::<Vec<_>>();

        let mut inc = [1usize; D];
        (0..D).for_each(|i| {
            if i == 0 {
                return;
            }

            inc[i] = nb_dim[i - 1] * inc[i - 1];
        });

        data.iter().enumerate().for_each(|(i, p)| {
            let p: [T; D] = (*p).into();
            let mut dim = 0usize;
            (0..D).for_each(|i| {
                let v = (p[i] - min[i]) * inv_div[i];
                let d = match v.to_usize() {
                    Some(d) => d,
                    None => return,
                };
                dim += d * inc[i];
            });
            let vec = self.voxel_map.entry(dim).or_default();
            vec.push(i);
        });
        true
    }
}

#[test]
fn serde() {
    let model = VoxelGrid::with_data(&[0.05f32; 3]);
    let content = serde_json::to_string(&model).unwrap();
    println!("{}", content);
    let text = r#"{"leaf":[0.05,0.05,0.05]}"#;
    let model_de: VoxelGrid<f32, 3> = serde_json::from_str(text).unwrap();
    assert_eq!(model.leaf, model_de.leaf);
}
