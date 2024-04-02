use std::ops::Index;

use f3l_core::BasicFloat;

use crate::{Convex, FaceIdType};


#[derive(Debug, Clone)]
pub struct ConvexHull3D<'a, T: BasicFloat, P>
where
    P: Into<[T; 3]> + Clone + Copy + Send + Sync + Index<usize, Output = T>
{
    pub data: &'a [P],
    pub hulls: Vec<FaceIdType>,
}

impl<'a, T: BasicFloat, P> Convex<'a, P> for ConvexHull3D<'a, T, P>
where
    P: Into<[T; 3]> + Clone + Copy + Send + Sync + Index<usize, Output = T>
{
    fn new(data: &'a [P]) -> Self {
        Self {
            data, hulls: vec![]
        }
    }

    fn compute(&mut self) {
        todo!()
    }
}