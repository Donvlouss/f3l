use f3l_core::BasicFloat;

pub struct TreeHeapElement<D, O: BasicFloat>
{
    pub raw: D,
    pub order: O
}

impl<D, O: BasicFloat> Eq for TreeHeapElement<D, O> {}
impl<D, O: BasicFloat> PartialEq for TreeHeapElement<D, O> {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}
impl<D, O: BasicFloat> PartialEq<O> for TreeHeapElement<D, O> {
    fn eq(&self, other: &O) -> bool {
        self.order == *other
    }
}
impl<D, O: BasicFloat> PartialOrd for TreeHeapElement<D, O> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.order.partial_cmp(&other.order)
    }
}
impl<D, O: BasicFloat> PartialOrd<O> for TreeHeapElement<D, O> {
    fn partial_cmp(&self, other: &O) -> Option<std::cmp::Ordering> {
        self.order.partial_cmp(other)
    }
}
impl<D, O: BasicFloat> Ord for TreeHeapElement<D, O> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.partial_cmp(&other.order).unwrap()
    }
}