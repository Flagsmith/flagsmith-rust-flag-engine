use serde::{Deserialize, Deserializer, Serialize, Serializer};

use serde::de::{self, Unexpected};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum FeatureStateValueType {
    String,
    Bool,
    Integer,
    Float,
    None,
}
#[derive(Clone, Debug)]
pub struct FeatureStateValue {
    pub value_type: FeatureStateValueType,
    pub value: String,
}

impl Serialize for FeatureStateValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.value_type {
            FeatureStateValueType::Bool => serializer.serialize_bool(self.value.parse().unwrap()),
            FeatureStateValueType::Integer => serializer.serialize_i64(self.value.parse().unwrap()),
            FeatureStateValueType::String => serializer.serialize_str(self.value.as_str()),
            FeatureStateValueType::None => serializer.serialize_none(),
            FeatureStateValueType::Float => serializer.serialize_f64(self.value.parse().unwrap()),
        }
    }
}
struct FeatureStateValueVisitor;
impl<'de> de::Visitor<'de> for FeatureStateValueVisitor {
    type Value = FeatureStateValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer, a string, None or boolean")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FeatureStateValue {
            value: v.to_string(),
            value_type: FeatureStateValueType::Integer,
        })
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FeatureStateValue {
            value: v.to_string(),
            value_type: FeatureStateValueType::Integer,
        })
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FeatureStateValue {
            value: "".to_string(),
            value_type: FeatureStateValueType::None,
        })
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FeatureStateValue {
            value: v.to_string(),
            value_type: FeatureStateValueType::String,
        })
    }
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FeatureStateValue {
            value: v.to_string(),
            value_type: FeatureStateValueType::Bool,
        })
    }
}
impl<'de> Deserialize<'de> for FeatureStateValue {
    fn deserialize<D>(deserializer: D) -> Result<FeatureStateValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(FeatureStateValueVisitor)
    }
}
