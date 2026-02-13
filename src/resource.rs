use chrono::{DateTime, Utc};

use crate::{
    error::{Error, Result},
    id::ResourceUuid,
    utils::convert_timestamp_to_datetime,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceRecord {
    pub path: String,
    pub resource_id: ResourceUuid,
    pub timestamp_millis: u64,
    pub timestamp: DateTime<Utc>,
    pub size_bytes: u64,
    pub payload: ResourcePayload,
}

impl ResourceRecord {
    pub fn read(
        path: impl Into<String>,
        mut reader: impl std::io::Read,
        size_bytes: u64,
    ) -> Result<Self> {
        let path = path.into();
        let file_name = file_name_from_path(&path)?;
        let (resource_id, timestamp_millis) = parse_resource_file_name(file_name)?;
        let timestamp = convert_timestamp_to_datetime(timestamp_millis)?;

        let payload = if size_bytes == 0 {
            ResourcePayload::Empty
        } else {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf)?;
            if buf.is_empty() {
                ResourcePayload::Empty
            } else {
                ResourcePayload::Raw(buf)
            }
        };

        Ok(Self {
            path,
            resource_id,
            timestamp_millis,
            timestamp,
            size_bytes,
            payload,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResourcePayload {
    Empty,
    Raw(Vec<u8>),
}

pub fn parse_resource_file_name(file_name: &str) -> Result<(ResourceUuid, u64)> {
    let mut segments = file_name.split('#');
    let resource_id = segments.next();
    let timestamp = segments.next();
    let extra = segments.next();

    if resource_id.is_none() || timestamp.is_none() || extra.is_some() {
        return Err(Error::InvalidArchiveEntryName(file_name.to_string()));
    }

    let resource_id = ResourceUuid::from_str(resource_id.unwrap())?;
    let timestamp_millis = timestamp.unwrap().parse::<u64>().map_err(|e| {
        Error::InvalidTimestampFormat(format!(
            "Failed to parse resource timestamp in '{}': {}",
            file_name, e
        ))
    })?;

    Ok((resource_id, timestamp_millis))
}

fn file_name_from_path(path: &str) -> Result<&str> {
    path.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| Error::InvalidArchiveEntryName(path.to_string()))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{ResourcePayload, ResourceRecord, parse_resource_file_name};
    use crate::id::ResourceUuid;

    #[test]
    fn parses_resource_filename() {
        let (resource_id, timestamp) =
            parse_resource_file_name("b199553d-a169-4f7f-ae01-f9312c9cbd3c#1753456222446").unwrap();

        assert_eq!(
            resource_id,
            ResourceUuid::from_str("b199553d-a169-4f7f-ae01-f9312c9cbd3c").unwrap()
        );
        assert_eq!(timestamp, 1753456222446);
    }

    #[test]
    fn parses_empty_resource_payload() {
        let record = ResourceRecord::read(
            "resource/pb/b199553d-a169-4f7f-ae01-f9312c9cbd3c#1753456222446",
            Cursor::new(Vec::<u8>::new()),
            0,
        )
        .unwrap();

        assert!(matches!(record.payload, ResourcePayload::Empty));
        assert_eq!(record.size_bytes, 0);
    }
}
