use rusqlite::{Connection, Result};

const SCHEMA: &str = include_str!("../../migrations/schema.sql");

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA)
}
