use crate::{
    id::{PointsUuid, ShapeGroupUuid, ShapeUuid, StrokeUuid},
    json::Dimensions,
    shape::json::{DisplayScale, LineStyleContainer},
    utils::{convert_timestamp_to_datetime, parse_json},
};

#[derive(Debug, Clone)]
pub struct Shape {
    pub shape_id: ShapeUuid,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub sentinel_i64: i64,
    pub stroke_width: f32,
    pub bbox: Dimensions,
    pub render_scale: DisplayScale,
    pub z_order: i64,
    pub points_id: PointsUuid,
    pub line_style: Option<LineStyleContainer>,
    pub shape_group_id: ShapeGroupUuid,
    pub points_json: String,
}

impl Shape {
    pub fn from_protobuf(shape: protobuf::Shape) -> crate::error::Result<Self> {
        Ok(Self {
            shape_id: ShapeUuid::from_str(&shape.uuid)?,
            created: convert_timestamp_to_datetime(shape.created)?,
            modified: convert_timestamp_to_datetime(shape.modified)?,
            sentinel_i64: shape.sentinel_i64,
            stroke_width: shape.stroke_width,
            bbox: parse_json(&shape.bbox_json)?,
            render_scale: parse_json(&shape.render_scale_json)?,
            z_order: shape.z_order,
            points_id: PointsUuid::from_str(&shape.points_uuid)?,
            line_style: if shape.line_style_json.is_empty() {
                None
            } else {
                Some(parse_json(&shape.line_style_json)?)
            },
            shape_group_id: ShapeGroupUuid::from_str(&shape.another_uuid)?,
            points_json: shape.empty_array_json,
        })
    }

    pub fn print(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}Shape ID: {}", indent_str, self.shape_id);
        println!("{}Created: {}", indent_str, self.created);
        println!("{}Modified: {}", indent_str, self.modified);
        println!("{}Sentinel i64: {}", indent_str, self.sentinel_i64);
        println!("{}Stroke Width: {}", indent_str, self.stroke_width);
        println!("{}Bounding Box: {:?}", indent_str, self.bbox);
        println!("{}Render Scale: {:?}", indent_str, self.render_scale);
        println!("{}Z-Order: {}", indent_str, self.z_order);
        println!("{}Points ID: {}", indent_str, self.points_id);
        if let Some(line_style) = &self.line_style {
            println!("{}Line Style: {:?}", indent_str, line_style);
        }
        println!("{}Shape Group ID: {}", indent_str, self.shape_group_id);
        println!("{}Points JSON: {}", indent_str, self.points_json);
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

pub mod protobuf {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct ShapeContainer {
        #[prost(message, repeated, tag = "1")]
        pub shapes: Vec<Shape>,
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
        pub points_uuid: String,
        #[prost(string, tag = "17")]
        pub line_style_json: String,
        #[prost(string, tag = "18")]
        pub another_uuid: String,
        #[prost(string, tag = "21")]
        pub empty_array_json: String,
    }
}
