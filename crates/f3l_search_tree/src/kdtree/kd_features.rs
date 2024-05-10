#[cfg(all(feature="core", not(feature="pure")))]
use f3l_core::serde::{self, Deserialize, Serialize};
#[cfg(all(feature="pure", not(feature="core")))]
use crate::serde::{self, Deserialize, Serialize};

/// KD Tree node type, all data store in `leaf` node
#[derive(Clone, Debug, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(crate = "self::serde")] 
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
