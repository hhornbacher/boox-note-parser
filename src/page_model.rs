use crate::{
    id::PageUuid,
    json::{Dimensions, Layer},
    utils::{convert_timestamp_to_datetime, parse_json},
};

pub struct PageModelGroup{
    pub page_models: Vec<PageModel>,
}

impl PageModelGroup {
    pub fn read(mut reader: impl std::io::Read) -> crate::error::Result<Self> {
        let container = protobuf::PageModelContainer::read(&mut reader)?;
        let page_models = container
            .page_model
            .iter()
            .map(PageModel::from_protobuf)
            .collect::<crate::error::Result<_>>()?;
        Ok(Self { page_models })
    }
}

#[derive(Debug, Clone)]
pub struct PageModel {
    pub page_id: PageUuid,
    pub layers: Vec<Layer>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub dimensions: Dimensions,
}

impl PageModel {
    pub fn from_protobuf(page_model: &protobuf::PageModel) -> crate::error::Result<Self> {
        let page_model_layers: json::PageModelLayers = parse_json(&page_model.layers_json)?;
        Ok(Self {
            page_id: PageUuid::from_str(&page_model.page_uuid)?,
            layers: page_model_layers.layer_list,
            created: convert_timestamp_to_datetime(page_model.created)?,
            modified: convert_timestamp_to_datetime(page_model.modified)?,
            dimensions: parse_json(&page_model.dimensions_json)?,
        })
    }
}

mod json {
    use serde::Deserialize;

    use crate::json::Layer;

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PageModelLayers {
        pub layer_list: Vec<Layer>,
    }
}

mod protobuf {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct PageModelContainer {
        #[prost(message, repeated, tag = "1")]
        pub page_model: Vec<PageModel>,
    }

    impl PageModelContainer {
        pub fn read(mut reader: impl std::io::Read) -> crate::error::Result<Self> {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf)?;
            Ok(PageModelContainer::decode(&buf[..])?)
        }
    }

    #[derive(Clone, PartialEq, Message)]
    pub struct PageModel {
        #[prost(string, tag = "1")]
        pub page_uuid: String,
        #[prost(string, tag = "2")]
        pub layers_json: String,
        #[prost(uint64, tag = "5")]
        pub created: u64,
        #[prost(uint64, tag = "6")]
        pub modified: u64,
        #[prost(string, tag = "7")]
        pub dimensions_json: String,
    }
}
