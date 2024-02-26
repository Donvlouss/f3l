use num_traits::{
    Float,
    NumAssign,
    NumOps
};

pub trait BasicFloat: Float + NumOps + NumAssign + Send + Sync + Copy + Clone {}
impl BasicFloat for f32 {}
impl BasicFloat for f64 {}

