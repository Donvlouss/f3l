use super::Eigenvectors3;
use f3l_core::{BasicFloat, glam::Vec3};
use f3l_search_tree::*;

pub struct NormalEstimation<'a, P, T: BasicFloat, const D: usize>
where
    P:Into<[T; D]> + Clone + Copy,
    [T; D]: Into<P>
{
    pub radius: T,
    pub threshold: usize,
    data: Option<&'a Vec<P>>,
    tree: KdTree<T, D>,
    normals: Vec<Vec3>
}

