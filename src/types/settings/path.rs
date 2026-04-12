use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::defaults;

/// All settings that can be cofnigured regarding the directories which will be
/// referenced during the site generation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Path {
    /// Input directory path.
    pub input: PathBuf,
    /// Output directory path.
    pub output: PathBuf,
    /// Template directory path.
    pub template: PathBuf,
    /// Asset directory paths.
    pub assets: Vec<PathBuf>,
}

impl Default for Path {
    fn default() -> Self {
        Self {
            input: PathBuf::from(defaults::DEFAULT_INPUT_PATH),
            output: PathBuf::from(defaults::DEFAULT_OUTPUT_PATH),
            template: PathBuf::from(defaults::DEFAULT_TEMPLATE_PATH),
            assets: vec![PathBuf::from(defaults::DEFAULT_ASSET_PATH)],
        }
    }
}
