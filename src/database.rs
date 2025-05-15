use diesel::prelude::*;
use std::cmp::max;
use std::env;
use std::path::PathBuf;

table! {
    zero_index (t) {
        t -> Double,
        N -> BigInt,
        filename -> Text,
        offset -> Integer,
        block_number -> Integer,
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = zero_index)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[allow(non_snake_case)]
#[allow(dead_code)]
struct Row {
    t: f64,
    N: i64,
    filename: String,
    offset: i32,
    block_number: i32,
}

#[derive(Debug)]
pub struct Block {
    pub t: f64,
    pub offset: u32,
}

impl From<Row> for Block {
    fn from(value: Row) -> Self {
        let Row { t, offset, .. } = value;
        Block {
            t,
            offset: max(0, offset) as u32,
        }
    }
}

pub struct DBConnection {
    db: SqliteConnection,
}

impl Default for DBConnection {
    fn default() -> Self {
        let path =
            env::var("CARGO_MANIFEST_DIR").expect("Cargo manifest dir not found in environment.");
        let path = PathBuf::from(path).join("resources/lmf.db");
        Self {
            db: SqliteConnection::establish(path.to_str().unwrap())
                .expect("Cannot open connection to DB."),
        }
    }
}

impl DBConnection {
    pub fn for_file(&mut self, file_name: &str) -> Vec<Block> {
        use self::zero_index::dsl::*;
        zero_index
            .select(Row::as_select())
            .filter(filename.eq(file_name))
            .order(block_number.asc())
            .load(&mut self.db)
            .expect("Unable to query DB.")
            .into_iter()
            .map(Block::from)
            .collect()
    }
}
