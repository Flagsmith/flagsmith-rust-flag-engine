// A module that helps serialize/deserialize datetime that may or may not have timezone using the rfc3339 format
// NOTE: Datetime that don't have timezone info are assumed to be in UTC
use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Deserializer, Serializer};

pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&date.to_rfc3339())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let datetime = match DateTime::parse_from_rfc3339(&s) {
        Ok(datetime) => datetime,
        // TODO: pattern match on ErrorKind once it's public
        // ref: https://github.com/chronotope/chrono/issues/319
        Err(error) => match error.to_string().as_str() {
            "premature end of input" => {
                let datetime_str = s + "Z";
                DateTime::parse_from_rfc3339(&datetime_str).map_err(serde::de::Error::custom)?
            }
            _ => panic!("Invalid datetime"),
        },
    };
    let datatime_utc = datetime.with_timezone(&Utc);
    return Ok(datatime_utc);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;
    use chrono::Utc;
    use rstest::*;
    use serde::Serialize;

    #[rstest]
    #[case(serde_json::json!({"datetime":"2021-11-29T17:15:51.694223Z"}))]
    #[case(serde_json::json!({"datetime":"2021-11-29T17:15:51.694223"}))]
    #[case(serde_json::json!({"datetime":"2021-11-29T17:15:51.694223+00:00"}))]
    fn serialize_deserialize_datetime(#[case] given_datetime: serde_json::Value) {
        // Given
        #[derive(Serialize, Deserialize)]
        struct AStruct {
            #[serde(with = "utils::datetime")]
            datetime: DateTime<Utc>,
        }

        // When
        let deserialized_struct: AStruct = serde_json::from_value(given_datetime.clone()).unwrap();

        // Then
        assert_eq!(deserialized_struct.datetime.timezone(), Utc);

        // and
        let serialized_struct = serde_json::to_value(deserialized_struct).unwrap();
        assert_eq!(
            serialized_struct,
            serde_json::json!({"datetime":"2021-11-29T17:15:51.694223+00:00"})
        );
    }
}
