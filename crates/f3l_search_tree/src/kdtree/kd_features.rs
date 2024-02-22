#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
pub enum KdFeature
{
    Split((usize, f32)),
    Leaf(usize),
}

impl Default for KdFeature {
    fn default() -> Self {
        Self::Split((0, 0.0))
    }
}