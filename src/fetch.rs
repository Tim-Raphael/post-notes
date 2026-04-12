use anyhow::Result;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Raw note source loaded from disk.
pub type RawNoteSource = (PathBuf, String);

/// Reads markdown note files from the input directory.
///
/// Non-markdown files are ignored. Directory entry and file-read failures are
/// logged and skipped to preserve the current resilient behavior.
pub fn notes(path: &Path) -> Result<Vec<RawNoteSource>> {
    Ok(fs::read_dir(path)?
        .par_bridge()
        .filter_map(|entry_result| match entry_result {
            Ok(entry) => Some(entry.path()),
            Err(err) => {
                log::error!("Could get directory entry: {err}");
                None
            }
        })
        .filter(|path_buf| {
            path_buf
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext_str| ext_str == "md")
                .unwrap_or(false)
        })
        .filter_map(|path_buf| {
            let raw_content = match fs::read_to_string(&path_buf) {
                Ok(raw_content) => raw_content,
                Err(err) => {
                    log::error!(
                        "Could not read content of {:?}: {}",
                        path_buf.display(),
                        err
                    );
                    return None;
                }
            };

            Some((path_buf, raw_content))
        })
        .collect())
}
