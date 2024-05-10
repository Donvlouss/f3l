mod delaunay_2d;

pub use delaunay_2d::*;
use f3l_core::{BasicFloat, Line, serde::{self, Deserialize, Serialize}};

use crate::FaceIdType;

/// Structure represent a triangle to be computed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(crate="self::serde")]
pub struct SubTriangle<T: BasicFloat> {
    pub tri: FaceIdType,
    pub removed: bool,
    pub center: [T; 2],
    pub radius: T,
}

/// Structure represent a shape contains triangle meshes and multiple contours(with holes).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate="self::serde")]
pub struct Delaunay2DShape {
    pub mesh: Vec<FaceIdType>,
    pub contours: Vec<Vec<Line>>,
}
