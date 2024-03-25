use super::KdFeature;

/// All KD nodes
#[derive(Clone, Debug, Default)]
pub struct KdLeaf {
    pub left: Option<Box<KdLeaf>>,
    pub right: Option<Box<KdLeaf>>,
    pub feature: KdFeature,
}
