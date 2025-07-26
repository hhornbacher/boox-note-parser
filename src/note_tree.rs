use crate::{id::NoteId, utils::{convert_timestamp_to_datetime, parse_json}};
use chrono::{DateTime, Utc};
use json::*;

#[derive(Debug, Clone)]
pub struct NoteMetadata {
    pub note_id: NoteId,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub name: String,
    pub flag: u32,
    pub pen_width: f32,
    pub scale_factor: f32,
    pub pen_settings: PenSettings,
    pub canvas_state: CanvasState,
    pub background_config: BackgroundConfig,
    pub device_info: DeviceInfo,
    pub fill_color: u32,
    pub pen_type: u32,
    pub active_pages: PageNameList,
    pub reserved_pages: PageNameList,
    pub canvas_width: f32,
    pub canvas_height: f32,
    pub location: String,
    pub has_share_section: u32,
    pub stroke_data_len: u32,
    pub has_share_user: u32,
    pub share_user: String,
    pub has_json7: u32,
    pub detached_pages: PageNameList,
}

impl NoteMetadata {
    pub fn from_protobuf(note: &protobuf::NoteMetadata) -> crate::error::Result<Self> {
        Ok(Self {
            note_id: NoteId::from_str(&note.note_id)?,
            created: convert_timestamp_to_datetime(note.created)?,
            modified: convert_timestamp_to_datetime(note.modified)?,
            name: note.note_name.clone(),
            flag: note.flag,
            pen_width: note.pen_width,
            scale_factor: note.scale_factor,
            pen_settings: parse_json(&note.pen_settings_json)?,
            canvas_state: parse_json(&note.canvas_state_json)?,
            background_config: parse_json(&note.background_config_json)?,
            device_info: parse_json(&note.device_info_json)?,
            fill_color: note.fill_color,
            pen_type: note.pen_type,
            active_pages: parse_json(&note.active_pages_json)?,
            reserved_pages: parse_json(&note.reserved_pages_json)?,
            canvas_width: note.canvas_width,
            canvas_height: note.canvas_height,
            location: note.location.clone(),
            has_share_section: note.has_share_section,
            stroke_data_len: note.stroke_data_len,
            has_share_user: note.has_share_user,
            share_user: note.share_user.clone(),
            has_json7: note.has_json7,
            detached_pages: parse_json(&note.detached_pages_json)?,
        })
    }
}


mod json {
    use std::collections::HashMap;

    use serde::Deserialize;
    use uuid::Uuid;

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PenSettings {
        pub fill_color: i32,
        pub graphics_shape_color: i32,
        pub graphics_shape_type: u8,
        pub normal_pen_shape_type: u8,
        pub pen_line_style: PenLineStyle,
        #[serde(rename = "penWithMap")]
        pub pen_width_map: HashMap<u8, f32>,
        pub quick_pen_list: QuickPenList,
        pub shape_line_style: PenLineStyle,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PenLineStyle {
        pub line_style: LineStyle,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LineStyle {
        pub phase: f32,
        pub type_: u8,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QuickPenList {
        pub quick_pens: Vec<QuickPen>,
        pub selected_id: String,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QuickPen {
        pub color: i32,
        pub id: String,
        pub type_: u8,
        pub width: f32,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CanvasState {
        pub canvas_expand_type: String,
        pub cover_page_id: String,
        pub default_page_rect: Dimensions,
        pub page_info_map: HashMap<String, PageInfo>,
        pub zoom_info: ZoomInfo,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PageInfo {
        pub current_layer_id: u32,
        pub height: u32,
        pub last_modify_time: u64,
        pub layer_count: u32,
        pub layer_list: Vec<Layer>,
        pub width: u32,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ZoomInfo {
        pub fit_to_screen: bool,
        pub scale_type: u8,
        pub view_port_height: f32,
        pub view_port_pos: ViewPortPos,
        pub view_port_width: f32,
        pub viewport_scale: f32,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ViewPortPos {
        pub is_empty: bool,
        pub pressure: f32,
        pub size: f32,
        pub tilt_x: i32,
        pub tilt_y: i32,
        pub timestamp: u64,
        pub x: f32,
        pub y: f32,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct BackgroundConfig {
        #[serde(rename = "bkGroundConfig")]
        pub background_config: BackgroundSettings,
        #[serde(rename = "docBKGround")]
        pub document_background: DocBackground,
        #[serde(rename = "pageBKGroundMap")]
        pub page_backgrounds: HashMap<Uuid, PageBackground>,
        #[serde(rename = "useDocBKGround")]
        pub use_document_background: bool,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct BackgroundSettings {
        pub apply_all_page: bool,
        pub as_default: bool,
        pub canvas_auto_expand: bool,
        pub scale_type: u8,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DocBackground {
        pub cloud: bool,
        pub global: bool,
        pub height: f32,
        pub res_index: u32,
        pub type_: u32,
        pub visible: bool,
        pub width: f32,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PageBackground {
        pub cloud: bool,
        pub global: bool,
        pub height: f32,
        pub res_id: String,
        pub res_index: u32,
        pub title: String,
        pub type_: u32,
        pub value: String,
        pub visible: bool,
        pub width: f32,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DeviceInfo {
        pub device_name: String,
        pub size: DeviceDimensions,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DeviceDimensions {
        pub width: f32,
        pub height: f32,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PageNameList {
        pub page_name_list: Vec<String>,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Layer {
        pub id: u32,
        pub lock: bool,
        pub show: bool,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Dimensions {
        pub top: f32,
        pub right: f32,
        pub bottom: f32,
        pub left: f32,
        pub empty: bool,
        pub stability: u32,
    }
}

pub mod protobuf {
    use prost::Message;

    use crate::error::Result;

    #[derive(Clone, PartialEq, Message)]
    pub struct NoteTree {
        #[prost(message, repeated, tag = "1")]
        pub notes: Vec<NoteMetadata>,
    }

    impl NoteTree {
        pub fn read(mut reader: impl std::io::Read) -> Result<Self> {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf)?;
            Ok(NoteTree::decode(&buf[..])?)
        }
    }

    #[derive(Clone, PartialEq, Message)]
    pub struct NoteMetadata {
        #[prost(string, tag = "1")]
        pub note_id: String,
        #[prost(uint64, tag = "2")]
        pub created: u64,
        #[prost(uint64, tag = "3")]
        pub modified: u64,
        #[prost(string, tag = "6")]
        pub note_name: String,
        #[prost(uint32, tag = "8")]
        pub flag: u32,
        #[prost(float, tag = "9")]
        pub pen_width: f32,
        #[prost(float, tag = "10")]
        pub scale_factor: f32,
        #[prost(string, tag = "11")]
        pub pen_settings_json: String, // Formerly json1
        #[prost(string, tag = "12")]
        pub canvas_state_json: String, // Formerly json2
        #[prost(string, tag = "13")]
        pub background_config_json: String, // Formerly json3
        #[prost(string, tag = "14")]
        pub device_info_json: String, // Formerly json4
        #[prost(uint32, tag = "15")]
        pub fill_color: u32,
        #[prost(uint32, tag = "16")]
        pub pen_type: u32,
        #[prost(string, tag = "20")]
        pub active_pages_json: String, // Formerly json5
        #[prost(string, tag = "21")]
        pub reserved_pages_json: String, // Formerly json6
        #[prost(float, tag = "22")]
        pub canvas_width: f32,
        #[prost(float, tag = "23")]
        pub canvas_height: f32,
        #[prost(string, tag = "24")]
        pub location: String,
        #[prost(uint32, tag = "31")]
        pub has_share_section: u32, // always 1 in the samples
        #[prost(uint32, tag = "32")]
        pub stroke_data_len: u32, // varies with note complexity
        #[prost(uint32, tag = "37")]
        pub has_share_user: u32, // always 1
        #[prost(string, tag = "39")]
        pub share_user: String,
        #[prost(uint32, tag = "40")]
        pub has_json7: u32, // always 1
        #[prost(string, tag = "44")]
        pub detached_pages_json: String, // Formerly json7
    }
}
