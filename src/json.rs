use serde::Deserialize;

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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    pub id: u32,
    pub lock: bool,
    pub show: bool,
}
