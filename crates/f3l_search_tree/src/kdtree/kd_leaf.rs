#[cfg(all(feature = "pure", not(feature = "core")))]
use crate::serde::{self, Deserialize, Serialize};
#[cfg(all(feature = "core", not(feature = "pure")))]
use f3l_core::serde::{self, Deserialize, Serialize};

use super::KdFeature;

/// All KD nodes
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct KdLeaf {
    pub left: Option<Box<KdLeaf>>,
    pub right: Option<Box<KdLeaf>>,
    pub feature: KdFeature,
}
