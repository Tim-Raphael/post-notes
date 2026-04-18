use config;
use std::path;

use crate::{defaults, settings};

/// Optional path settings used to parse command line arguments.
#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    clap::Parser,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct Path {
    /// Input directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<path::PathBuf>,
    /// Output directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<path::PathBuf>,
    /// Template directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub templates: Option<path::PathBuf>,
    /// Asset directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    pub assets: Option<Vec<path::PathBuf>>,
}

/// Command line arguments - mirrors [Settings] structure.
#[derive(
    Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize, clap::Parser,
)]
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
    pub path: Path,
}

/// All settings that can be cofnigured regarding the directories which will be
/// referenced during the site generation.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Settings {
    /// Input directory path.
    pub input: path::PathBuf,
    /// Output directory path.
    pub output: path::PathBuf,
    /// Template directory path.
    pub templates: path::PathBuf,
    /// Asset directory paths.
    pub assets: Vec<path::PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            input: path::PathBuf::from(defaults::INPUT_PATH),
            output: path::PathBuf::from(defaults::OUTPUT_PATH),
            templates: path::PathBuf::from(defaults::TEMPLATE_PATH),
            assets: vec![path::PathBuf::from(defaults::ASSET_PATH)],
        }
    }
}

pub struct Provider {
    settings: settings::types::Settings,
}

impl Provider {
    /// Loads the configured settings from either `Config.toml` or the command line
    /// arguments.
    /// - If both are set the command line arguments overwrites the settings from
    ///   the `Config.toml`.
    /// - If neither are set the default settings are used.
    pub fn new() -> Self {
        let args = clap::Args::parse();

        let default = config::Config::try_from(&settings::types::Settings::default())
            .map_err(|err| log::error!("Could not interpret the default settings as config: {err}"))
            .ok();

        let file = config::Config::builder()
            .add_source(File::with_name(&args.config).required(false))
            .build()
            .map_err(|err| log::error!("Could not interpret config file: {err}"))
            .ok();

        let args = config::Config::try_from(&args)
            .map_err(|err| log::error!("Could not interpret cli arguments: {err}"))
            .ok();

        if let Some(default) = default {
            let raw_settings = {
                let mut raw_settings = config::Config::builder().add_source(default);

                if let Some(file) = file {
                    raw_settings = raw_settings.add_source(file);
                }

                if let Some(args) = args {
                    raw_settings = raw_settings.add_source(args);
                };
            };

            if let Ok(settings) = raw_settings
                .build()
                .map_err(|err| log::error!("Could not build merged settings: {err}"))
                .try_deserialize::<settings::types::Settings>()
                .map_err(|err| log::error!("Could not deserialize merged settings: {err}"))
            {
                return Ok(Self { settings });
            }
        }

        log::warn!(
            "Could not load settings from config file or command line arguments, using default settings instead."
        );

        return Self {
            settings: settings::types::Settings::default(),
        };
    }
}

impl settings::Provide for Provider {
    fn input(&self) -> &path::Path {
        self.settings.input
    }

    fn output(&self) -> &path::Path {
        self.settings.output
    }

    fn templates(&self) -> &path::Path {
        self.settings.templates
    }

    fn assets(&self) -> &path::Path {
        self.settings.assets
    }
}

//#[cfg(test)]
//mod tests {
//    use std::path::PathBuf;
//
//    use super::*;
//    use config::FileFormat;
//    use pretty_assertions::assert_eq;
//
//    #[test]
//    fn test_merge_default_settings_with_config_file() {
//        let expected = settings::types::settings::Settings {
//            path: settings::types::settings::Path {
//                input: PathBuf::from("../notes"),
//                ..settings::types::settings::Path::default()
//            },
//        };
//        let default_settings = Config::try_from(&settings::types::settings::Settings::default()).unwrap();
//        let config_file = Config::builder()
//            .add_source(File::from_str("[path]\ninput='../notes'", FileFormat::Toml))
//            .build()
//            .unwrap();
//        let produced = merge_settings(default_settings, Some(config_file), None).unwrap();
//
//        assert_eq!(expected, produced);
//    }
//
//    #[test]
//    fn test_merge_default_settings_with_args() {
//        let expected = settings::types::settings::Settings {
//            path: settings::types::settings::Path {
//                input: PathBuf::from("../notes"),
//                ..settings::types::settings::Path::default()
//            },
//        };
//        let default_settings = Config::try_from(&settings::types::settings::Settings::default()).unwrap();
//        let args =
//            settings::types::settings::cli::Args::try_parse_from(["post_notes", "-i", "../notes"]).unwrap();
//        let config_args = Config::try_from(&args).unwrap();
//        let produced = merge_settings(default_settings, None, Some(config_args)).unwrap();
//
//        assert_eq!(expected, produced);
//    }
//}
