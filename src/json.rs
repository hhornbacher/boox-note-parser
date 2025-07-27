use serde::Deserialize;

use crate::id::LayerId;

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

impl Dimensions {
    pub fn print(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}Top: {}", indent_str, self.top);
        println!("{}Right: {}", indent_str, self.right);
        println!("{}Bottom: {}", indent_str, self.bottom);
        println!("{}Left: {}", indent_str, self.left);
        println!("{}Empty: {}", indent_str, self.empty);
        println!("{}Stability: {}", indent_str, self.stability);
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    pub id: LayerId,
    pub lock: bool,
    pub show: bool,
}

impl Layer {
    pub fn print(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}ID: {}", indent_str, self.id);
        println!("{}Lock: {}", indent_str, self.lock);
        println!("{}Show: {}", indent_str, self.show);
    }
}
