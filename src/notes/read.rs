use super::types;
use std::io;
use tokio_stream;

pub trait Read {
    async fn files(&self) -> impl tokio_stream::Stream<Item = io::Result<types::RawNote>>;
}
