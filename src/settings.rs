use anyhow::Error;
use clap::Parser;
use config::{Config, File};

use crate::types;

/// Read Settings from `Config.toml` or command line arguments.
fn merge_settings(
    default: Config,
    file: Option<Config>,
    args: Option<Config>,
) -> Result<types::settings::Settings, Error> {
    let mut raw_settings = Config::builder().add_source(default);
    if let Some(file) = file {
        raw_settings = raw_settings.add_source(file);
    }
    if let Some(args) = args {
        raw_settings = raw_settings.add_source(args);
    };

    Ok(raw_settings
        .build()?
        .try_deserialize::<types::settings::Settings>()?)
}

/// Loads the configured settings from either `Config.toml` or the command line
/// arguments.
/// - If both are set the command line arguments overwrites the settings from
///   the `Config.toml`.
/// - If neither are set the default settings are used.
pub fn get_settings() -> types::settings::Settings {
    let args = types::settings::cli::Args::parse();
    // Interpret default settings.
    let config_default = Config::try_from(&types::settings::Settings::default())
        .map_err(|err| log::error!("Could not interpret the default settings as config: {err}"))
        .ok();
    // Load and interpret config file.
    let config_file = Config::builder()
        .add_source(File::with_name(&args.config).required(false))
        .build()
        .map_err(|err| log::error!("Could not interpret config file: {err}"))
        .ok();
    // Interpret cli arguments.
    let config_args = Config::try_from(&args)
        .map_err(|err| log::error!("Could not interpret cli arguments: {err}"))
        .ok();
    // If we have a default config, try to merge everything.
    if let Some(default) = config_default {
        if let Ok(settings) = merge_settings(default, config_file, config_args) {
            return settings;
        }
        log::error!("Could not merge settings.");
    }
    log::info!(
        "Could not load settings from config file or command line arguments, using default settings instead."
    );

    types::settings::Settings::default()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use config::FileFormat;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_merge_default_settings_with_config_file() {
        let expected = types::settings::Settings {
            path: types::settings::Path {
                input: PathBuf::from("../notes"),
                ..types::settings::Path::default()
            },
        };
        let default_settings = Config::try_from(&types::settings::Settings::default()).unwrap();
        let config_file = Config::builder()
            .add_source(File::from_str("[path]\ninput='../notes'", FileFormat::Toml))
            .build()
            .unwrap();
        let produced = merge_settings(default_settings, Some(config_file), None).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn test_merge_default_settings_with_args() {
        let expected = types::settings::Settings {
            path: types::settings::Path {
                input: PathBuf::from("../notes"),
                ..types::settings::Path::default()
            },
        };
        let default_settings = Config::try_from(&types::settings::Settings::default()).unwrap();
        let args =
            types::settings::cli::Args::try_parse_from(["post_notes", "-i", "../notes"]).unwrap();
        let config_args = Config::try_from(&args).unwrap();
        let produced = merge_settings(default_settings, None, Some(config_args)).unwrap();

        assert_eq!(expected, produced);
    }
}
