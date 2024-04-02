mod convex_hull_2d;
mod convex_hull_3d;

pub use convex_hull_2d::*;
pub use convex_hull_3d::*;


pub trait Convex<'a, P> {
    fn new(data: &'a [P]) -> Self;

    fn compute(&mut self);
}

pub trait ConvexDetail<'a, P, T> {
    type Model;
    type Sample;

    fn generate_model(ids: &[P]) -> Self::Model;
    fn distance(model: Self::Model, p: &P) -> T;
    fn split_data(&self, model: &Self::Model, points: &[usize], signed: bool) -> Vec<usize>;
    fn compute_recursive(&self, ids: &[usize], samples: &Self::Sample, hulls: &mut Vec<usize>);
}