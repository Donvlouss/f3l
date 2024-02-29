use std::ops::{
    Bound,
    Range
};

use f3l_core::BasicFloat;
use super::F3lFilter;

pub struct PassThrough<'a, P, T: BasicFloat, const D: usize>
where
    P:Into<[T; D]> + Clone + Copy,
{
    pub negative: bool,
    pub dim: usize,
    pub bound: Option<Range<Bound<T>>>,
    data: Option<&'a Vec<P>>,
    inliers: Vec<usize>,
}

impl<'a, P, T: BasicFloat, const D: usize> Default for PassThrough<'a, P, T, D>
where
    P:Into<[T; D]> + Clone + Copy
{
    fn default() -> Self {
        Self {
            negative: false,
            dim: Default::default(),
            bound: Default::default(),
            data: Default::default(),
            inliers: Default::default()
        }
    }
}

impl<'a, P, T: BasicFloat, const D: usize> PassThrough<'a, P, T, D>
where
    P:Into<[T; D]> + Clone + Copy,
{
    pub fn with_data(data: &'a Vec<P>, bound: Range<Bound<T>>, dim: usize) -> Self {
        Self {
            negative: false,
            dim,
            bound: Some(bound),
            data: Some(data),
            inliers: Default::default()
        }
    }

    pub fn set_parameter(&mut self, dim: usize, bound: Range<Bound<T>>) {
        self.dim = dim;
        self.bound = Some(bound);
    }
}

impl<'a, P, T: BasicFloat, const D: usize> F3lFilter<'a, P> for PassThrough<'a, P, T, D>
where
    P:Into<[T; D]> + Clone + Copy + Send + Sync,
    [T; D]: Into<P>
{
    fn set_negative(&mut self, negative: bool) {
        self.negative = negative;
    }

    fn set_data(&mut self, data: &'a Vec<P>) {
        self.data = Some(data);
    }

    fn filter(&mut self) -> Vec<usize> {
        if !self.apply_filter() {
            return vec![];
        }
        self.inliers.clone()
    }

    fn filter_instance(&mut self) -> Vec<P> {
        if !self.apply_filter() {
            return vec![];
        }
        let data = self.data.unwrap();
        self.inliers.iter()
            .map(|i| data[*i])
            .collect()
    }

    fn apply_filter(&mut self) -> bool {
        if self.dim >= D {
            return false;
        }

        let (start, end) = if let Some(bound) = &self.bound {
            (bound.start, bound.end)
        } else {
            return false;
        };

        let data = if let Some(data) = self.data {
            data
        } else {
            return false;
        };
        
        use rayon::prelude::*;
        self.inliers = data
            .par_iter()
            .enumerate()
            .filter_map(|(i, &p)| {
                let p: [T; D] = p.into();
                let p = p[self.dim];
                let b_start =  match start {
                    Bound::Included(v) => p >= v,
                    Bound::Excluded(v) => p > v,
                    Bound::Unbounded => true,
                };
                let b_end = match end {
                    Bound::Included(v) => p <= v,
                    Bound::Excluded(v) => p < v,
                    Bound::Unbounded => true,
                };
                if !self.negative && (b_start && b_end) {
                    Some(i)
                } else if self.negative && !(b_start && b_end) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        true
    }
}