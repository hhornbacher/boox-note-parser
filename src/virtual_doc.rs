use uuid::Uuid;

use crate::{id::VirtualDocId, utils::{convert_timestamp_to_datetime, parse_json}, virtual_doc::json::Content};

#[derive(Debug, Clone)]
pub struct VirtualDoc {
    pub virtual_doc_id: VirtualDocId,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub template_uuid: Uuid,
    pub stability: f32,
    pub content: Content,
}

impl VirtualDoc {
    pub fn from_protobuf(doc: &protobuf::VirtualDoc) -> crate::error::Result<Self> {
        Ok(Self {
            virtual_doc_id: VirtualDocId::from_str(&doc.uuid)?,
            created: convert_timestamp_to_datetime(doc.created)?,
            modified: convert_timestamp_to_datetime(doc.modified)?,
            template_uuid: uuid::Uuid::parse_str(&doc.template_uuid)
                .map_err(|e| crate::error::Error::UuidParse(e))?,
            stability: doc.stability,
            content: parse_json(&doc.content_json)?,
        })
    }
}

mod json {
    use serde::Deserialize;

    use crate::json::Dimensions;

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Content {
        pub content_id: String,
        pub content_page_id: String,
        pub content_page_size: Dimensions,
        pub content_relative_path: String,
        pub content_type: String,
    }
}

mod protobuf {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct VirtualDoc {
        #[prost(string, tag = "1")]
        pub uuid: String,
        #[prost(uint64, tag = "2")]
        pub created: u64,
        #[prost(uint64, tag = "3")]
        pub modified: u64,
        #[prost(string, tag = "4")]
        pub template_uuid: String,
        #[prost(float, tag = "5")]
        pub stability: f32,
        #[prost(string, tag = "9")]
        pub content_json: String,
    }

    impl VirtualDoc {
        pub fn read(mut reader: impl std::io::Read) -> crate::error::Result<Self> {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf)?;
            Ok(VirtualDoc::decode(&buf[..])?)
        }
    }
}
