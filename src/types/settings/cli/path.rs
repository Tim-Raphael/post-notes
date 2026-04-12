use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

/// Optional path settings used to parse command line arguments - mirros
/// [crate::types::settings::Path].
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default, Parser,
)]
pub struct Path {
    /// Input directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<PathBuf>,
    /// Output directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<PathBuf>,
    /// Template directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<PathBuf>,
    /// Asset directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    pub assets: Option<Vec<PathBuf>>,
}
