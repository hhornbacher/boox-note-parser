use crate::{
    error::Result, id::PageUuid, json::Dimensions, utils::{convert_timestamp_to_datetime, parse_json}
};

#[derive(Debug, Clone)]
pub struct VirtualPage {
    pub page_id: PageUuid,
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
    pub fn read(mut reader: impl std::io::Read) -> Result<Self> {
        let container = protobuf::VirtualPageContainer::read(&mut reader)?;
        Ok(Self {
            page_id: PageUuid::from_str(&container.virtual_page.uuid)?,
            created: convert_timestamp_to_datetime(container.virtual_page.created)?,
            modified: convert_timestamp_to_datetime(container.virtual_page.modified)?,
            zoom_scale: container.virtual_page.zoom_scale,
            dimensions: parse_json(&container.virtual_page.dimensions_json)?,
            layout: parse_json(&container.virtual_page.layout_json)?,
            geo: parse_json(&container.virtual_page.geo_json)?,
            geo_layout: container.virtual_page.geo_layout.clone(),
            template_path: container.virtual_page.template_path.clone(),
            page_number: container.virtual_page.page_number.clone(),
        })
    }
}

pub mod protobuf {
    use prost::Message;

    use crate::error::Result;

    #[derive(Clone, PartialEq, Message)]
    pub struct VirtualPageContainer {
        #[prost(message, required, tag = "1")]
        pub virtual_page: VirtualPage,
    }

    impl VirtualPageContainer {
        pub fn read(mut reader: impl std::io::Read) -> Result<Self> {
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
