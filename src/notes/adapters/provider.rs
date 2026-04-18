use crate::notes;
use crate::settings;

use tokio::fs;
use tokio_stream::StreamExt as _;

use std::{error, marker, sync};

pub struct Unsynced;
pub struct Synced;

pub struct Provider<Reader, State = Unsynced>
where
    Reader: notes::Read,
{
    notes: Vec<notes::types::Note>,
    reader: Reader,
    // TODO: add watcher
    _state: marker::PhantomData<State>,
}

struct SyncError<R>
where
    R: notes::Read,
{
    source: Box<dyn error::Error>,
    provider: Provider<R, Unsynced>,
}

impl<R> Provider<R, Unsynced>
where
    R: notes::Read,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            notes: vec![],
            _state: marker::PhantomData,
        }
    }

    pub async fn sync(self) -> anyhow::Result<Provider<R, Synced>> {
        let mut notes = Vec::new();

        {
            let raw_notes = self.reader.raw_notes().await;
            tokio::pin!(raw_notes);

            while let Some(raw_note) = raw_notes.try_next().await? {
                match notes::types::Note::try_from(raw_note) {
                    Ok(note) => notes.push(note),
                    Err(err) => { /* log */ }
                }
            }
        }

        Ok(Provider {
            notes,
            reader: self.reader,
            _state: marker::PhantomData::<Synced>,
        })
    }
}

impl<R> notes::Provide for Provider<R, Synced>
where
    R: notes::Read,
{
    fn notes(&self) -> &[notes::types::Note] {
        &self.notes
    }
}

impl From<sync::Arc<settings::Provider>> for Provider<notes::Reader<settings::Provider>> {
    fn from(settings: sync::Arc<settings::Provider>) -> Self {
        let reader = notes::Reader::new(settings.clone());
        Self::new(reader)
    }
}

/// Reads markdown note files from the input directory.
///
/// Non-markdown files are ignored. Directory entry and file-read failures are
/// logged and skipped to preserve the current resilient behavior.
pub fn notes(path: &std::path::Path) -> Result<Vec<types_::note::Source>> {
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
