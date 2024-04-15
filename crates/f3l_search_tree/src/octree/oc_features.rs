pub enum OcFeature {
    /// 8 children
    Split([usize; 8]),
    /// point ids
    Leaf,
}

pub enum OcDistance<T> {
    Outside(T),
    Inside,
}
