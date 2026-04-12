use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::defaults;

/// Command line arguments - mirrors [crate::types::settings::Settings] structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Parser)]
#[command(name = "post-notes")]
#[command(about = "Building a cute digital garden.")]
#[command(version)]
pub struct Args {
    /// Config file path.
    #[arg(short, long, default_value = defaults::CONFIG_PATH )]
    #[serde(skip)]
    pub config: String,
    /// Path settings.
    #[command(flatten)]
    pub path: super::Path,
}
