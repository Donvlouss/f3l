mod delaunay_2d;

pub use delaunay_2d::*;
use f3l_core::BasicFloat;

use crate::FaceIdType;

#[derive(Debug, Clone,Copy)]
pub struct SubTriangle<T: BasicFloat> {
    pub tri: FaceIdType,
    pub removed: bool,
    pub center: [T; 2],
    pub radius: T
}