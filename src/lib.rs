use crate::dat::FileProcessor;
use crate::database::DBConnection;
use crate::repository::{read_repository, FileDigest};
use futures::TryFutureExt;
use rug::Float;
use std::io;
use std::pin::Pin;

mod dat;
mod database;
mod lmfdb;
mod repository;

pub trait ZeroStream: Unpin {
    fn is_closed(&self) -> bool;
    fn send(&mut self, zero_number: u64, zero: Float);
}

pub enum SeekPattern {
    /// Will stream all zeros from beginning till end until closed.
    None,
    /// Will stream all zeros that are equal to or greater than ... or until closed.
    StartWithImaginaryPart(f64),
    /// Will stream all zeros starting from zero number ... or until closed. (zero-indexed)
    StartWithZeroNumber(u64),
    /// Will stream a given amount of zeros that are equal to or greater than ... or until closed.
    StartWithImaginaryAmount(f64, u64),
    /// Will stream a given amount of zeros starting from zero number ... or until closed. (zero-indexed)
    StartWithZeroNumberAmount(u64, u64),
}

pub async fn zero_stream(
    sink: &mut impl ZeroStream,
    pattern: SeekPattern,
) -> Result<(), io::Error> {
    let mut sink = ZeroPort::new(sink, pattern);
    read_repository()
        .and_then(|files| process_files(files, &mut sink))
        .await
}

async fn process_files(
    files: Vec<FileDigest>,
    sink: &mut ZeroPort<'_>,
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

struct ZeroPort<'a> {
    out_stream: Pin<&'a mut dyn ZeroStream>,
    pattern: SeekPattern,
    amount_sent: u64,
}

impl<'a> ZeroPort<'a> {
    pub fn new(zero_stream: &'a mut impl ZeroStream, pattern: SeekPattern) -> Self {
        Self {
            out_stream: Pin::new(zero_stream),
            pattern,
            amount_sent: 0,
        }
    }

    pub fn is_closed(&self) -> bool {
        self.is_amount_reached() || self.out_stream.is_closed()
    }

    pub fn send(&mut self, zero_number: u64, zero: Float) {
        if self.zero_ok(&zero_number, &zero) && !self.is_amount_reached() {
            self.out_stream.send(zero_number, zero);
            self.amount_sent += 1;
        }
    }

    pub fn block_ok(&self, t0: &f64, t1: &f64, n1: &u64) -> bool {
        match &self.pattern {
            SeekPattern::None => true,
            SeekPattern::StartWithImaginaryPart(start) => (t0 <= start && t1 > start) || t0 > start,
            SeekPattern::StartWithZeroNumber(start) => n1 > start,
            SeekPattern::StartWithImaginaryAmount(start, _) => (t0 <= start && t1 > start) || t0 > start,
            SeekPattern::StartWithZeroNumberAmount(start, _) => n1 > start,
        }
    }

    fn is_amount_reached(&self) -> bool {
        match self.pattern {
            SeekPattern::None => false,
            SeekPattern::StartWithImaginaryPart(_) => false,
            SeekPattern::StartWithZeroNumber(_) => false,
            SeekPattern::StartWithImaginaryAmount(_, amount) => self.amount_sent >= amount,
            SeekPattern::StartWithZeroNumberAmount(_, amount) => self.amount_sent >= amount,
        }
    }

    fn zero_ok(&self, zero_number: &u64, zero: &Float) -> bool {
        match &self.pattern {
            SeekPattern::None => true,
            SeekPattern::StartWithImaginaryPart(start) => zero >= start,
            SeekPattern::StartWithZeroNumber(start) => zero_number >= start,
            SeekPattern::StartWithImaginaryAmount(start, _) => zero >= start,
            SeekPattern::StartWithZeroNumberAmount(start, _) => zero_number >= start,
        }
    }
}
