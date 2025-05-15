use std::env;
use log::debug;
use rusqlite::fallible_iterator::FallibleIterator;
use rusqlite::{Connection, OpenFlags, Row};
use sqlite_vfs_http::{register_http_vfs, HTTP_VFS};

const DB_LOCATION: &str = "https://beta.lmfdb.org/riemann-zeta-zeros/index.db";
const DB_ENV: &str = "ZETA_DB";

#[derive(Debug)]
pub struct Block {
    pub t: f64,
    pub offset: u32,
}

impl TryFrom<&Row<'_>> for Block {
    type Error = rusqlite::Error;
    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Block {
            t: value.get("t").expect("Column `t` not found in row."),
            offset: value
                .get("offset")
                .expect("Column `offset` not found in row."),
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
    pub fn for_file(&mut self, file_name: &str) -> rusqlite::Result<Vec<Block>> {
        self.db
            .prepare("SELECT * FROM zero_index WHERE filename = ?1")?
            .query([file_name])?
            .map(|row| Block::try_from(row))
            .collect()
    }
}
