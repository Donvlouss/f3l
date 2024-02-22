use super::KdFeature;

#[derive(Clone, Debug, Default)]
pub struct KdLeaf
{
    pub left: Option<Box<KdLeaf>>,
    pub right: Option<Box<KdLeaf>>,
    pub feature: KdFeature,
}