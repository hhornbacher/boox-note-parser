use crate::{
    id::PageModelUuid,
    json::{Dimensions, Layer}, utils::{convert_timestamp_to_datetime, parse_json},
};

#[derive(Debug, Clone)]
pub struct PageModel {
    pub page_model_id: PageModelUuid,
    pub layers: Vec<Layer>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub dimensions: Dimensions,
}

impl PageModel {
    pub fn from_protobuf(model: &protobuf::PageModel) -> crate::error::Result<Self> {
        Ok(Self {
            page_model_id: PageModelUuid::from_str(&model.uuid)?,
            layers: parse_json(&model.layers_json)?,
            created: convert_timestamp_to_datetime(model.created)?,
            modified: convert_timestamp_to_datetime(model.modified)?,
            dimensions: parse_json(&model.dimensions_json)?,
        })
    }
}

mod protobuf {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct PageModel {
        #[prost(string, tag = "1")]
        pub uuid: String,
        #[prost(string, tag = "2")]
        pub layers_json: String,
        #[prost(uint64, tag = "5")]
        pub created: u64,
        #[prost(uint64, tag = "6")]
        pub modified: u64,
        #[prost(string, tag = "7")]
        pub dimensions_json: String,
    }

    impl PageModel {
        pub fn read(mut reader: impl std::io::Read) -> crate::error::Result<Self> {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf)?;
            Ok(PageModel::decode(&buf[..])?)
        }
    }
}
