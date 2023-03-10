//! General Discord constructs

use std::fmt::Display;

use chrono::NaiveDateTime;
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

/// Time stamp in Seconds since 1/1/1970
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnixTimestamp(u64);

impl Default for UnixTimestamp {
    fn default() -> Self {
        Self(
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        )
    }
}

impl UnixTimestamp {
    /// Convert to chrono::DateTime
    pub fn as_chrono(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp_opt(self.0 as i64, 0).unwrap()
    }
}

/// Discord Numeric Identifier
///
/// (Due to JS limitations, Discord transmits 64 bit integers as strings)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Snowflake(pub(crate) u64);

impl Snowflake {
    /// Get creation datetime as UnixTimestamp
    pub fn timestamp(&self) -> UnixTimestamp {
        UnixTimestamp((self.0 >> 22) + 1420070400000)
    }
}

impl Display for Snowflake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Snowflake {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{}", self.0))
    }
}

impl<'de> Deserialize<'de> for Snowflake {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(SnowflakeVistor)
    }
}

struct SnowflakeVistor;

impl<'de> Visitor<'de> for SnowflakeVistor {
    type Value = Snowflake;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "A String encoded 64 bit 'Snowflake'")
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(Snowflake(v.parse().map_err(|e| E::custom(e))?))
    }
}
