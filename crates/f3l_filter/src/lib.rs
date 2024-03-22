mod condition_removal;
mod pass_through;
mod radius_outlier_removal;
mod statistical_outlier_removal;
mod voxel_grid;

pub trait F3lFilter<'a, P> {
    /// if true, get outlier else inlier
    fn set_negative(&mut self, negative: bool);
    fn set_data(&mut self, data: &'a Vec<P>);
    fn filter(&mut self) -> Vec<usize>;
    fn filter_instance(&mut self) -> Vec<P>;
    fn apply_filter(&mut self) -> bool;
}

pub use condition_removal::ConditionRemoval;
pub use pass_through::PassThrough;
pub use radius_outlier_removal::RadiusOutlierRemoval;
pub use statistical_outlier_removal::StatisticalOutlierRemoval;
pub use voxel_grid::VoxelGrid;
