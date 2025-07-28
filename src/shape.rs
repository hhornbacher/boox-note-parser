use crate::{
    id::{PointsUuid, ShapeGroupUuid, StrokeUuid},
    json::Dimensions,
    shape::json::{DisplayScale, LineStyleContainer},
    utils::{convert_timestamp_to_datetime, parse_json},
};

#[derive(Debug, Clone)]
pub struct ShapeGroup {
    shapes: Vec<Shape>,
}

impl ShapeGroup {
    pub fn read(mut reader: impl std::io::Read) -> crate::error::Result<Self> {
        let container = protobuf::ShapeContainer::read(&mut reader)?;
        let shapes = container
            .shapes
            .iter()
            .map(Shape::from_protobuf)
            .collect::<crate::error::Result<_>>()?;
        Ok(Self { shapes })
    }
}

#[derive(Debug, Clone)]
pub struct Shape {
    pub stroke_id: StrokeUuid,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub unknown: i64,
    pub stroke_width: f32,
    pub bbox: Dimensions,
    pub render_scale: DisplayScale,
    pub z_order: i64,
    pub points_id: Option<PointsUuid>,
    pub line_style: Option<LineStyleContainer>,
    pub shape_group_id: ShapeGroupUuid,
    pub points_json: String,
}

impl Shape {
    fn from_protobuf(shape: &protobuf::Shape) -> crate::error::Result<Self> {
        Ok(Self {
            stroke_id: StrokeUuid::from_str(&shape.stroke_uuid)?,
            created: convert_timestamp_to_datetime(shape.created)?,
            modified: convert_timestamp_to_datetime(shape.modified)?,
            unknown: shape.unknown,
            stroke_width: shape.stroke_width,
            bbox: parse_json(&shape.bbox_json)?,
            render_scale: parse_json(&shape.render_scale_json)?,
            z_order: shape.z_order,
            points_id: if shape.points_uuid.is_empty() {
                None
            } else {
                Some(PointsUuid::from_str(&shape.points_uuid)?)
            },
            line_style: if shape.line_style_json.is_empty() {
                None
            } else {
                Some(parse_json(&shape.line_style_json)?)
            },
            shape_group_id: ShapeGroupUuid::from_str(&shape.shape_group_uuid)?,
            points_json: shape.empty_array_json.clone(),
        })
    }
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

    use crate::error::Result;

    #[derive(Clone, PartialEq, Message)]
    pub struct ShapeContainer {
        #[prost(message, repeated, tag = "1")]
        pub shapes: Vec<Shape>,
    }

    impl ShapeContainer {
        pub fn read(mut reader: impl std::io::Read) -> Result<Self> {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf)?;
            Ok(ShapeContainer::decode(&buf[..])?)
        }
    }

    #[derive(Clone, PartialEq, Message)]
    pub struct Shape {
        #[prost(string, tag = "1")]
        pub stroke_uuid: String,
        #[prost(uint64, tag = "2")]
        pub created: u64,
        #[prost(uint64, tag = "3")]
        pub modified: u64,
        #[prost(sint64, tag = "4")]
        pub unknown: i64,
        #[prost(float, tag = "5")]
        pub stroke_width: f32,
        #[prost(string, tag = "7")]
        pub bbox_json: String,
        #[prost(string, tag = "11")]
        pub render_scale_json: String,
        #[prost(sint64, tag = "12")]
        pub z_order: i64,
        #[prost(string, tag = "16")]
        pub points_uuid: String,
        #[prost(string, tag = "17")]
        pub line_style_json: String,
        #[prost(string, tag = "18")]
        pub shape_group_uuid: String,
        #[prost(string, tag = "21")]
        pub empty_array_json: String,
    }
}
