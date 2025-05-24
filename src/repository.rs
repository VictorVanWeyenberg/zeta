use crate::lmfdb::lmfdb_data_resolve;
use futures::{AsyncBufRead, AsyncBufReadExt, TryStreamExt};
use std::cmp::Ordering;
use std::str::FromStr;
use url::Url;

#[derive(Eq, PartialEq)]
pub struct DatFile {
    pub file_name: String,
    pub order: usize,
}

impl From<String> for DatFile {
    fn from(value: String) -> Self {
        let file_name = value[34..].to_string();
        let len = value.len();
        let order = usize::from_str(&value[40..len - 4]).expect("");
        Self {
            file_name,
            order,
        }
    }
}

impl PartialOrd<Self> for DatFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DatFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order.cmp(&other.order)
    }
}

fn md5_url() -> Url {
    lmfdb_data_resolve("md5.txt")
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

pub async fn read_repository() -> Result<Vec<DatFile>, std::io::Error> {
    repository_reader()
        .await
        .map_err(std::io::Error::other)?
        .lines()
        .map_ok(DatFile::from)
        .try_collect::<Vec<DatFile>>()
        .await
        .map(|mut ok| {
            ok.sort();
            ok
        })
}
