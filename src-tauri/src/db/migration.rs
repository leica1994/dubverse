use rusqlite::{Connection, Result};

const SCHEMA: &str = include_str!("../../migrations/schema.sql");
const M003: &str = include_str!("../../migrations/003_translation_progress.sql");

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA)?;
    conn.execute_batch(M003)
}
