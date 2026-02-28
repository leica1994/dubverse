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

// ── Dubbing Jobs ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubbingJob {
    pub id: String,
    pub project_dir: String,
    pub video_path: String,
    pub subtitle_count: i32,
    pub reference_mode: String,
    pub reference_audio_path: Option<String>,
    pub tts_plugin_id: Option<String>,
    pub status: String,
    pub current_stage: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubbingStageState {
    pub job_id: String,
    pub stage: String,
    pub status: String,
    pub progress: i32,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubbingTtsItem {
    pub job_id: String,
    pub subtitle_index: i32,
    pub preprocessed_text: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub reference_audio_path: Option<String>,
    pub tts_audio_path: Option<String>,
    pub tts_duration_ms: Option<i64>,
    pub status: String,
    pub retry_count: i32,
    pub error: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsPlugin {
    pub id: String,
    pub name: String,
    pub plugin_type: String,
    pub config_json: String,
    pub requires_ref: bool,
    pub is_enabled: bool,
    pub sort_order: i32,
    pub created_at: String,
}

pub fn get_dubbing_job_by_dir(conn: &Connection, project_dir: &str) -> Result<Option<DubbingJob>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_dir, video_path, subtitle_count, reference_mode, reference_audio_path,
                tts_plugin_id, status, current_stage, error, created_at, updated_at
         FROM dubbing_jobs WHERE project_dir = ?1",
    )?;
    let mut rows = stmt.query([project_dir])?;
    if let Some(row) = rows.next()? {
        Ok(Some(DubbingJob {
            id: row.get(0)?,
            project_dir: row.get(1)?,
            video_path: row.get(2)?,
            subtitle_count: row.get(3)?,
            reference_mode: row.get(4)?,
            reference_audio_path: row.get(5)?,
            tts_plugin_id: row.get(6)?,
            status: row.get(7)?,
            current_stage: row.get(8)?,
            error: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn upsert_dubbing_job(conn: &Connection, job: &DubbingJob) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO dubbing_jobs
         (id, project_dir, video_path, subtitle_count, reference_mode, reference_audio_path,
          tts_plugin_id, status, current_stage, error, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        rusqlite::params![
            job.id, job.project_dir, job.video_path, job.subtitle_count,
            job.reference_mode, job.reference_audio_path, job.tts_plugin_id,
            job.status, job.current_stage, job.error, job.created_at, job.updated_at,
        ],
    )?;
    Ok(())
}

pub fn update_dubbing_job_status(
    conn: &Connection,
    job_id: &str,
    status: &str,
    current_stage: Option<&str>,
    error: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE dubbing_jobs SET status=?2, current_stage=?3, error=?4, updated_at=datetime('now') WHERE id=?1",
        rusqlite::params![job_id, status, current_stage, error],
    )?;
    Ok(())
}

pub fn delete_dubbing_job(conn: &Connection, job_id: &str) -> Result<()> {
    conn.execute("DELETE FROM dubbing_tts_items WHERE job_id=?1", [job_id])?;
    conn.execute("DELETE FROM dubbing_stage_states WHERE job_id=?1", [job_id])?;
    conn.execute("DELETE FROM dubbing_jobs WHERE id=?1", [job_id])?;
    Ok(())
}

pub fn upsert_dubbing_stage(conn: &Connection, state: &DubbingStageState) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO dubbing_stage_states
         (job_id, stage, status, progress, output_path, error, completed_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        rusqlite::params![
            state.job_id, state.stage, state.status, state.progress,
            state.output_path, state.error, state.completed_at,
        ],
    )?;
    Ok(())
}

pub fn get_dubbing_stages(conn: &Connection, job_id: &str) -> Result<Vec<DubbingStageState>> {
    let mut stmt = conn.prepare(
        "SELECT job_id, stage, status, progress, output_path, error, completed_at
         FROM dubbing_stage_states WHERE job_id=?1",
    )?;
    let rows = stmt.query_map([job_id], |row| {
        Ok(DubbingStageState {
            job_id: row.get(0)?,
            stage: row.get(1)?,
            status: row.get(2)?,
            progress: row.get(3)?,
            output_path: row.get(4)?,
            error: row.get(5)?,
            completed_at: row.get(6)?,
        })
    })?;
    let mut result = Vec::new();
    for r in rows { result.push(r?); }
    Ok(result)
}

pub fn bulk_upsert_tts_items(conn: &Connection, items: &[DubbingTtsItem]) -> Result<()> {
    for item in items {
        conn.execute(
            "INSERT OR IGNORE INTO dubbing_tts_items
             (job_id, subtitle_index, preprocessed_text, start_ms, end_ms, status)
             VALUES (?1,?2,?3,?4,?5,'pending')",
            rusqlite::params![
                item.job_id, item.subtitle_index, item.preprocessed_text,
                item.start_ms, item.end_ms,
            ],
        )?;
    }
    Ok(())
}

pub fn get_pending_tts_items(conn: &Connection, job_id: &str) -> Result<Vec<DubbingTtsItem>> {
    let mut stmt = conn.prepare(
        "SELECT job_id, subtitle_index, preprocessed_text, start_ms, end_ms,
                reference_audio_path, tts_audio_path, tts_duration_ms,
                status, retry_count, error, completed_at
         FROM dubbing_tts_items WHERE job_id=?1 AND status != 'completed'
         ORDER BY subtitle_index",
    )?;
    let rows = stmt.query_map([job_id], |row| {
        Ok(DubbingTtsItem {
            job_id: row.get(0)?,
            subtitle_index: row.get(1)?,
            preprocessed_text: row.get(2)?,
            start_ms: row.get(3)?,
            end_ms: row.get(4)?,
            reference_audio_path: row.get(5)?,
            tts_audio_path: row.get(6)?,
            tts_duration_ms: row.get(7)?,
            status: row.get(8)?,
            retry_count: row.get(9)?,
            error: row.get(10)?,
            completed_at: row.get(11)?,
        })
    })?;
    let mut result = Vec::new();
    for r in rows { result.push(r?); }
    Ok(result)
}

pub fn get_all_tts_items(conn: &Connection, job_id: &str) -> Result<Vec<DubbingTtsItem>> {
    let mut stmt = conn.prepare(
        "SELECT job_id, subtitle_index, preprocessed_text, start_ms, end_ms,
                reference_audio_path, tts_audio_path, tts_duration_ms,
                status, retry_count, error, completed_at
         FROM dubbing_tts_items WHERE job_id=?1 ORDER BY subtitle_index",
    )?;
    let rows = stmt.query_map([job_id], |row| {
        Ok(DubbingTtsItem {
            job_id: row.get(0)?,
            subtitle_index: row.get(1)?,
            preprocessed_text: row.get(2)?,
            start_ms: row.get(3)?,
            end_ms: row.get(4)?,
            reference_audio_path: row.get(5)?,
            tts_audio_path: row.get(6)?,
            tts_duration_ms: row.get(7)?,
            status: row.get(8)?,
            retry_count: row.get(9)?,
            error: row.get(10)?,
            completed_at: row.get(11)?,
        })
    })?;
    let mut result = Vec::new();
    for r in rows { result.push(r?); }
    Ok(result)
}

pub fn update_tts_item_completed(
    conn: &Connection,
    job_id: &str,
    subtitle_index: i32,
    tts_audio_path: &str,
    tts_duration_ms: i64,
) -> Result<()> {
    conn.execute(
        "UPDATE dubbing_tts_items SET status='completed', tts_audio_path=?3, tts_duration_ms=?4,
         completed_at=datetime('now'), error=NULL WHERE job_id=?1 AND subtitle_index=?2",
        rusqlite::params![job_id, subtitle_index, tts_audio_path, tts_duration_ms],
    )?;
    Ok(())
}

pub fn update_tts_item_failed(
    conn: &Connection,
    job_id: &str,
    subtitle_index: i32,
    error: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE dubbing_tts_items SET status='failed', error=?3,
         retry_count=retry_count+1 WHERE job_id=?1 AND subtitle_index=?2",
        rusqlite::params![job_id, subtitle_index, error],
    )?;
    Ok(())
}

pub fn update_tts_item_reference(
    conn: &Connection,
    job_id: &str,
    subtitle_index: i32,
    reference_audio_path: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE dubbing_tts_items SET reference_audio_path=?3 WHERE job_id=?1 AND subtitle_index=?2",
        rusqlite::params![job_id, subtitle_index, reference_audio_path],
    )?;
    Ok(())
}

// ── TTS Plugins ──────────────────────────────────────────────────────────────

pub fn get_all_tts_plugins(conn: &Connection) -> Result<Vec<TtsPlugin>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, plugin_type, config_json, requires_ref, is_enabled, sort_order, created_at
         FROM tts_plugins ORDER BY sort_order ASC, created_at ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(TtsPlugin {
            id: row.get(0)?,
            name: row.get(1)?,
            plugin_type: row.get(2)?,
            config_json: row.get(3)?,
            requires_ref: row.get::<_, i32>(4)? != 0,
            is_enabled: row.get::<_, i32>(5)? != 0,
            sort_order: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;
    let mut result = Vec::new();
    for r in rows { result.push(r?); }
    Ok(result)
}

pub fn get_tts_plugin(conn: &Connection, id: &str) -> Result<Option<TtsPlugin>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, plugin_type, config_json, requires_ref, is_enabled, sort_order, created_at
         FROM tts_plugins WHERE id=?1",
    )?;
    let mut rows = stmt.query([id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(TtsPlugin {
            id: row.get(0)?,
            name: row.get(1)?,
            plugin_type: row.get(2)?,
            config_json: row.get(3)?,
            requires_ref: row.get::<_, i32>(4)? != 0,
            is_enabled: row.get::<_, i32>(5)? != 0,
            sort_order: row.get(6)?,
            created_at: row.get(7)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn create_tts_plugin(conn: &Connection, plugin: &TtsPlugin) -> Result<()> {
    conn.execute(
        "INSERT INTO tts_plugins (id, name, plugin_type, config_json, requires_ref, is_enabled, sort_order, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        rusqlite::params![
            plugin.id, plugin.name, plugin.plugin_type, plugin.config_json,
            plugin.requires_ref as i32, plugin.is_enabled as i32,
            plugin.sort_order, plugin.created_at,
        ],
    )?;
    Ok(())
}

pub fn update_tts_plugin(conn: &Connection, plugin: &TtsPlugin) -> Result<()> {
    conn.execute(
        "UPDATE tts_plugins SET name=?2, plugin_type=?3, config_json=?4,
         requires_ref=?5, is_enabled=?6, sort_order=?7 WHERE id=?1",
        rusqlite::params![
            plugin.id, plugin.name, plugin.plugin_type, plugin.config_json,
            plugin.requires_ref as i32, plugin.is_enabled as i32, plugin.sort_order,
        ],
    )?;
    Ok(())
}

pub fn delete_tts_plugin(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM tts_plugins WHERE id=?1", [id])?;
    Ok(())
}

// ── Workbench Tasks ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchTask {
    pub id: String,
    pub name: String,
    pub project_dir: String,
    pub video_path: String,
    pub video_name: String,
    pub video_size: i64,
    pub video_duration: f64,
    pub video_width: i32,
    pub video_height: i32,
    pub current_step: i32,
    pub step_statuses: String,
    pub source_language: String,
    pub target_language: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchStepTranscribe {
    pub task_id: String,
    pub config_json: String,
    pub subtitles_path: Option<String>,
    pub subtitle_count: i32,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchStepTranslate {
    pub task_id: String,
    pub config_json: String,
    pub translated_subtitles_path: Option<String>,
    pub subtitle_count: i32,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchTaskFull {
    pub id: String,
    pub name: String,
    pub project_dir: String,
    pub video_path: String,
    pub video_name: String,
    pub video_size: i64,
    pub video_duration: f64,
    pub video_width: i32,
    pub video_height: i32,
    pub current_step: i32,
    pub step_statuses: String,
    pub source_language: String,
    pub target_language: String,
    pub status: String,
    pub created_at: String,
    pub step_transcribe: Option<WorkbenchStepTranscribe>,
    pub step_translate: Option<WorkbenchStepTranslate>,
}

pub fn create_workbench_task(conn: &Connection, task: &WorkbenchTask) -> Result<()> {
    conn.execute(
        "INSERT INTO workbench_tasks
         (id, name, project_dir, video_path, video_name, video_size, video_duration,
          video_width, video_height, current_step, step_statuses,
          source_language, target_language, status, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
        rusqlite::params![
            task.id, task.name, task.project_dir, task.video_path, task.video_name,
            task.video_size, task.video_duration, task.video_width, task.video_height,
            task.current_step, task.step_statuses, task.source_language, task.target_language,
            task.status, task.created_at, task.updated_at,
        ],
    )?;
    Ok(())
}

pub fn update_workbench_task_progress(
    conn: &Connection,
    id: &str,
    current_step: i32,
    step_statuses: &str,
    source_language: &str,
    target_language: &str,
    status: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE workbench_tasks
         SET current_step=?2, step_statuses=?3, source_language=?4, target_language=?5,
             status=?6, updated_at=datetime('now')
         WHERE id=?1",
        rusqlite::params![id, current_step, step_statuses, source_language, target_language, status],
    )?;
    Ok(())
}

pub fn list_workbench_tasks(conn: &Connection) -> Result<Vec<WorkbenchTask>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, project_dir, video_path, video_name, video_size, video_duration,
                video_width, video_height, current_step, step_statuses,
                source_language, target_language, status, created_at, updated_at
         FROM workbench_tasks ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(WorkbenchTask {
            id: row.get(0)?,
            name: row.get(1)?,
            project_dir: row.get(2)?,
            video_path: row.get(3)?,
            video_name: row.get(4)?,
            video_size: row.get(5)?,
            video_duration: row.get(6)?,
            video_width: row.get(7)?,
            video_height: row.get(8)?,
            current_step: row.get(9)?,
            step_statuses: row.get(10)?,
            source_language: row.get(11)?,
            target_language: row.get(12)?,
            status: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
        })
    })?;
    let mut result = Vec::new();
    for r in rows { result.push(r?); }
    Ok(result)
}

pub fn get_workbench_task_full(conn: &Connection, task_id: &str) -> Result<Option<WorkbenchTaskFull>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, project_dir, video_path, video_name, video_size, video_duration,
                video_width, video_height, current_step, step_statuses,
                source_language, target_language, status, created_at
         FROM workbench_tasks WHERE id=?1",
    )?;
    let mut rows = stmt.query([task_id])?;
    let task = match rows.next()? {
        Some(row) => WorkbenchTaskFull {
            id: row.get(0)?,
            name: row.get(1)?,
            project_dir: row.get(2)?,
            video_path: row.get(3)?,
            video_name: row.get(4)?,
            video_size: row.get(5)?,
            video_duration: row.get(6)?,
            video_width: row.get(7)?,
            video_height: row.get(8)?,
            current_step: row.get(9)?,
            step_statuses: row.get(10)?,
            source_language: row.get(11)?,
            target_language: row.get(12)?,
            status: row.get(13)?,
            created_at: row.get(14)?,
            step_transcribe: None,
            step_translate: None,
        },
        None => return Ok(None),
    };

    let mut full = task;

    // Load transcribe step
    let mut st_stmt = conn.prepare(
        "SELECT task_id, config_json, subtitles_path, subtitle_count, completed_at
         FROM workbench_step_transcribe WHERE task_id=?1",
    )?;
    let mut st_rows = st_stmt.query([task_id])?;
    if let Some(row) = st_rows.next()? {
        full.step_transcribe = Some(WorkbenchStepTranscribe {
            task_id: row.get(0)?,
            config_json: row.get(1)?,
            subtitles_path: row.get(2)?,
            subtitle_count: row.get(3)?,
            completed_at: row.get(4)?,
        });
    }

    // Load translate step
    let mut tr_stmt = conn.prepare(
        "SELECT task_id, config_json, translated_subtitles_path, subtitle_count, completed_at
         FROM workbench_step_translate WHERE task_id=?1",
    )?;
    let mut tr_rows = tr_stmt.query([task_id])?;
    if let Some(row) = tr_rows.next()? {
        full.step_translate = Some(WorkbenchStepTranslate {
            task_id: row.get(0)?,
            config_json: row.get(1)?,
            translated_subtitles_path: row.get(2)?,
            subtitle_count: row.get(3)?,
            completed_at: row.get(4)?,
        });
    }

    Ok(Some(full))
}

pub fn delete_workbench_task(conn: &Connection, task_id: &str) -> Result<()> {
    conn.execute("DELETE FROM workbench_tasks WHERE id=?1", [task_id])?;
    Ok(())
}

pub fn upsert_step_transcribe(conn: &Connection, step: &WorkbenchStepTranscribe) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO workbench_step_transcribe
         (task_id, config_json, subtitles_path, subtitle_count, completed_at)
         VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![
            step.task_id, step.config_json, step.subtitles_path,
            step.subtitle_count, step.completed_at,
        ],
    )?;
    Ok(())
}

pub fn upsert_step_translate(conn: &Connection, step: &WorkbenchStepTranslate) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO workbench_step_translate
         (task_id, config_json, translated_subtitles_path, subtitle_count, completed_at)
         VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![
            step.task_id, step.config_json, step.translated_subtitles_path,
            step.subtitle_count, step.completed_at,
        ],
    )?;
    Ok(())
}
