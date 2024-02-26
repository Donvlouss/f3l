use f3l_core::{
    BasicFloat,
    get_minmax
};
use super::F3lFilter;
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Default)]
pub struct VoxelGridParameter<T: BasicFloat, const D: usize> {
    bound: Option<([T; D], [T; D])>,
    inverse_div: Option<[T; D]>,
    nb_dim: Option<[usize; D]>
}

impl<T: BasicFloat, const D: usize> VoxelGridParameter<T, D> {
    pub fn new() -> Self {
        Self {
            bound: None,
            inverse_div: None,
            nb_dim: None
        }
    }
}

pub struct VoxelGrid<'a, P, T: BasicFloat, const D: usize>
where
    P:Into<[T; D]> + Clone + Copy,
    [T; D]: Into<P>
{
    pub leaf: [T; D],
    pub parameter: VoxelGridParameter<T, D>,
    data: Option<&'a Vec<P>>,
    voxel_map: HashMap<usize, Vec<usize>>
}

impl<'a, P, T:BasicFloat, const D: usize> VoxelGrid<'a, P, T, D>
where
    P:Into<[T; D]> + Clone + Copy,
    [T; D]: Into<P>
{
    pub fn new() -> Self {
        Self {
            leaf: [T::zero(); D],
            data: None,
            parameter: VoxelGridParameter::new(),
            voxel_map: HashMap::new()
        }
    }

    pub fn with_data(data: &'a Vec<P>, leaf: &P) -> Self {
        Self {
            leaf: (*leaf).into(),
            data: Some(data),
            parameter: VoxelGridParameter::<T, D>::new(),
            voxel_map: HashMap::new()
        }
    }

    pub fn set_leaf(&mut self, leaf: &P) {
        self.leaf = (*leaf).into();
    }

    pub fn set_leaf_raw(&mut self, leaf: &[T; D]) {
        (0..D).for_each(|i| self.leaf[i] = leaf[i]);
    }

    fn compute_bound(&mut self) {
        if self.data.is_none() {
            return;
        }
        let data = self.data.unwrap();
        if data.is_empty() {
            return;
        }
        let (min, max) = get_minmax(data);
        self.parameter.bound = Some((min.into(), max.into()));
    }

    pub fn leaf_check(&mut self) -> bool {
        if self.parameter.bound.is_none() {
            self.compute_bound();
        }
        let mut inverse = [T::zero(); D];
        let (min, max) = self.parameter.bound.unwrap();
        let mut box_range = [T::zero(); D];
        (0..D)
            .for_each(|i| {
                let dif = max[i] - min[i];
                box_range[i] = dif / self.leaf[i];
                inverse[i] = T::one() / self.leaf[i];
            });
        
        let mut dims = [0usize; D];
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
                },
                None => return false,
            }
        }
        self.parameter.inverse_div = Some(inverse);
        self.parameter.nb_dim = Some(dims);

        true
    }
}

impl<'a, P, T:BasicFloat, const D: usize> F3lFilter<'a, P> for VoxelGrid<'a, P, T, D>
where
    P:Into<[T; D]> + Clone + Copy + Send + Sync + Debug,
    [T; D]: Into<P>
{
    fn set_negative(&mut self, _negative: bool) {
    }

    fn set_data(&mut self, data: &'a Vec<P>) {
        self.data = Some(data)
    }

    fn filter(&mut self) -> Vec<usize> {
        if !self.apply_filter() {
            return vec![];
        }

        let keys = self.voxel_map.keys();
        keys.into_iter()
            .map(|k| *k)
            .collect()
    }

    fn filter_instance(&mut self) -> Vec<P> {
        if !self.apply_filter() {
            return vec![];
        }

        let maps = &self.voxel_map;
        let data = self.data.unwrap();
        maps.into_iter()
            .map(|(_, pts)| {
                let nb = T::from(pts.len()).unwrap();
                let factor = T::one() / nb;
                let mut sum = [T::zero(); D];
                pts.into_iter()
                    .for_each(|p| {
                        let p: [T; D] = data[*p].into();
                        (0..D)
                            .for_each(|i| {
                                sum[i] += p[i] * factor;
                            });
                    });
                sum.into()
            }).collect()
    }

    fn apply_filter(&mut self) -> bool {
        if !self.leaf_check() {
            return false;
        }

        let VoxelGridParameter{bound: Some((min, _)), inverse_div: Some(inv_div), nb_dim: Some(nb_dim)}
            = self.parameter
        else {
            return false;
        };

        let mut inc = [1usize; D];
        (0..D)
            .for_each(|i| {
                if i == 0 { return; }
                
                inc[i] = nb_dim[i-1] * inc[i-1];
            });
        
        let points = self.data.unwrap();
        points.into_iter()
            .enumerate()
            .for_each(|(i, p)| {
                let p: [T; D] = (*p).into();
                let mut dim = 0usize;
                (0..D)
                    .for_each(|i| {
                        let v = (p[i] - min[i]) * inv_div[i];
                        let d = match v.to_usize() {
                            Some(d) => d,
                            None => return,
                        };
                        dim += d * inc[i];
                    });
                let vec = self.voxel_map.entry(dim).or_insert(vec![]);
                vec.push(i);
            });
        true
    }
}
