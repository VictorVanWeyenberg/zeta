use futures::Sink;
use rug::Float;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io;
use log::debug;
use zeta::{zero_stream, ZeroStream};

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut sink = ZeroSink::default();
    zero_stream(&mut sink).await.expect("");
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
        debug!("{item}");
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

impl ZeroStream for ZeroSink {
    fn is_closed(&self) -> bool {
        self.amt >= 5000
    }
}

impl ZeroSink {
    fn track_amt(&mut self) {
        self.amt += 1;
    }
}
