use crate::{
    id::{PageUuid, VirtualDocUuid},
    utils::{convert_timestamp_to_datetime, parse_json},
    virtual_doc::json::Content,
};

#[derive(Debug, Clone)]
pub struct VirtualDoc {
    pub virtual_doc_id: VirtualDocUuid,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub page_id: PageUuid,
    pub stability: f32,
    pub content: Content,
}

impl VirtualDoc {
    pub fn from_protobuf(doc: &protobuf::VirtualDoc) -> crate::error::Result<Self> {
        Ok(Self {
            virtual_doc_id: VirtualDocUuid::from_str(&doc.uuid)?,
            created: convert_timestamp_to_datetime(doc.created)?,
            modified: convert_timestamp_to_datetime(doc.modified)?,
            page_id: PageUuid::from_str(&doc.template_uuid)?,
            stability: doc.stability,
            content: parse_json(&doc.content_json)?,
        })
    }

    pub fn print(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}Virtual Doc ID: {}", indent_str, self.virtual_doc_id);
        println!("{}Created: {}", indent_str, self.created);
        println!("{}Modified: {}", indent_str, self.modified);
        println!("{}Page ID: {}", indent_str, self.page_id);
        println!("{}Stability: {}", indent_str, self.stability);
        println!("{}Content: {:?}", indent_str, self.content);
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

pub mod protobuf {
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
