pub enum OcFeature {
    /// 8 children
    Split(Option<[usize; 8]>),
    /// point ids
    Leaf
}