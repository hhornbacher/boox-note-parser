use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;

use crate::error::{Error, Result};

pub fn convert_timestamp_to_datetime(ts: u64) -> Result<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp_millis(ts as i64).ok_or_else(|| Error::InvalidTimestamp(ts))
}

pub fn parse_json<T: DeserializeOwned>(json_str: &str) -> Result<T> {
    serde_json::from_str(json_str).map_err(|e| Error::Json {
        error: e,
        json_string: json_str.to_string(),
    })
}
