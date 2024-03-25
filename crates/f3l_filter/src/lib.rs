mod condition_removal;
mod pass_through;
mod radius_outlier_removal;
mod statistical_outlier_removal;
mod voxel_grid;

/// A Trait for Filters
pub trait F3lFilter<'a, P> {
    /// if true, get outlier else inlier
    fn set_negative(&mut self, negative: bool);
    fn set_data(&mut self, data: &'a [P]);
    /// Return: Indices of inlier or outlier
    fn filter(&mut self) -> Vec<usize>;
    /// Return: data of inlier or outlier
    fn filter_instance(&mut self) -> Vec<P>;
    /// Call by `filter` or `filter_instance`.
    /// Use `filter` or `filter_instance` directly instead of call this.
    fn apply_filter(&mut self) -> bool;
}

pub use condition_removal::ConditionRemoval;
pub use pass_through::PassThrough;
pub use radius_outlier_removal::RadiusOutlierRemoval;
pub use statistical_outlier_removal::StatisticalOutlierRemoval;
pub use voxel_grid::VoxelGrid;
