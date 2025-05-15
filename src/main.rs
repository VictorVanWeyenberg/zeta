use crate::dat::FileProcessor;
use crate::database::DBConnection;
use crate::repository::{read_repository, FileDigest};
use futures::{Sink, TryFutureExt};
use rug::Float;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

mod dat;
mod database;
mod lmfdb;
mod repository;

#[tokio::main]
async fn main() {
    read_repository()
        .and_then(to_file_processors)
        .await
        .expect("Error occurred.")
}

async fn to_file_processors(files: Vec<FileDigest>) -> Result<(), std::io::Error> {
    let mut db = DBConnection::default();
    let mut sink = ZeroSink::default();
    for file in files {
        FileProcessor::new(file, &mut db)
            .await?
            .process(&mut sink)
            .await?;
    }
    Ok(())
}

#[derive(Default)]
pub struct ZeroSink {
    amt: usize,
}

impl Sink<Float> for ZeroSink {
    type Error = io::Error;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: Float) -> Result<(), Self::Error> {
        self.track_amt();
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl ZeroSink {
    fn track_amt(&mut self) {
        self.amt += 1;
        print!("\rZeros streamed: {}", self.amt);
    }
}
