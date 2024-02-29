mod radius_outlier_removal;
mod voxel_grid;
mod pass_through;
mod condition_removal;
mod statistical_outlier_removal;

pub trait F3lFilter<'a, P>
{
    /// if true, get outlier else inlier
    fn set_negative(&mut self, negative: bool);
    fn set_data(&mut self, data: &'a Vec<P>);
    fn filter(&mut self) -> Vec<usize>;
    fn filter_instance(&mut self) -> Vec<P>;
    fn apply_filter(&mut self) -> bool;
}

pub use radius_outlier_removal::RadiusOutlierRemoval;
pub use voxel_grid::VoxelGrid;
pub use pass_through::PassThrough;
pub use condition_removal::ConditionRemoval;
pub use statistical_outlier_removal::StatisticalOutlierRemoval;