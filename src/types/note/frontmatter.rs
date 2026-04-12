use serde::{Deserialize, Serialize};

/// Front matter properties extracted from a note file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub description: String,
    pub image: Option<String>,
    pub tags: Vec<super::Tag>,
    pub created: String,
    pub modified: Option<String>,
    pub public: bool,
}
