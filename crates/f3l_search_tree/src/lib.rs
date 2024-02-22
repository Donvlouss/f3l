mod utils;
mod tree;
mod kdtree;

use num_traits::{Float, NumAssign, NumOps};
pub use utils::TreeHeapElement;
pub use tree::*;
pub use kdtree::*;


pub trait BasicFloat: Float + NumOps + NumAssign + Send + Sync + Copy + Clone {}
impl BasicFloat for f32 {}
impl BasicFloat for f64 {}
