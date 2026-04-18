use tokio_stream::StreamExt as _;

use crate::notes;
use crate::settings;
use crate::types;
use std::{marker, sync};

pub struct Unsynced;
pub struct Synced;

pub struct Provider<Settings, Reader, State = Unsynced>
where
    Settings: settings::Provide,
    Reader: notes::Read,
{
    settings: sync::Arc<Settings>,
    notes: Vec<notes::types::Note>,
    reader: Reader,
    // TODO: add watcher
    _state: marker::PhantomData<State>,
}

impl<S, R> Provider<S, R, Unsynced>
where
    S: settings::Provide,
    R: notes::Read,
{
    pub fn new(settings: sync::Arc<S>, reader: R) -> Self {
        Self {
            settings,
            reader,
            notes: vec![],
            _state: marker::PhantomData,
        }
    }

    pub async fn sync(&mut self) -> anyhow::Result<()> {
        let files = self.reader.files().await;
        while let Some(file) = files.try_next() {}
        todo!()
    }
}

impl<S, R> notes::Provide for Provider<S, R, Synced>
where
    S: settings::Provide,
    R: notes::Read,
{
    fn notes(&self) -> &[notes::types::Note] {
        &self.notes
    }
}

impl From<sync::Arc<settings::Provider>>
    for Provider<settings::Provider, notes::Reader<settings::Provider>>
{
    fn from(settings: sync::Arc<settings::Provider>) -> Self {
        let reader = notes::Reader::new(settings.clone());
        Self::new(settings, reader)
    }
}

/// Reads markdown note files from the input directory.
///
/// Non-markdown files are ignored. Directory entry and file-read failures are
/// logged and skipped to preserve the current resilient behavior.
pub fn notes(path: &std::path::Path) -> Result<Vec<types::note::Source>> {
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
