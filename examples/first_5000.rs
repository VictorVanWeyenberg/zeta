use log::debug;
use rug::Float;
use zeta::{zero_stream, SeekPattern, ZeroStream};

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut sink = ZeroSink::default();
    zero_stream(&mut sink, SeekPattern::StartWithZeroNumberAmount(0, 5000)).await.expect("");
}

#[derive(Default)]
pub struct ZeroSink {
    amt: usize,
}

impl ZeroStream for ZeroSink {
    fn is_closed(&self) -> bool {
        false
    }

    fn send(&mut self, zero_number: u64, zero: Float) {
        debug!("{zero_number}: {zero}");
        self.track_amt();
    }
}

impl ZeroSink {
    fn track_amt(&mut self) {
        self.amt += 1;
    }
}
