use crate::{
    id::NoteUuid,
    utils::{convert_timestamp_to_datetime, parse_json},
};
use chrono::{DateTime, Utc};
use json::*;

#[derive(Debug, Clone)]
pub struct NoteMetadata {
    pub note_id: NoteUuid,
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
        let fix_regex = regex::Regex::new(r"(\d+):").unwrap();
        let fixed_pen_settings_json = fix_regex.replace_all(&note.pen_settings_json, "\"$1\":");

        Ok(Self {
            note_id: NoteUuid::from_str(&note.note_id)?,
            created: convert_timestamp_to_datetime(note.created)?,
            modified: convert_timestamp_to_datetime(note.modified)?,
            name: note.note_name.clone(),
            flag: note.flag,
            pen_width: note.pen_width,
            scale_factor: note.scale_factor,
            pen_settings: parse_json(&fixed_pen_settings_json)?,
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

    pub fn print(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}Note ID: {}", indent_str, self.note_id);
        println!("{}Created: {}", indent_str, self.created);
        println!("{}Modified: {}", indent_str, self.modified);
        println!("{}Name: {}", indent_str, self.name);
        println!("{}Flag: {:032b}", indent_str, self.flag);
        println!("{}Pen Width: {}", indent_str, self.pen_width);
        println!("{}Scale Factor: {:.3}", indent_str, self.scale_factor);
        println!("{}Fill Color: {:08x}", indent_str, self.fill_color as u32);
        // Print pen settings
        println!("{}Pen Settings:", indent_str);
        self.pen_settings.print(indent + 2);
        // Print canvas state
        println!("{}Canvas State:", indent_str);
        self.canvas_state.print(indent + 2);
        // Print background config
        println!("{}Background Config:", indent_str);
        self.background_config.print(indent + 2);
        // Print device info
        println!("{}Device Info:", indent_str);
        self.device_info.print(indent + 2);
    }
}

mod json {
    use std::collections::HashMap;

    use serde::Deserialize;
    use uuid::Uuid;

    use crate::{id::{LayerId, PenId}, json::{Dimensions, Layer}};

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PenSettings {
        #[serde(deserialize_with = "crate::utils::deserialize_color")]
        pub fill_color: u32,
        #[serde(deserialize_with = "crate::utils::deserialize_color")]
        pub graphics_shape_color: u32,
        pub graphics_shape_type: u8,
        pub normal_pen_shape_type: u8,
        pub pen_line_style: PenLineStyle,
        #[serde(rename = "penWithMap")]
        pub pen_width_map: HashMap<u8, f32>,
        pub quick_pen_list: QuickPenList,
        pub shape_line_style: PenLineStyle,
    }

    impl PenSettings {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Fill Color: {:08x}", indent_str, self.fill_color);
            println!(
                "{}Graphics Shape Color: {:08x}",
                indent_str, self.graphics_shape_color
            );
            println!(
                "{}Graphics Shape Type: {}",
                indent_str, self.graphics_shape_type
            );
            println!(
                "{}Normal Pen Shape Type: {}",
                indent_str, self.normal_pen_shape_type
            );
            println!("{}Pen Line Style:", indent_str);
            self.pen_line_style.print(indent + 2);
            println!("{}Pen Width Map:", indent_str);
            for (key, value) in &self.pen_width_map {
                println!("{}  {}: {}", indent_str, key, value);
            }
            println!("{}Quick Pen List:", indent_str);
            self.quick_pen_list.print(indent + 2);
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PenLineStyle {
        pub line_style: LineStyle,
    }

    impl PenLineStyle {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Line Style:", indent_str);
            println!("{}  Phase: {}", indent_str, self.line_style.phase);
            println!("{}  Type: {}", indent_str, self.line_style.type_);
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LineStyle {
        pub phase: f32,
        pub type_: u8,
    }

    impl LineStyle {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Phase: {}", indent_str, self.phase);
            println!("{}Type: {}", indent_str, self.type_);
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QuickPenList {
        pub quick_pens: Vec<QuickPen>,
        pub selected_id: PenId,
    }

    impl QuickPenList {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Selected ID: {}", indent_str, self.selected_id);
            println!("{}Quick Pens:", indent_str);
            for quick_pen in &self.quick_pens {
                quick_pen.print(indent + 2);
            }
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QuickPen {
        #[serde(deserialize_with = "crate::utils::deserialize_color")]
        pub color: u32,
        pub id: PenId,
        pub type_: u8,
        pub width: f32,
    }

    impl QuickPen {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}ID: {}", indent_str, self.id);
            println!("{}Color: {:08x}", indent_str, self.color);
            println!("{}Type: {}", indent_str, self.type_);
            println!("{}Width: {}", indent_str, self.width);
        }
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

    impl CanvasState {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!(
                "{}Canvas Expand Type: {}",
                indent_str, self.canvas_expand_type
            );
            println!("{}Cover Page ID: {}", indent_str, self.cover_page_id);
            println!(
                "{}Default Page Rect: {:?}",
                indent_str, self.default_page_rect
            );
            println!("{}Zoom Info:", indent_str);
            self.zoom_info.print(indent + 2);
            println!("{}Page Info Map:", indent_str);
            for (key, page_info) in &self.page_info_map {
                println!("{}  {}: {:?}", indent_str, key, page_info);
            }
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PageInfo {
        pub current_layer_id: LayerId,
        pub height: u32,
        pub last_modify_time: u64,
        pub layer_count: u32,
        pub layer_list: Vec<Layer>,
        pub width: u32,
    }

    impl PageInfo {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Current Layer ID: {}", indent_str, self.current_layer_id);
            println!("{}Height: {}", indent_str, self.height);
            println!("{}Last Modify Time: {}", indent_str, self.last_modify_time);
            println!("{}Layer Count: {}", indent_str, self.layer_count);
            println!("{}Width: {}", indent_str, self.width);
            println!("{}Layers:", indent_str);
            for layer in &self.layer_list {
                layer.print(indent + 2);
            }
        }
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

    impl ZoomInfo {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Fit to Screen: {}", indent_str, self.fit_to_screen);
            println!("{}Scale Type: {}", indent_str, self.scale_type);
            println!("{}View Port Height: {}", indent_str, self.view_port_height);
            println!("{}View Port Width: {}", indent_str, self.view_port_width);
            println!("{}Viewport Scale: {}", indent_str, self.viewport_scale);
            println!("{}View Port Position:", indent_str);
            self.view_port_pos.print(indent + 2);
        }
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

    impl ViewPortPos {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Is Empty: {}", indent_str, self.is_empty);
            println!("{}Pressure: {}", indent_str, self.pressure);
            println!("{}Size: {}", indent_str, self.size);
            println!("{}Tilt X: {}", indent_str, self.tilt_x);
            println!("{}Tilt Y: {}", indent_str, self.tilt_y);
            println!("{}Timestamp: {}", indent_str, self.timestamp);
            println!("{}X: {}", indent_str, self.x);
            println!("{}Y: {}", indent_str, self.y);
        }
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

    impl BackgroundConfig {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Background Settings:", indent_str);
            self.background_config.print(indent + 2);
            println!("{}Document Background:", indent_str);
            self.document_background.print(indent + 2);
            println!("{}Page Backgrounds:", indent_str);
            for (key, page_background) in &self.page_backgrounds {
                println!("{}  {}: {:?}", indent_str, key, page_background);
            }
            println!(
                "{}Use Document Background: {}",
                indent_str, self.use_document_background
            );
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct BackgroundSettings {
        pub apply_all_page: bool,
        pub as_default: bool,
        pub canvas_auto_expand: bool,
        pub scale_type: u8,
    }

    impl BackgroundSettings {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Apply All Page: {}", indent_str, self.apply_all_page);
            println!("{}As Default: {}", indent_str, self.as_default);
            println!(
                "{}Canvas Auto Expand: {}",
                indent_str, self.canvas_auto_expand
            );
            println!("{}Scale Type: {}", indent_str, self.scale_type);
        }
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

    impl DocBackground {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Cloud: {}", indent_str, self.cloud);
            println!("{}Global: {}", indent_str, self.global);
            println!("{}Height: {}", indent_str, self.height);
            println!("{}Resource Index: {}", indent_str, self.res_index);
            println!("{}Type: {}", indent_str, self.type_);
            println!("{}Visible: {}", indent_str, self.visible);
            println!("{}Width: {}", indent_str, self.width);
        }
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

    impl PageBackground {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Cloud: {}", indent_str, self.cloud);
            println!("{}Global: {}", indent_str, self.global);
            println!("{}Height: {}", indent_str, self.height);
            println!("{}Resource ID: {}", indent_str, self.res_id);
            println!("{}Resource Index: {}", indent_str, self.res_index);
            println!("{}Title: {}", indent_str, self.title);
            println!("{}Type: {}", indent_str, self.type_);
            println!("{}Value: {}", indent_str, self.value);
            println!("{}Visible: {}", indent_str, self.visible);
            println!("{}Width: {}", indent_str, self.width);
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DeviceInfo {
        pub device_name: String,
        pub size: DeviceDimensions,
    }

    impl DeviceInfo {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Device Name: {}", indent_str, self.device_name);
            println!("{}Size: {:?}", indent_str, self.size);
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DeviceDimensions {
        pub width: f32,
        pub height: f32,
    }

    impl DeviceDimensions {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Width: {}", indent_str, self.width);
            println!("{}Height: {}", indent_str, self.height);
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PageNameList {
        pub page_name_list: Vec<String>,
    }

    impl PageNameList {
        pub fn print(&self, indent: usize) {
            let indent_str = " ".repeat(indent);
            println!("{}Page Name List:", indent_str);
            for page_name in &self.page_name_list {
                println!("{}  {}", indent_str, page_name);
            }
        }
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
        pub pen_settings_json: String,
        #[prost(string, tag = "12")]
        pub canvas_state_json: String,
        #[prost(string, tag = "13")]
        pub background_config_json: String,
        #[prost(string, tag = "14")]
        pub device_info_json: String,
        #[prost(uint32, tag = "15")]
        pub fill_color: u32,
        #[prost(uint32, tag = "16")]
        pub pen_type: u32,
        #[prost(string, tag = "20")]
        pub active_pages_json: String,
        #[prost(string, tag = "21")]
        pub reserved_pages_json: String,
        #[prost(float, tag = "22")]
        pub canvas_width: f32,
        #[prost(float, tag = "23")]
        pub canvas_height: f32,
        #[prost(string, tag = "24")]
        pub location: String,
        #[prost(uint32, tag = "31")]
        pub has_share_section: u32,
        #[prost(uint32, tag = "32")]
        pub stroke_data_len: u32,
        #[prost(uint32, tag = "37")]
        pub has_share_user: u32,
        #[prost(string, tag = "39")]
        pub share_user: String,
        #[prost(uint32, tag = "40")]
        pub has_json7: u32,
        #[prost(string, tag = "44")]
        pub detached_pages_json: String,
    }
}
