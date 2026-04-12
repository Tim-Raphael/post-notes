use serde::{Deserialize, Serialize};

/// Configurable application settings which get derived from command line
/// arguments and the `Config.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Settings {
    /// Settings related to the paths of input files or assets and the like.
    pub path: super::Path,
}
