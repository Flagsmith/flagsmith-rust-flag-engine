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
