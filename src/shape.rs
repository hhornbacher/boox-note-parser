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

mod protobuf {}
