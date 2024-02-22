use f3l_search_tree::{
    BasicFloat,
    KdTree,
    SearchBy,
    TreeRadiusResult,
    TreeResult
};
use rayon::prelude::*;

use crate::F3lFilter;

#[derive(Debug)]
pub struct RadiusOutlierRemoval<'a, P, T: BasicFloat, const D: usize>
where
    P:Into<[T; D]> + Send + Sync + Clone,
    [T; D]: Into<P> + Send + Sync
{
    pub negative: bool,
    pub radius: T,
    pub threshold: usize,
    data: Option<&'a Vec<P>>,
    tree: KdTree<T, D>,
    inlier: Vec<bool>
}

impl<'a, P, T:BasicFloat, const D: usize> RadiusOutlierRemoval<'a, P, T, D>
where
    P:Into<[T; D]> + Send + Sync + Clone,
    [T; D]: Into<P> + Send + Sync
{
    pub fn new(radius: T, threshold: usize) -> Self {
        Self {
            negative: false,
            radius, threshold,
            data: None,
            tree: KdTree::<T, D>::new(),
            inlier: vec![]
        }
    }

    pub fn with_data(radius: T, threshold: usize, data: &'a Vec<P>) -> Self {
        let mut entity = Self::new(radius, threshold);
        entity.set_data(data);
        entity
    }

    #[inline]
    fn ok(&self, is_inlier: bool) -> bool {
        (!is_inlier && self.negative) || (is_inlier && !self.negative)
    }

}

impl<'a, P, T:BasicFloat, const D: usize> F3lFilter<'a, P, T, D> for RadiusOutlierRemoval<'a, P, T, D>
where
    P:Into<[T; D]> + Send + Sync + Clone,
    [T; D]: Into<P> + Send + Sync
{
    fn set_negative(&mut self, negative: bool) {
        self.negative = negative;
    }

    fn set_data(&mut self, data: &'a Vec<P>) {
        self.data = Some(data);
        self.tree.set_data(data);
    }

    fn filter(&mut self) -> Vec<usize> {
        self.apply_filter();
        
        self.inlier.iter()
            .enumerate()
            .filter(|&(_, f)| self.ok(*f))
            .map(|(i, _)| i)
            .collect()
    }

    fn filter_instance(&mut self) -> Vec<P> {
        self.apply_filter();
        
        let data = self.data.unwrap();
        self.inlier.iter()
            .enumerate()
            .filter(|&(_, f)| self.ok(*f))
            .map(|(i, _)| data[i].clone())
            .collect()
    }

    fn apply_filter(&mut self) -> bool {
        if self.tree.data.len() == 0 {
            return false;
        }
        self.tree.build();
        let capacity = self.tree.data.len() / 10;
        let capacity = if capacity > 10 {capacity} else { 10 };
        
        let r = (self.radius.to_f32().unwrap()).powi(2);
        let by = if self.radius==T::zero() {SearchBy::Radius(1.0)} else {SearchBy::Radius(r)};
        let data = self.data.unwrap();

        let th = self.threshold;

        self.inlier.resize(data.len(), false);
        let inlier = data
            .par_iter()
            .enumerate()
            .map(|(i, p)| {
                let mut result = TreeRadiusResult::with_capacity(r, capacity);
                self.tree.search((*p).clone(), by, &mut result);
                (i, result.data.len() >= th)
            })
            .collect::<Vec<_>>();
        inlier.iter()
            .filter(|(_,f)| *f)
            .for_each(|(i,_)| self.inlier[*i] = true);

        true
    }
}