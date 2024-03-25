
/// KD Tree node type, all data store in `leaf` node
#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
pub enum KdFeature {
    /// Tree node
    Split((usize, f32)),
    /// Leaf node
    Leaf(usize),
}

impl Default for KdFeature {
    fn default() -> Self {
        Self::Split((0, 0.0))
    }
}
