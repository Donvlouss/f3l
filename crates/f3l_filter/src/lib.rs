mod condition_removal;
mod pass_through;
mod radius_outlier_removal;
mod statistical_outlier_removal;
mod voxel_grid;

pub trait F3lFilterInverse {
    /// if true, get outlier else inlier
    fn set_negative(&mut self, negative: bool);
}

/// A Trait for Filters
pub trait F3lFilter<'a, P, const D: usize>: F3lFilterInverse {
    /// Return: Indices of inlier or outlier
    fn filter(&mut self, data: &'a Vec<P>) -> Vec<usize>;
    /// Return: data of inlier or outlier
    fn filter_instance(&mut self, data: &'a Vec<P>) -> Vec<P>;
    /// Call by `filter` or `filter_instance`.
    /// Use `filter` or `filter_instance` directly instead of call this.
    fn apply_filter(&mut self, data: &'a Vec<P>) -> bool;
}

pub use condition_removal::ConditionRemoval;
pub use pass_through::PassThrough;
pub use radius_outlier_removal::RadiusOutlierRemoval;
pub use statistical_outlier_removal::StatisticalOutlierRemoval;
pub use voxel_grid::VoxelGrid;
