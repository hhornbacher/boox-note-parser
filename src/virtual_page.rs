use crate::{
    id::VirtualPageId,
    json::Dimensions,
    utils::{convert_timestamp_to_datetime, parse_json},
};

#[derive(Debug, Clone)]
pub struct VirtualPage {
    pub virtual_page_id: VirtualPageId,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub zoom_scale: f32,
    pub dimensions: Dimensions,
    pub layout: Dimensions,
    pub geo: Dimensions,
    pub geo_layout: String,
    pub template_path: String,
    pub page_number: String,
}

impl VirtualPage {
    pub fn from_protobuf(page: &protobuf::VirtualPage) -> crate::error::Result<Self> {
        Ok(Self {
            virtual_page_id: VirtualPageId::from_str(&page.uuid)?,
            created: convert_timestamp_to_datetime(page.created)?,
            modified: convert_timestamp_to_datetime(page.modified)?,
            zoom_scale: page.zoom_scale,
            dimensions: parse_json(&page.dimensions_json)?,
            layout: parse_json(&page.layout_json)?,
            geo: parse_json(&page.geo_json)?,
            geo_layout: page.geo_layout.clone(),
            template_path: page.template_path.clone(),
            page_number: page.page_number.clone(),
        })
    }
}

mod protobuf {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct VirtualPageContainer {
        #[prost(message, optional, tag = "1")]
        pub virtual_page: Option<VirtualPage>,
    }

    impl VirtualPageContainer {
        pub fn read(mut reader: impl std::io::Read) -> crate::error::Result<Self> {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf)?;
            Ok(VirtualPageContainer::decode(&buf[..])?)
        }
    }

    #[derive(Clone, PartialEq, Message)]
    pub struct VirtualPage {
        #[prost(string, tag = "1")]
        pub uuid: String,
        #[prost(uint64, tag = "2")]
        pub created: u64,
        #[prost(uint64, tag = "3")]
        pub modified: u64,
        #[prost(float, tag = "4")]
        pub zoom_scale: f32,
        #[prost(string, tag = "6")]
        pub dimensions_json: String,
        #[prost(string, tag = "7")]
        pub layout_json: String,
        #[prost(string, tag = "8")]
        pub geo_json: String,
        #[prost(string, tag = "9")]
        pub geo_layout: String,
        #[prost(string, tag = "10")]
        pub template_path: String,
        #[prost(string, tag = "12")]
        pub page_number: String,
    }
}
