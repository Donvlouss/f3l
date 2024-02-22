mod radius_outlier_removal;

pub trait F3lFilter<'a, P, T: f3l_search_tree::BasicFloat, const D: usize>
where
    P:Into<[T; D]> + Send + Sync + Clone,
    [T; D]: Into<P> + Send + Sync
{
    /// if true, get outlier else inlier
    fn set_negative(&mut self, negative: bool);
    fn set_data(&mut self, data: &'a Vec<P>);
    fn filter(&mut self) -> Vec<usize>;
    fn filter_instance(&mut self) -> Vec<P>;
    fn apply_filter(&mut self) -> bool;
}

pub use radius_outlier_removal::RadiusOutlierRemoval;