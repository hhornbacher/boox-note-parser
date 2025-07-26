use crate::{
    id::PageModelId,
    json::{Dimensions, Layer},
};

#[derive(Debug, Clone)]
pub struct PageModel {
    pub page_model_id: PageModelId,
    pub layers: Vec<Layer>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub dimensions: Dimensions,
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
}
