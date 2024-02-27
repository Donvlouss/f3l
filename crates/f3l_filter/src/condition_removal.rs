use std::ops::{
    Bound,
    Range,
};

use f3l_core::BasicFloat;
use super::F3lFilter;

pub struct ConditionRemoval<'a, P, T: BasicFloat, const D: usize>
where
    P:Into<[T; D]> + Clone + Copy,
{
    pub negative: bool,
    pub bound: Option<&'a Vec<(usize, Range<Bound<T>>)>>,
    data: Option<&'a Vec<P>>,
    inliers: Vec<usize>,
}

impl<'a, P, T: BasicFloat, const D: usize> Default for ConditionRemoval<'a, P, T, D>
where
    P:Into<[T; D]> + Clone + Copy
{
    fn default() -> Self {
        Self {
            negative: false,
            bound: Default::default(),
            data: Default::default(),
            inliers: Default::default()
        }
    }
}

impl<'a, P, T: BasicFloat, const D: usize> ConditionRemoval<'a, P, T, D>
where
    P:Into<[T; D]> + Clone + Copy,
{
    // pub fn with_data(data: &'a Vec<P>, bound: &'a Vec<(usize, Bound<T>, Bound<T>)>) -> Self {
    pub fn with_data(data: &'a Vec<P>, bound: &'a Vec<(usize, Range<Bound<T>>)>) -> Self {
        Self {
            negative: false,
            bound: Some(bound),
            data: Some(data),
            inliers: Default::default()
        }
    }

    pub fn set_parameter(&mut self, bound: &'a Vec<(usize, Range<Bound<T>>)>) {
        self.bound = Some(bound);
    }
}


impl<'a, P, T: BasicFloat, const D: usize> F3lFilter<'a, P> for ConditionRemoval<'a, P, T, D>
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
        let bounds = if let Some(bound) = self.bound {
            bound
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
                let mut ok = true;
                bounds.iter()
                    .for_each(|(dim, bound)| {
                        let p = p[*dim];
                        let b_start =  match bound.start {
                            Bound::Included(v) => p >= v,
                            Bound::Excluded(v) => p >  v,
                            Bound::Unbounded => true,
                        };
                        let b_end = match bound.end {
                            Bound::Included(v) => p <= v,
                            Bound::Excluded(v) => p <  v,
                            Bound::Unbounded => true,
                        };
                        if !self.negative && (b_start && b_end) {
                            ok &= true;
                        } else if self.negative && !(b_start && b_end) {
                            ok &= true;
                        } else {
                            ok = false;
                            return;
                        }
                    });
                if ok {
                    Some(i)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        true
    }
}