use crate::lmfdb::lmfdb_resolve;
use futures::{AsyncBufRead, AsyncBufReadExt, TryStreamExt};
use md5::Digest;
use std::cmp::Ordering;
use std::str::FromStr;
use url::Url;

#[derive(Eq, PartialEq)]
pub struct FileDigest {
    pub file_name: String,
    pub order: usize,
    pub digest: Digest,
}

impl From<String> for FileDigest {
    fn from(value: String) -> Self {
        let mut digest = [0u8; 16];
        hex::decode_to_slice(&value[0..32], &mut digest).expect("");
        let digest = Digest(digest);
        let file_name = value[34..].to_string();
        let len = value.len();
        let order = usize::from_str(&value[40..len - 4]).expect("");
        Self {
            file_name,
            order,
            digest,
        }
    }
}

impl PartialOrd<Self> for FileDigest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileDigest {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order.cmp(&other.order)
    }
}

fn md5_url() -> Url {
    lmfdb_resolve("data/md5.txt")
}

async fn repository_reader() -> Result<impl AsyncBufRead, reqwest::Error> {
    let url = md5_url();
    println!("Reading file repository from {url} ...");
    Ok(reqwest::get(url)
        .await?
        .bytes_stream()
        .map_err(std::io::Error::other)
        .into_async_read())
}

pub async fn read_repository() -> Result<Vec<FileDigest>, std::io::Error> {
    repository_reader()
        .await
        .map_err(std::io::Error::other)?
        .lines()
        .map_ok(FileDigest::from)
        .try_collect::<Vec<FileDigest>>()
        .await
        .map(|mut ok| {
            ok.sort();
            ok
        })
}
