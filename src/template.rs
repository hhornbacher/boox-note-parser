use std::collections::HashMap;

use serde::Deserialize;

use crate::{error::Result, id::PageUuid, json::Dimensions, utils::parse_json};

pub const DEFAULT_TEMPLATE_FILE_NAME: &str = ".template_json";
pub const TEMPLATE_FILE_SUFFIX: &str = ".template_json";

#[derive(Debug, Clone)]
pub struct TemplateSet {
    pub default: Option<TemplateDescriptor>,
    pub by_page: HashMap<PageUuid, TemplateDescriptor>,
    pub by_other_id: HashMap<String, TemplateDescriptor>,
}

impl TemplateSet {
    pub fn new() -> Self {
        Self {
            default: None,
            by_page: HashMap::new(),
            by_other_id: HashMap::new(),
        }
    }

    pub fn insert_if_absent(&mut self, key: TemplateKey, descriptor: TemplateDescriptor) {
        match key {
            TemplateKey::Default => {
                if self.default.is_none() {
                    self.default = Some(descriptor);
                }
            }
            TemplateKey::Page(page_id) => {
                self.by_page.entry(page_id).or_insert(descriptor);
            }
            TemplateKey::Other(id) => {
                self.by_other_id.entry(id).or_insert(descriptor);
            }
        }
    }

    pub fn for_page(&self, page_id: &PageUuid) -> Option<&TemplateDescriptor> {
        self.by_page.get(page_id)
    }
}

impl Default for TemplateSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateKey {
    Default,
    Page(PageUuid),
    Other(String),
}

pub fn classify_template_file_name(file_name: &str) -> Option<TemplateKey> {
    if file_name == DEFAULT_TEMPLATE_FILE_NAME {
        return Some(TemplateKey::Default);
    }

    let id = file_name.strip_suffix(TEMPLATE_FILE_SUFFIX)?;
    if id.is_empty() {
        return None;
    }

    if let Ok(page_id) = PageUuid::from_str(id) {
        return Some(TemplateKey::Page(page_id));
    }

    Some(TemplateKey::Other(id.to_string()))
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateDescriptor {
    #[serde(rename = "type")]
    pub type_: String,
    pub properties: TemplateProperties,
}

impl TemplateDescriptor {
    pub fn read(mut reader: impl std::io::Read) -> Result<Self> {
        let mut json_string = String::new();
        reader.read_to_string(&mut json_string)?;
        parse_json(&json_string)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateProperties {
    pub layout_type: String,
    pub page_margins: PageMargins,
    pub radius: f32,
    pub resource_attr: ResourceAttr,
    pub selection_point_type: String,
    pub sub_type: String,
    pub under_content: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageMargins {
    pub layout_rect: Dimensions,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub padding_right: f32,
    pub padding_top: f32,
    pub page_padding_rect: Dimensions,
    pub spacing: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceAttr {
    pub res_name: String,
}

#[cfg(test)]
mod tests {
    use super::{TemplateDescriptor, TemplateKey, classify_template_file_name};
    use crate::id::PageUuid;

    #[test]
    fn classifies_default_template_filename() {
        let key = classify_template_file_name(".template_json").unwrap();
        assert_eq!(key, TemplateKey::Default);
    }

    #[test]
    fn classifies_page_template_filename() {
        let key =
            classify_template_file_name("ba338e220eda49268c7126a02970a160.template_json").unwrap();
        assert_eq!(
            key,
            TemplateKey::Page(PageUuid::from_str("ba338e220eda49268c7126a02970a160").unwrap())
        );
    }

    #[test]
    fn classifies_non_uuid_template_filename_as_other() {
        let key = classify_template_file_name("custom-background.template_json").unwrap();
        assert_eq!(key, TemplateKey::Other("custom-background".to_string()));
    }

    #[test]
    fn parses_template_descriptor_json() {
        let json = r#"{"properties":{"layoutType":"LayoutResVector","pageMargins":{"layoutRect":{"bottom":0.0,"empty":true,"left":0.0,"right":0.0,"stability":0,"top":0.0},"paddingBottom":0.0,"paddingLeft":0.0,"paddingRight":0.0,"paddingTop":0.0,"pagePaddingRect":{"bottom":0.0,"empty":true,"left":0.0,"right":0.0,"stability":0,"top":0.0},"spacing":10.0},"radius":0.0,"resourceAttr":{"resName":"com.onyx.android.note:drawable/ic_to_do_list"},"selectionPointType":"SCALE","subType":"Layout","underContent":false},"type":"Feature"}"#;
        let descriptor = TemplateDescriptor::read(std::io::Cursor::new(json)).unwrap();

        assert_eq!(descriptor.type_, "Feature");
        assert_eq!(
            descriptor.properties.resource_attr.res_name,
            "com.onyx.android.note:drawable/ic_to_do_list"
        );
        assert_eq!(descriptor.properties.page_margins.spacing, 10.0);
    }
}
