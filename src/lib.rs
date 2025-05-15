use crate::dat::FileProcessor;
use crate::database::DBConnection;
use crate::repository::{read_repository, FileDigest};
use futures::{Sink, TryFutureExt};
use rug::Float;
use sqlite_vfs_http::register_http_vfs;
use std::io;

mod dat;
mod database;
mod lmfdb;
mod repository;

pub trait ZeroStream: Sink<Float, Error = io::Error> + Unpin {
    fn is_closed(&self) -> bool;
}

pub async fn zero_stream(sink: &mut impl ZeroStream) -> Result<(), io::Error> {
    register_http_vfs();
    read_repository()
        .and_then(|files| to_file_processors(files, sink))
        .await
}

async fn to_file_processors(
    files: Vec<FileDigest>,
    sink: &mut impl ZeroStream,
) -> Result<(), io::Error> {
    let mut db = DBConnection::default();
    for file in files {
        FileProcessor::new(file, &mut db)
            .await?
            .process(sink)
            .await?;
        if sink.is_closed() {
            break;
        }
    }
    Ok(())
}
