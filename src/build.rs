use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::{fs, io};

use serde_json::json;
use tera::{Context, Tera};

use crate::content_map::ContentMap;
use crate::navigation::Navigation;
use crate::settings::Settings;
use crate::types::PostNote;

/// Builds the static site by rendering templates and copying assets.
///
/// Steps:
/// - Initializes the Tera template engine with HTML templates
/// - Creates the output directory structure
/// - Copies all static asset directories to output
/// - Copies media files referenced in notes
/// - Writes the content map index
/// - Renders all notes using templates
///
/// # Errors
///
/// Returns an error if template loading, directory creation, file copying, or rendering fails.
pub fn website(
    notes: &[PostNote],
    content_map: ContentMap,
    navigation: Navigation,
    settings: &Settings,
) -> anyhow::Result<()> {
    let template_pattern = format!("{}/**/*.html", settings.path.template.display());
    let tera = Tera::new(&template_pattern)?;
    for asset_path in &settings.path.assets {
        static_dir(asset_path, &settings.path.output)?;
    }
    media(notes, &settings.path.input, &settings.path.output)?;
    map(content_map, &settings.path.output)?;
    pages(notes, &navigation, &tera, &settings.path.output)?;

    Ok(())
}

fn pages(
    notes: &[PostNote],
    navigation: &Navigation,
    tera: &Tera,
    output_path: &Path,
) -> anyhow::Result<()> {
    notes.par_iter().for_each(|note| {
        let mut context = Context::new();

        if let Err(err) = context.try_insert("note", note) {
            log::error!("Failed to insert note for {:?}: {}", &note.file_name, err);
            return;
        }

        if let Err(err) = context.try_insert("navigation", navigation) {
            log::error!(
                "Failed to insert navigation for {:?}: {}",
                &note.file_name,
                err
            );
            return;
        }

        let content = match tera.render("base.html", &context) {
            Ok(content) => content,
            Err(err) => {
                log::error!("Rendering failed for {:?}: {}", note.file_name, err);
                return;
            }
        };

        let path = output_path.join(note.file_name.to_string());
        if let Err(err) = fs::write(&path, content) {
            log::error!("Writing failed for {}: {}", path.display(), err);
        } else {
            log::info!("Rendered: {}", path.display());
        }
    });

    Ok(())
}

/// Recursively copies a directory tree from source to destination.
///
/// Creates the destination directory if it doesn't exist. For each entry in the source:
/// - Directories are recursively copied
/// - Files are copied directly
///
/// If destination already exists, contents are merged (existing files are overwritten).
///
/// # Errors
///
/// Returns an error if any filesystem operation fails (reading, creating directories, copying).
fn static_dir(from: &Path, to: &Path) -> io::Result<()> {
    // Ensure the destination directory exists before copying contents.
    fs::create_dir_all(to)?;
    // Iterate through all entries in the source directory.
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let from = entry.path();
        let to = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            // Recursively copy subdirectories.
            static_dir(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }

    Ok(())
}

fn media(notes: &[PostNote], src: &Path, out: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(out)?;
    notes.par_iter().for_each(|note| {
        note.media_links.par_iter().for_each(|media_link| {
            let media_path = PathBuf::from(media_link.to_string());
            let output_media_path = PathBuf::from(media_link.to_string());
            if let Some(parent) = media_path.parent()
                && let Err(err) = fs::create_dir_all(out.join(parent))
            {
                log::warn!("Could not create parent directory: {}", err);
            };
            if let Err(err) = fs::copy(src.join(&media_path), out.join(&output_media_path)) {
                log::warn!(
                    "Could not copy file {:?} into output directory: {}",
                    &src.join(&media_path),
                    err
                );
            }
        })
    });

    Ok(())
}

fn map(content: ContentMap, out: &Path) -> anyhow::Result<()> {
    let map_json = serde_json::to_string(&json!(content))?;
    let path = out.join("map.json");

    fs::write(&path, map_json)?;
    log::info!("Created the content map at: {}", path.display());

    Ok(())
}
