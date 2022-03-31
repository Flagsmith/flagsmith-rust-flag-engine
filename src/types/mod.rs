use serde::{Deserialize, Deserializer, Serialize, Serializer};

use serde::de::{self, Unexpected};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum FlagsmithValueType {
    String,
    Bool,
    Integer,
    Float,
    None,
}
#[derive(Clone, Debug)]
pub struct FlagsmithValue {
    pub value_type: FlagsmithValueType,
    pub value: String,
}

impl Serialize for FlagsmithValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.value_type {
            FlagsmithValueType::Bool => serializer.serialize_bool(self.value.parse().unwrap()),
            FlagsmithValueType::Integer => serializer.serialize_i64(self.value.parse().unwrap()),
            FlagsmithValueType::String => serializer.serialize_str(self.value.as_str()),
            FlagsmithValueType::None => serializer.serialize_none(),
            FlagsmithValueType::Float => serializer.serialize_f64(self.value.parse().unwrap()),
        }
    }
}
struct FlagsmithValueVisitor;
impl<'de> de::Visitor<'de> for FlagsmithValueVisitor {
    type Value = FlagsmithValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer, a string, None or boolean")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FlagsmithValue {
            value: v.to_string(),
            value_type: FlagsmithValueType::Integer,
        })
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FlagsmithValue {
            value: v.to_string(),
            value_type: FlagsmithValueType::Integer,
        })
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FlagsmithValue {
            value: "".to_string(),
            value_type: FlagsmithValueType::None,
        })
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FlagsmithValue {
            value: v.to_string(),
            value_type: FlagsmithValueType::String,
        })
    }
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FlagsmithValue {
            value: v.to_string(),
            value_type: FlagsmithValueType::Bool,
        })
    }
}
impl<'de> Deserialize<'de> for FlagsmithValue {
    fn deserialize<D>(deserializer: D) -> Result<FlagsmithValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(FlagsmithValueVisitor)
    }
}
