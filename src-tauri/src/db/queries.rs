use rusqlite::{Connection, Result};
use std::collections::HashMap;

pub fn get_all_config(conn: &Connection) -> Result<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM app_config")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (k, v) = row?;
        map.insert(k, v);
    }
    Ok(map)
}

pub fn set_config(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO app_config (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
        [key, value],
    )?;
    Ok(())
}

pub fn get_provider_secret(conn: &Connection, provider_id: &str) -> Result<Option<String>> {
    let mut stmt =
        conn.prepare("SELECT secret_json FROM provider_secrets WHERE provider_id = ?1")?;
    let mut rows = stmt.query([provider_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_provider_secret(conn: &Connection, provider_id: &str, secret_json: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO provider_secrets (provider_id, secret_json, updated_at) VALUES (?1, ?2, datetime('now'))",
        [provider_id, secret_json],
    )?;
    Ok(())
}
