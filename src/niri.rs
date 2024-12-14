use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct NiriWindow {
    pub id: u32,
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub workspace_id: u8,
    pub is_focused: bool
}

#[derive(Deserialize)]
pub struct NiriWorkspace {
    pub id: u8,
    pub idx: u32,
    pub name: Option<String>,
    pub output: String,
    pub is_active: bool,
    pub is_focused: bool,
    pub active_window_id: Option<u32>
}