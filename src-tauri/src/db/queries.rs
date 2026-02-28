use rusqlite::{Connection, Result};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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

// ── AI Configs ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub id: String,
    pub title: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub sort_order: i32,
    pub is_default: bool,
    pub concurrent_limit: i32,
    pub request_timeout: i32,
    pub rate_limit: i32,
}

pub fn get_all_ai_configs(conn: &Connection) -> Result<Vec<AiConfig>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, base_url, api_key, model, sort_order, is_default,
                concurrent_limit, request_timeout, rate_limit
         FROM ai_configs
         ORDER BY is_default DESC, sort_order ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(AiConfig {
            id: row.get(0)?,
            title: row.get(1)?,
            base_url: row.get(2)?,
            api_key: row.get(3)?,
            model: row.get(4)?,
            sort_order: row.get(5)?,
            is_default: row.get::<_, i32>(6)? != 0,
            concurrent_limit: row.get(7)?,
            request_timeout: row.get(8)?,
            rate_limit: row.get(9)?,
        })
    })?;
    let mut configs = Vec::new();
    for row in rows {
        configs.push(row?);
    }
    Ok(configs)
}

pub fn create_ai_config(conn: &Connection, config: &AiConfig) -> Result<()> {
    // If this is the first config, make it default
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM ai_configs", [], |r| r.get(0))?;
    let is_default = if count == 0 { 1 } else { config.is_default as i32 };

    conn.execute(
        "INSERT INTO ai_configs (id, title, base_url, api_key, model, sort_order, is_default,
                                 concurrent_limit, request_timeout, rate_limit)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            config.id,
            config.title,
            config.base_url,
            config.api_key,
            config.model,
            config.sort_order,
            is_default,
            config.concurrent_limit,
            config.request_timeout,
            config.rate_limit,
        ],
    )?;
    Ok(())
}

pub fn update_ai_config(conn: &Connection, config: &AiConfig) -> Result<()> {
    conn.execute(
        "UPDATE ai_configs
         SET title = ?2, base_url = ?3, api_key = ?4, model = ?5,
             sort_order = ?6, is_default = ?7,
             concurrent_limit = ?8, request_timeout = ?9, rate_limit = ?10
         WHERE id = ?1",
        rusqlite::params![
            config.id,
            config.title,
            config.base_url,
            config.api_key,
            config.model,
            config.sort_order,
            config.is_default as i32,
            config.concurrent_limit,
            config.request_timeout,
            config.rate_limit,
        ],
    )?;
    Ok(())
}

pub fn delete_ai_config(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM ai_configs WHERE id = ?1", [id])?;
    Ok(())
}

pub fn set_default_ai_config(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("UPDATE ai_configs SET is_default = 0", [])?;
    conn.execute("UPDATE ai_configs SET is_default = 1 WHERE id = ?1", [id])?;
    Ok(())
}

// ── Translation Progress ────────────────────────────────────────────────────

pub fn get_translation_progress(
    conn: &Connection,
    project_dir: &str,
    phase: &str,
) -> Result<HashMap<i32, String>> {
    let mut stmt = conn.prepare(
        "SELECT subtitle_index, result_text FROM translation_progress WHERE project_dir = ?1 AND phase = ?2",
    )?;
    let rows = stmt.query_map(rusqlite::params![project_dir, phase], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (k, v) = row?;
        map.insert(k, v);
    }
    Ok(map)
}

pub fn save_translation_progress(
    conn: &Connection,
    project_dir: &str,
    subtitle_index: i32,
    phase: &str,
    result_text: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO translation_progress (project_dir, subtitle_index, phase, result_text) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![project_dir, subtitle_index, phase, result_text],
    )?;
    Ok(())
}

pub fn clear_translation_progress(conn: &Connection, project_dir: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM translation_progress WHERE project_dir = ?1",
        [project_dir],
    )?;
    Ok(())
}
