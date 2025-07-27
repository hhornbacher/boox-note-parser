use crate::{
    id::VirtualPageUuid,
    json::Dimensions,
    utils::{convert_timestamp_to_datetime, parse_json},
};

#[derive(Debug, Clone)]
pub struct VirtualPage {
    pub virtual_page_id: VirtualPageUuid,
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
            virtual_page_id: VirtualPageUuid::from_str(&page.uuid)?,
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

    pub fn print(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}Virtual Page ID: {}", indent_str, self.virtual_page_id);
        println!("{}Created: {}", indent_str, self.created);
        println!("{}Modified: {}", indent_str, self.modified);
        println!("{}Zoom Scale: {}", indent_str, self.zoom_scale);
        println!("{}Dimensions:", indent_str);
        self.dimensions.print(indent + 2);
        println!("{}Layout:", indent_str);
        self.layout.print(indent + 2);
        println!("{}Geo:", indent_str);
        self.geo.print(indent + 2);
        println!("{}Geo Layout: {}", indent_str, self.geo_layout);
        println!("{}Template Path: {}", indent_str, self.template_path);
        println!("{}Page Number: {}", indent_str, self.page_number);
    }
}

pub mod protobuf {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct VirtualPageContainer {
        #[prost(message, required, tag = "1")]
        pub virtual_page: VirtualPage,
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
