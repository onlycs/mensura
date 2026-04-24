use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{Dimension, Quantity};

/// Serializes as a plain `f64` (SI value).
impl<const D: Dimension> Serialize for Quantity<D> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

/// Deserializes from a plain `f64` (interpreted as SI value).
impl<'de, const D: Dimension> Deserialize<'de> for Quantity<D> {
    fn deserialize<Des: Deserializer<'de>>(deserializer: Des) -> Result<Self, Des::Error> {
        Ok(Self(f64::deserialize(deserializer)?))
    }
}
