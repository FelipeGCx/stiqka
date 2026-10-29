use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackInfo {
    pub name_pack: String,
    pub author_pack: String,
    pub folder_path: String,
    pub icon_path: Option<String>,
    pub preserve_images: bool,
}
