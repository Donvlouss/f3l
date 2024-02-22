use num_traits::Float;

pub struct TreeHeapElement<D, O: Float>
{
    pub raw: D,
    pub order: O
}

impl<D, O: Float> Eq for TreeHeapElement<D, O> {}
impl<D, O: Float> PartialEq for TreeHeapElement<D, O> {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}
impl<D, O: Float> PartialEq<O> for TreeHeapElement<D, O> {
    fn eq(&self, other: &O) -> bool {
        self.order == *other
    }
}
impl<D, O: Float> PartialOrd for TreeHeapElement<D, O> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.order.partial_cmp(&other.order)
    }
}
impl<D, O: Float> PartialOrd<O> for TreeHeapElement<D, O> {
    fn partial_cmp(&self, other: &O) -> Option<std::cmp::Ordering> {
        self.order.partial_cmp(other)
    }
}
impl<D, O: Float> Ord for TreeHeapElement<D, O> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.partial_cmp(&other.order).unwrap()
    }
}