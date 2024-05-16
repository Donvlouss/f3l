#[cfg(all(feature = "pure", not(feature = "core")))]
use crate::serde::{self, Deserialize, Serialize};
#[cfg(all(feature = "core", not(feature = "pure")))]
use f3l_core::serde::{self, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum OcFeature {
    /// 8 children
    Split([usize; 8]),
    /// point ids
    Leaf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum OcDistance<T> {
    Outside(T),
    Inside,
}
