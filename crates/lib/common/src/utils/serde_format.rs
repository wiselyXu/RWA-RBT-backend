use mongodb::bson::DateTime;
use serde::{self, Serializer, Deserializer, Deserialize};
use chrono::{TimeZone, Utc, DateTime as ChronoDateTime};

const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S"; // Example: "2024-03-15 10:30:00"

pub mod mongodb_datetime_as_string {
    use super::*;

    pub fn serialize<S>(date: &DateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert BSON DateTime to milliseconds, then to chrono DateTime<Utc>
        let millis = date.timestamp_millis();
        let chrono_datetime: ChronoDateTime<Utc> = Utc.timestamp_millis_opt(millis).single()
            .ok_or_else(|| serde::ser::Error::custom("Invalid BSON DateTime encountered during serialization"))?;
        let s = format!("{}", chrono_datetime.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Parse the string into chrono DateTime<Utc>
        let chrono_datetime_res = Utc.datetime_from_str(&s, "%Y-%m-%d %H:%M:%S%.f %Z")
            .or_else(|_| Utc.datetime_from_str(&s, FORMAT));
        let chrono_datetime: ChronoDateTime<Utc> = chrono_datetime_res
            .map_err(|e| serde::de::Error::custom(format!("Failed to parse datetime string '{}': {}", s, e)))?;
        // Convert chrono DateTime<Utc> to milliseconds, then to BSON DateTime
        let millis = chrono_datetime.timestamp_millis();
        Ok(DateTime::from_millis(millis))
    }
}

pub mod option_mongodb_datetime_as_string {
     use super::*;

    pub fn serialize<S>(date: &Option<DateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(d) => {
                 // Convert BSON DateTime to milliseconds, then to chrono DateTime<Utc>
                 let millis = d.timestamp_millis();
                 let chrono_datetime: ChronoDateTime<Utc> = Utc.timestamp_millis_opt(millis).single()
                     .ok_or_else(|| serde::ser::Error::custom("Invalid BSON DateTime encountered during serialization"))?;
                 let s = format!("{}", chrono_datetime.format(FORMAT));
                 serializer.serialize_some(&s)
            },
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s_val) => {
                // Parse the string into chrono DateTime<Utc>
                let chrono_datetime_res = Utc.datetime_from_str(&s_val, "%Y-%m-%d %H:%M:%S%.f %Z")
                    .or_else(|_| Utc.datetime_from_str(&s_val, FORMAT));
                let chrono_datetime: ChronoDateTime<Utc> = chrono_datetime_res
                    .map_err(|e| serde::de::Error::custom(format!("Failed to parse datetime string '{}': {}", s_val, e)))?;
                 // Convert chrono DateTime<Utc> to milliseconds, then to BSON DateTime
                 let millis = chrono_datetime.timestamp_millis();
                 Ok(Some(DateTime::from_millis(millis)))
            }
            None => Ok(None),
        }
    }
}

// Optional: Add formats for NaiveDate, etc., if needed elsewhere 