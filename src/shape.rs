use uuid::Uuid;

use crate::{
    id::{ShapeId, StrokeId},
    json::Dimensions,
    shape::json::{DisplayScale, LineStyleContainer},
};

#[derive(Debug, Clone)]
pub struct Shape {
    pub shape_id: ShapeId,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub sentinel_i64: i64,
    pub stroke_width: f32,
    pub bbox: Dimensions,
    pub render_scale: DisplayScale,
    pub z_order: i64,
    pub stroke_id: Option<StrokeId>,
    pub line_style: Option<LineStyleContainer>,
    pub group_id: Uuid,
    pub points_json: String,
}

mod json {
    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DisplayScale {
        display_scale: f32,
        max_pressure: f32,
        revised_display_scale: f32,
        source: u32,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LineStyleContainer {
        line_style: LineStyle,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LineStyle {
        pub phase: f32,
        pub type_: u8,
    }
}

mod protobuf {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct ShapeContainer {
        #[prost(message, repeated, tag = "1")]
        pub shape: Vec<Shape>,
    }

    impl ShapeContainer {
        pub fn read(mut reader: impl std::io::Read) -> crate::error::Result<Self> {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf)?;
            Ok(ShapeContainer::decode(&buf[..])?)
        }
    }

    #[derive(Clone, PartialEq, Message)]
    pub struct Shape {
        #[prost(string, tag = "1")]
        pub uuid: String,
        #[prost(uint64, tag = "2")]
        pub created: u64,
        #[prost(uint64, tag = "3")]
        pub modified: u64,
        #[prost(sint64, tag = "4")]
        pub sentinel_i64: i64,
        #[prost(float, tag = "5")]
        pub stroke_width: f32,
        #[prost(string, tag = "7")]
        pub bbox_json: String,
        #[prost(string, tag = "11")]
        pub render_scale_json: String,
        #[prost(sint64, tag = "12")]
        pub z_order: i64,
        #[prost(string, tag = "16")]
        pub related_uuid: String,
        #[prost(string, tag = "17")]
        pub line_style_json: String,
        #[prost(string, tag = "18")]
        pub another_uuid: String,
        #[prost(string, tag = "21")]
        pub empty_array_json: String,
    }
}
