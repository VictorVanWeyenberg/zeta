use std::env;
use log::debug;
use rusqlite::fallible_iterator::FallibleIterator;
use rusqlite::{Connection, OpenFlags, Row};
use sqlite_vfs_http::{register_http_vfs, HTTP_VFS};

const DB_LOCATION: &str = "https://beta.lmfdb.org/riemann-zeta-zeros/index.db";
const DB_ENV: &str = "ZETA_DB";

#[derive(Debug)]
pub struct Block {
    pub offset: usize,
    pub filename: String,
    pub block_number: u64,
}

impl TryFrom<&Row<'_>> for Block {
    type Error = rusqlite::Error;
    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Block {
            offset: value
                .get("offset")
                .expect("Column `offset` not found in row."),
            filename: value
                .get("filename")
                .expect("Column `filename` not found in row."),
            block_number: value
                .get("block_number")
                .expect("Column `block_number` not found in row."),
        })
    }
}

pub struct DBConnection {
    db: Connection,
}

impl Default for DBConnection {
    fn default() -> Self {
        let db = if let Ok(db_location) = env::var(DB_ENV) {
            debug!("Using database located at {db_location}");
            Connection::open_with_flags(
                db_location,
                OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
        } else {
            debug!("Could not find \"ZETA_DB\" environment variable.");
            register_http_vfs();
            Connection::open_with_flags_and_vfs(
                DB_LOCATION,
                OpenFlags::SQLITE_OPEN_READ_ONLY,
                HTTP_VFS,
            )
        }.expect("Unable to establish connection to database.");
        Self { db }
    }
}

impl DBConnection {
    pub fn first_block_start_t(&mut self, t: f64) -> rusqlite::Result<Block> {
        self.db.prepare("SELECT offset, filename, block_number FROM zero_index WHERE t <= ?1 ORDER BY t ASC LIMIT 1")?
            .query([t])?
            .map(|row| Block::try_from(row))
            .next()
            .map(|option| option.unwrap())
    }

    pub fn first_block_start_n(&mut self, n: u64) -> rusqlite::Result<Block> {
        self.db.prepare("SELECT offset, filename, block_number FROM zero_index WHERE N <= ?1 ORDER BY N ASC LIMIT 1")?
            .query([n])?
            .map(|row| Block::try_from(row))
            .next()
            .map(|option| option.unwrap())
    }
}
