use std::io;
use std::sync;
use tokio;
use tokio_stream;

use crate::notes;
use crate::settings;

pub struct Reader<Settings>
where
    Settings: settings::Provide,
{
    settings: sync::Arc<Settings>,
}

impl<S> Reader<S>
where
    S: settings::Provide,
{
    pub fn new(settings_provider: sync::Arc<S>) -> Self {
        Self {
            settings: settings_provider,
        }
    }
}

impl<S> notes::Read for Reader<S>
where
    S: settings::Provide + Sync + Send + 'static,
{
    async fn files(&self) -> impl tokio_stream::Stream<Item = io::Result<notes::types::RawNote>> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let settings = self.settings.clone();

        tokio::spawn(async move {
            let result = async {
                let mut read_dir = tokio::fs::read_dir(settings.input()).await?;

                while let Some(entry) = read_dir.next_entry().await? {
                    let file_name = entry.file_name();
                    let raw_note = tokio::fs::read(entry.path())
                        .await
                        .map(|content| notes::types::RawNote::from((file_name, content)));

                    let receiver_alive = tx.send(raw_note).await.is_ok();

                    if !receiver_alive {
                        break;
                    }
                }

                Ok(())
            }
            .await;

            if let Err(e) = result {
                tx.send(Err(e)).await;
            }
        });

        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}
