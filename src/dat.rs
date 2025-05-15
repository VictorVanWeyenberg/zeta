use crate::database::{Block, DBConnection};
use crate::lmfdb::lmfdb_data_resolve;
use crate::repository::FileDigest;
use crate::ZeroStream;
use futures::{SinkExt, Stream, TryStreamExt};
use log::debug;
use rug::Float;
use std::io;
use std::ops::Mul;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;
use tokio_util::io::StreamReader;

type BoxedStream = Box<dyn Stream<Item = Result<bytes::Bytes, io::Error>> + Unpin>;

type StreamBoxed = BufReader<StreamReader<BoxedStream, bytes::Bytes>>;

pub struct FileProcessor {
    file_name: String,
    reader: SimpleSeeker,
    blocks: Vec<Block>,
}

impl FileProcessor {
    pub async fn new(
        digest: FileDigest,
        db: &mut DBConnection,
    ) -> Result<FileProcessor, io::Error> {
        let FileDigest { file_name, .. } = digest;
        let file_url = lmfdb_data_resolve(&file_name);
        let reader: BoxedStream = Box::new(
            reqwest::get(file_url)
                .await
                .expect("")
                .bytes_stream()
                .map_err(io::Error::other),
        );
        let reader = SimpleSeeker::new(BufReader::new(StreamReader::new(reader)));
        let blocks = db.for_file(&file_name).map_err(io::Error::other)?;
        Ok(Self {
            file_name,
            reader,
            blocks,
        })
    }

    pub async fn process(self, sink: &mut impl ZeroStream) -> Result<(), io::Error> {
        let FileProcessor {
            file_name,
            mut reader,
            blocks,
        } = self;
        debug!("Processing {file_name} ...");
        assert_number_of_blocks(&mut reader, &blocks).await?;
        for Block { t, offset, .. } in blocks {
            Self::process_block(&mut reader, t, offset, sink).await?;
            if sink.is_closed() {
                break;
            }
        }
        Ok(())
    }

    async fn process_block(
        reader: &mut SimpleSeeker,
        t: f64,
        offset: u32,
        sink: &mut impl ZeroStream,
    ) -> Result<(), io::Error> {
        reader.consume_until(offset as usize);
        let (t0, t1, n_t0, n_t1) = read_block_header(reader).await?;
        let mut z = 0u128;
        let precision = t1.log(2f64) as u32 + 111;
        let eps = Float::with_val(precision, -101f64).exp2();
        assert_eq!(t, t0);
        let number_of_zeros = n_t1 - n_t0;
        for _ in 0..number_of_zeros {
            let (z1, z2, z3) = read_block_entry(reader).await?;
            z += ((z3 as u128) << 96) + ((z2 as u128) << 64) + z1 as u128;
            let zero = Float::with_val(precision, t) + Float::with_val(precision, z).mul(&eps);
            sink.send(zero).await?;
            if sink.is_closed() {
                break;
            }
        }
        Ok(())
    }
}

async fn assert_number_of_blocks(
    reader: &mut SimpleSeeker,
    blocks: &[Block],
) -> Result<(), io::Error> {
    let number_of_blocks = reader.read_u64().await?;
    assert_eq!(number_of_blocks, blocks.len() as u64);
    Ok(())
}

async fn read_block_header(reader: &mut SimpleSeeker) -> Result<(f64, f64, u64, u64), io::Error> {
    Ok((
        reader.read_f64().await?,
        reader.read_f64().await?,
        reader.read_u64().await?,
        reader.read_u64().await?,
    ))
}

async fn read_block_entry(reader: &mut SimpleSeeker) -> Result<(u64, u32, u8), io::Error> {
    Ok((
        reader.read_u64().await?,
        reader.read_u32().await?,
        reader.read_u8().await?,
    ))
}

struct SimpleSeeker {
    reader: StreamBoxed,
    position: usize,
}

impl SimpleSeeker {
    fn new(reader: StreamBoxed) -> Self {
        Self {
            reader,
            position: 0,
        }
    }

    async fn read_u64(&mut self) -> io::Result<u64> {
        let value = self.reader.read_u64_le().await?;
        self.position += 8;
        Ok(value)
    }

    async fn read_f64(&mut self) -> io::Result<f64> {
        let value = self.reader.read_f64_le().await?;
        self.position += 8;
        Ok(value)
    }

    async fn read_u32(&mut self) -> io::Result<u32> {
        let value = self.reader.read_u32_le().await?;
        self.position += 4;
        Ok(value)
    }

    async fn read_u8(&mut self) -> io::Result<u8> {
        let value = self.reader.read_u8().await?;
        self.position += 1;
        Ok(value)
    }

    fn consume_until(&mut self, pos: usize) {
        self.reader.consume(pos - self.position);
        self.position = pos
    }
}
