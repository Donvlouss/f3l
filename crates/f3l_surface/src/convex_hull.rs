mod convex_hull_2d;
mod convex_hull_3d;
mod convex_hull_3d_2d;

use std::marker::PhantomData;

pub use convex_hull_2d::*;
pub use convex_hull_3d::*;
pub use convex_hull_3d_2d::*;

use crate::{FaceIdType, FaceInstanceType};
use f3l_core::serde::{self, Deserialize, Serialize};

/// Generic Convex Hull wrapper of [`ConvexHull2D`] and [`ConvexHull3D`] points data.
///
/// [`ConvexHull3D2D`] If 3d cloud is near a plane, would align 3d to 2d, then compute 2d.
///
/// # Panic
/// * 2d: data.len() < 3
/// * 3d: data.len() < 4
/// ## Example
/// * 2d
/// ```rust
/// let img = image::open("../../data/hull.png").unwrap();
/// let dimension = img.dimensions();
/// let points = img.pixels().into_iter().filter_map(|(x, y, rgb)| {
///     let a = rgb.0;
///     if a.iter().all(|&c| c != 0) {
///         Some([
///             x as f32 / dimension.0 as f32,
///             y as f32 / dimension.1 as f32,
///         ])
///     } else {
///         None
///     }
/// }).collect::<Vec<_>>();
///
/// let mut cvh = ConvexHull::with_data(&points);
/// cvh.compute();
///
/// let hulls = if let ConvexHullId::D2(hulls) = cvh.hulls() {
///     hulls
/// } else {
///     panic!("Could not resolve to D2 type.")};
/// };
/// ```
/// * 3d
/// ```rust
/// let vertices = load_ply("../../data/table_voxel_down.ply");
/// let mut cvh = ConvexHull::with_data(&points);
/// cvh.compute();
/// let hulls = if let ConvexHullId::D2(hulls) = cvh.hulls() {
/// hulls
/// } else {
///     panic!("Could not resolve to D2 type.")
/// };
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct ConvexHull<'a, T: f3l_core::BasicFloat, P, const D: usize, CVH>
where
    P: Into<[T; D]> + Clone + Copy + Send + Sync + std::ops::Index<usize, Output = T>,
    CVH: Convex<'a, P>,
{
    dim: usize,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    cvh: CVH,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    _marker: PhantomData<(T, &'a P)>,
}

/// Represent Convex Hull result of Ids.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum ConvexHullId {
    D2(Vec<usize>),
    D3(Vec<FaceIdType>),
}

impl Default for ConvexHullId {
    fn default() -> Self {
        Self::D2(vec![])
    }
}

/// Represent Convex Hull result of data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum ConvexHullInstance<P: Copy> {
    D2(Vec<P>),
    D3(Vec<FaceInstanceType<P>>),
}

impl<P: Copy> Default for ConvexHullInstance<P> {
    fn default() -> Self {
        Self::D2(vec![])
    }
}

impl<'a, T: f3l_core::BasicFloat, P> Convex<'a, P>
    for ConvexHull<'a, T, P, 2, ConvexHull2D<'a, T, P>>
where
    P: Into<[T; 2]> + Clone + Copy + Send + Sync + std::ops::Index<usize, Output = T>,
{
    fn new() -> Self {
        Self {
            dim: 2,
            cvh: ConvexHull2D::new(),
            _marker: PhantomData,
        }
    }
    fn with_data(data: &'a Vec<P>) -> Self {
        Self {
            dim: 2,
            cvh: ConvexHull2D::with_data(data),
            _marker: PhantomData,
        }
    }

    fn set_data(&mut self, data: &'a Vec<P>) {
        self.cvh.set_data(data);
    }

    fn compute(&mut self) {
        self.cvh.compute()
    }
}

impl<'a, T: f3l_core::BasicFloat, P> ConvexHull<'a, T, P, 2, ConvexHull2D<'a, T, P>>
where
    P: Into<[T; 2]> + Clone + Copy + Send + Sync + std::ops::Index<usize, Output = T>,
{
    pub fn hulls(&self) -> ConvexHullId {
        ConvexHullId::D2(self.cvh.hulls.clone())
    }

    pub fn hulls_instance(&self) -> Vec<P> {
        self.cvh.hulls.iter().map(|&i| self.cvh.data[i]).collect()
    }
}

impl<'a, T: f3l_core::BasicFloat, P> Convex<'a, P>
    for ConvexHull<'a, T, P, 3, ConvexHull3D<'a, T, P>>
where
    P: Into<[T; 3]> + Clone + Copy + Send + Sync + std::ops::Index<usize, Output = T>,
{
    /// Return `ConvexHull<ConvexHull3D>` wrapper
    fn new() -> Self {
        Self {
            dim: 3,
            cvh: ConvexHull3D::new(),
            _marker: PhantomData,
        }
    }
    fn with_data(data: &'a Vec<P>) -> Self {
        Self {
            dim: 3,
            cvh: ConvexHull3D::with_data(data),
            _marker: PhantomData,
        }
    }

    fn set_data(&mut self, data: &'a Vec<P>) {
        self.cvh.set_data(data);
    }

    fn compute(&mut self) {
        self.cvh.compute()
    }
}

impl<'a, T: f3l_core::BasicFloat, P> ConvexHull<'a, T, P, 3, ConvexHull3D<'a, T, P>>
where
    P: Into<[T; 3]> + Clone + Copy + Send + Sync + std::ops::Index<usize, Output = T>,
{
    /// Return
    /// * ConvexHullInstance::D2: If data is near a plane.
    /// * ConvexHullInstance::D3: data is normal point cloud.
    pub fn hulls(&self) -> ConvexHullId {
        self.cvh.hulls.clone()
    }

    /// Return
    /// * ConvexHullInstance::D2: If data is near a plane.
    /// * ConvexHullInstance::D3: data is normal point cloud.
    pub fn hulls_instance(&self) -> ConvexHullInstance<P> {
        match self.cvh.hulls.clone() {
            ConvexHullId::D2(edges) => {
                ConvexHullInstance::D2(edges.iter().map(|&i| self.cvh.data[i]).collect())
            }
            ConvexHullId::D3(faces) => ConvexHullInstance::D3(
                faces
                    .iter()
                    .map(|&tri| FaceInstanceType {
                        point: [
                            self.cvh.data[tri.point[0]],
                            self.cvh.data[tri.point[1]],
                            self.cvh.data[tri.point[2]],
                        ],
                    })
                    .collect(),
            ),
        }
    }
}

/// Convex Hull Trait.
pub trait Convex<'a, P> {
    // fn new(data: &'a Vec<P>) -> Self;
    fn new() -> Self;
    fn with_data(data: &'a Vec<P>) -> Self;
    fn set_data(&mut self, data: &'a Vec<P>);
    fn compute(&mut self);
    // fn compute(&mut self, data: &Vec<P>);
}
