use log::debug;
use rug::Float;
use zeta::{zero_stream, SeekPattern, ZeroStream};

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let mut sink = ZeroSink;
    let seek_pattern = SeekPattern::StartWithZeroNumberAmount(0, 5000);
    zero_stream(&mut sink, seek_pattern).await.expect("");
}

pub struct ZeroSink;

impl ZeroStream for ZeroSink {
    fn is_closed(&self) -> bool {
        false
    }

    fn send(&mut self, zero_number: u64, zero: Float) {
        debug!("{zero_number}: {zero}");
    }
}
