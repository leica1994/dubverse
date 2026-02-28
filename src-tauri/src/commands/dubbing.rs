use crate::ai_pool::AiPoolManager;
use crate::db::connection::DbState;
use crate::db::queries::{
    self, DubbingJob, DubbingStageState, DubbingTtsItem,
};
use crate::media::{aligner, composer, reference, separator};
use crate::tts::TtsSynthRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

// ── Cancel State ─────────────────────────────────────────────────────────────

pub struct DubbingCancelState(pub Arc<AtomicBool>);

// ── Serializable DTOs ─────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleEntry {
    pub id: i32,
    pub start_time: f64, // seconds
    pub end_time: f64,   // seconds
    pub text: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DubbingStageStateDto {
    job_id: String,
    stage: String,
    status: String,
    progress: i32,
    output_path: Option<String>,
    error: Option<String>,
    completed_at: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DubbingJobInfoDto {
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
    pub stages: Vec<DubbingStageStateDto>,
}

fn stage_to_dto(s: DubbingStageState) -> DubbingStageStateDto {
    DubbingStageStateDto {
        job_id: s.job_id,
        stage: s.stage,
        status: s.status,
        progress: s.progress,
        output_path: s.output_path,
        error: s.error,
        completed_at: s.completed_at,
    }
}

// ── Progress Events ───────────────────────────────────────────────────────────

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DubbingProgressEvent {
    stage: String,
    percent: f64,
    message: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DubbingStageChangeEvent {
    stage: String,
    status: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DubbingTtsItemDoneEvent {
    index: i32,
    status: String,
    audio_path: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreprocessUpdate {
    index: i32,
    text: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DubbingPreprocessBatchResult {
    updates: Vec<PreprocessUpdate>,
}

fn emit_progress(app: &AppHandle, stage: &str, percent: f64, message: &str) {
    let _ = app.emit("dubbing:progress", DubbingProgressEvent {
        stage: stage.to_string(),
        percent,
        message: message.to_string(),
    });
}

fn emit_stage_change(app: &AppHandle, stage: &str, status: &str) {
    let _ = app.emit("dubbing:stage_change", DubbingStageChangeEvent {
        stage: stage.to_string(),
        status: status.to_string(),
    });
}

fn emit_tts_item_done(app: &AppHandle, index: i32, status: &str, audio_path: Option<String>) {
    let _ = app.emit("dubbing:tts_item_done", DubbingTtsItemDoneEvent {
        index,
        status: status.to_string(),
        audio_path,
    });
}

// ── Helper: set stage status in DB ───────────────────────────────────────────

fn set_stage_status(
    db: &DbState,
    job_id: &str,
    stage: &str,
    status: &str,
    output_path: Option<String>,
    error: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let completed_at = if status == "completed" {
        Some("datetime('now')".to_string())
    } else {
        None
    };
    queries::upsert_dubbing_stage(&conn, &DubbingStageState {
        job_id: job_id.to_string(),
        stage: stage.to_string(),
        status: status.to_string(),
        progress: if status == "completed" { 100 } else { 0 },
        output_path,
        error,
        completed_at,
    })
    .map_err(|e| e.to_string())
}

// ── AI Helpers (reuse translate.rs patterns) ──────────────────────────────────

#[derive(serde::Deserialize)]
struct ChatResponse { choices: Vec<ChatChoice> }
#[derive(serde::Deserialize)]
struct ChatChoice { message: ChatMessage }
#[derive(serde::Deserialize)]
struct ChatMessage { content: String }

struct AiCfg {
    id: String,
    base_url: String,
    api_key: String,
    model: String,
    concurrent_limit: u32,
    request_timeout: u64,
    rate_limit: u32,
}

async fn call_ai_json(
    client: &reqwest::Client,
    cfg: &AiCfg,
    system_prompt: &str,
    user_content: &str,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": cfg.model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_content },
        ],
        "temperature": 0.1,
    });
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", cfg.api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("AI请求失败: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("AI API {status}: {text}"));
    }
    let chat: ChatResponse = resp.json().await.map_err(|e| format!("解析响应失败: {e}"))?;
    chat.choices.into_iter().next()
        .map(|c| c.message.content)
        .ok_or_else(|| "AI返回空choices".to_string())
}

const PREPROCESS_PROMPT: &str = r#"将以下字幕文本处理成适合中文TTS朗读的口语化文本：
规则：
1. 数字转中文读法（1024 → 一千零二十四）
2. 英文缩写展开（如上下文明确则展开，否则保留）
3. 删除括号内的舞台指示（[笑声]、(掌声) 等）
4. 标点规范化（省略号统一为…，感叹号不重复）
5. 保持原意，不翻译，不添加内容
## Rules
- Input/output format: JSON object {"index": "text", ...}
- Output count MUST exactly match input count
- Output ONLY the JSON object"#;

fn repair_json(s: &str) -> String {
    let out = s
        .replace('\u{201C}', "\"")
        .replace('\u{201D}', "\"")
        .replace('\u{2018}', "'")
        .replace('\u{2019}', "'");
    let mut result = String::with_capacity(out.len());
    let chars: Vec<char> = out.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == ',' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() { j += 1; }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                i += 1;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn parse_json_map(raw: &str, expected: usize) -> Result<HashMap<String, String>, String> {
    let try_parse = |s: &str| -> Result<HashMap<String, String>, String> {
        let map: HashMap<String, String> = serde_json::from_str(s)
            .map_err(|e| format!("JSON解析失败: {e}"))?;
        if map.len() != expected {
            return Err(format!("条目数不匹配: 期望{expected}, 实际{}", map.len()));
        }
        Ok(map)
    };
    if let Ok(m) = try_parse(raw) { return Ok(m); }
    let repaired = repair_json(raw);
    if let Ok(m) = try_parse(&repaired) { return Ok(m); }
    if let (Some(start), Some(end)) = (raw.find('{'), raw.rfind('}')) {
        if let Ok(m) = try_parse(&raw[start..=end]) { return Ok(m); }
    }
    Err(format!("无法解析JSON: {}", &raw[..raw.len().min(200)]))
}

// ── Tauri Commands ────────────────────────────────────────────────────────────

/// Initialize or resume a dubbing job for the given project_dir.
#[tauri::command]
pub fn cmd_init_dubbing_job(
    db: State<'_, DbState>,
    project_dir: String,
    video_path: String,
    subtitle_count: i32,
    reference_mode: String,
    reference_audio_path: Option<String>,
    tts_plugin_id: Option<String>,
) -> Result<DubbingJobInfoDto, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Check for existing job
    if let Some(existing) = queries::get_dubbing_job_by_dir(&conn, &project_dir)
        .map_err(|e| e.to_string())?
    {
        // Return existing job info
        let stages = queries::get_dubbing_stages(&conn, &existing.id)
            .map_err(|e| e.to_string())?;
        return Ok(DubbingJobInfoDto {
            id: existing.id,
            project_dir: existing.project_dir,
            video_path: existing.video_path,
            subtitle_count: existing.subtitle_count,
            reference_mode: existing.reference_mode,
            reference_audio_path: existing.reference_audio_path,
            tts_plugin_id: existing.tts_plugin_id,
            status: existing.status,
            current_stage: existing.current_stage,
            error: existing.error,
            stages: stages.into_iter().map(stage_to_dto).collect(),
        });
    }

    // Create new job
    let job_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let job = DubbingJob {
        id: job_id.clone(),
        project_dir: project_dir.clone(),
        video_path: video_path.clone(),
        subtitle_count,
        reference_mode: reference_mode.clone(),
        reference_audio_path: reference_audio_path.clone(),
        tts_plugin_id: tts_plugin_id.clone(),
        status: "pending".to_string(),
        current_stage: None,
        error: None,
        created_at: now.clone(),
        updated_at: now,
    };
    queries::upsert_dubbing_job(&conn, &job).map_err(|e| e.to_string())?;

    Ok(DubbingJobInfoDto {
        id: job_id,
        project_dir,
        video_path,
        subtitle_count,
        reference_mode,
        reference_audio_path,
        tts_plugin_id,
        status: "pending".to_string(),
        current_stage: None,
        error: None,
        stages: vec![],
    })
}

/// Get dubbing job for a project directory (for resume detection).
#[tauri::command]
pub fn cmd_get_dubbing_job(
    db: State<'_, DbState>,
    project_dir: String,
) -> Result<Option<DubbingJobInfoDto>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let job = queries::get_dubbing_job_by_dir(&conn, &project_dir)
        .map_err(|e| e.to_string())?;
    match job {
        None => Ok(None),
        Some(j) => {
            let stages = queries::get_dubbing_stages(&conn, &j.id)
                .map_err(|e| e.to_string())?;
            Ok(Some(DubbingJobInfoDto {
                id: j.id,
                project_dir: j.project_dir,
                video_path: j.video_path,
                subtitle_count: j.subtitle_count,
                reference_mode: j.reference_mode,
                reference_audio_path: j.reference_audio_path,
                tts_plugin_id: j.tts_plugin_id,
                status: j.status,
                current_stage: j.current_stage,
                error: j.error,
                stages: stages.into_iter().map(stage_to_dto).collect(),
            }))
        }
    }
}

/// Delete/reset a dubbing job so the user can start fresh.
#[tauri::command]
pub fn cmd_reset_dubbing_job(
    db: State<'_, DbState>,
    job_id: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    queries::delete_dubbing_job(&conn, &job_id).map_err(|e| e.to_string())
}

/// Cancel ongoing dubbing operations.
#[tauri::command]
pub fn cmd_cancel_dubbing(cancel: State<'_, DubbingCancelState>) -> Result<(), String> {
    cancel.0.store(true, Ordering::Relaxed);
    Ok(())
}

/// Stage 1: Preprocess subtitles via AI for TTS normalization.
#[tauri::command]
pub async fn cmd_run_preprocess(
    app: AppHandle,
    db: State<'_, DbState>,
    pool: State<'_, AiPoolManager>,
    cancel: State<'_, DubbingCancelState>,
    job_id: String,
    subtitles: Vec<SubtitleEntry>,
    batch_size: Option<u32>,
) -> Result<Vec<String>, String> {
    cancel.0.store(false, Ordering::Relaxed);
    emit_stage_change(&app, "preprocess", "running");
    set_stage_status(&db, &job_id, "preprocess", "running", None, None)?;

    let cfg = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let configs = queries::get_all_ai_configs(&conn).map_err(|e| e.to_string())?;
        configs.into_iter().find(|c| c.is_default)
            .ok_or_else(|| "未配置默认 AI 模型，请先在设置中添加".to_string())?
    };
    let ai_cfg = AiCfg {
        id: cfg.id.clone(),
        base_url: cfg.base_url.clone(),
        api_key: cfg.api_key.clone(),
        model: cfg.model.clone(),
        concurrent_limit: cfg.concurrent_limit as u32,
        request_timeout: cfg.request_timeout as u64,
        rate_limit: cfg.rate_limit as u32,
    };
    let client = pool.get_or_create_client(&ai_cfg.id, ai_cfg.request_timeout).await;
    let bs = batch_size.unwrap_or(20) as usize;
    let total = subtitles.len();
    let mut results = vec![String::new(); total];

    let batches: Vec<Vec<usize>> = (0..total)
        .collect::<Vec<_>>()
        .chunks(bs)
        .map(|c| c.to_vec())
        .collect();
    let total_batches = batches.len();

    for (bi, batch) in batches.iter().enumerate() {
        if cancel.0.load(Ordering::Relaxed) {
            set_stage_status(&db, &job_id, "preprocess", "failed", None, Some("已取消".to_string()))?;
            return Err("已取消".to_string());
        }

        let _permit = pool.acquire(&ai_cfg.id, ai_cfg.concurrent_limit, ai_cfg.rate_limit, &cancel.0).await?;
        let user_obj: HashMap<String, &str> = batch.iter()
            .map(|&idx| (idx.to_string(), subtitles[idx].text.as_str()))
            .collect();
        let user_content = serde_json::to_string(&user_obj).unwrap_or_default();

        let mut batch_ok = false;
        for attempt in 0..3u32 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(1 << attempt)).await;
            }
            match call_ai_json(&client, &ai_cfg, PREPROCESS_PROMPT, &user_content).await {
                Ok(raw) => {
                    if let Ok(map) = parse_json_map(&raw, batch.len()) {
                        for &idx in batch {
                            results[idx] = map.get(&idx.to_string())
                                .cloned()
                                .unwrap_or_else(|| subtitles[idx].text.clone());
                        }
                        batch_ok = true;
                        break;
                    }
                }
                Err(_) => {}
            }
        }
        if !batch_ok {
            // Fallback: keep original text
            for &idx in batch {
                results[idx] = subtitles[idx].text.clone();
            }
        }

        // Emit per-batch result so frontend can update the dual-column list in real time
        let updates: Vec<PreprocessUpdate> = batch.iter().map(|&idx| PreprocessUpdate {
            index: idx as i32,
            text: results[idx].clone(),
        }).collect();
        let _ = app.emit("dubbing:preprocess_batch_result", DubbingPreprocessBatchResult { updates });

        let percent = (bi as f64 + 1.0) / total_batches as f64 * 100.0;
        emit_progress(&app, "preprocess", percent, &format!("字幕预处理: {}/{total_batches}", bi + 1));
    }

    set_stage_status(&db, &job_id, "preprocess", "completed", None, None)?;
    emit_stage_change(&app, "preprocess", "completed");
    Ok(results)
}

/// Stage 2: Media separation (FFmpeg).
#[tauri::command]
pub async fn cmd_run_media_separation(
    app: AppHandle,
    db: State<'_, DbState>,
    job_id: String,
    video_path: String,
    work_dir: String,
) -> Result<serde_json::Value, String> {
    emit_stage_change(&app, "media", "running");
    set_stage_status(&db, &job_id, "media", "running", None, None)?;
    emit_progress(&app, "media", 10.0, "正在分离媒体...");

    let result = separator::separate_media(&video_path, &work_dir).await
        .map_err(|e| {
            let _ = set_stage_status(&db, &job_id, "media", "failed", None, Some(e.clone()));
            e
        })?;

    set_stage_status(&db, &job_id, "media", "completed",
        Some(result.vocal_audio_path.clone()), None)?;
    emit_stage_change(&app, "media", "completed");
    emit_progress(&app, "media", 100.0, "媒体分离完成");

    Ok(serde_json::json!({
        "vocalAudioPath": result.vocal_audio_path,
        "silentVideoPath": result.silent_video_path,
    }))
}

/// Stage 3: Reference audio generation.
#[tauri::command]
pub async fn cmd_run_reference_generation(
    app: AppHandle,
    db: State<'_, DbState>,
    job_id: String,
    reference_mode: String,
    vocal_audio_path: Option<String>,
    custom_audio_path: Option<String>,
    subtitle_entries: Vec<SubtitleEntry>,
    work_dir: String,
) -> Result<serde_json::Value, String> {
    emit_stage_change(&app, "reference", "running");
    set_stage_status(&db, &job_id, "reference", "running", None, None)?;

    let results: Vec<(i32, String)> = match reference_mode.as_str() {
        "clone" => {
            let vocal = vocal_audio_path
                .ok_or("clone 模式需要 vocal_audio_path")?;
            let entries: Vec<reference::SubtitleEntry> = subtitle_entries.iter().map(|s| {
                reference::SubtitleEntry {
                    index: s.id,
                    start_ms: (s.start_time * 1000.0) as i64,
                    end_ms: (s.end_time * 1000.0) as i64,
                }
            }).collect();
            reference::extract_reference_clips(&vocal, &entries, &work_dir).await
                .map_err(|e| {
                    let _ = set_stage_status(&db, &job_id, "reference", "failed", None, Some(e.clone()));
                    e
                })?
        }
        "custom" => {
            let custom = custom_audio_path
                .ok_or("custom 模式需要 custom_audio_path")?;
            let dest = reference::prepare_custom_reference(&custom, &work_dir).await
                .map_err(|e| {
                    let _ = set_stage_status(&db, &job_id, "reference", "failed", None, Some(e.clone()));
                    e
                })?;
            // Same reference for all subtitles
            subtitle_entries.iter().map(|s| (s.id, dest.clone())).collect()
        }
        _ => vec![], // "none" mode
    };

    // Store reference paths in DB
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        for (idx, path) in &results {
            let _ = queries::update_tts_item_reference(&conn, &job_id, *idx, path);
        }
    }

    set_stage_status(&db, &job_id, "reference", "completed", None, None)?;
    emit_stage_change(&app, "reference", "completed");
    emit_progress(&app, "reference", 100.0, "参考音频准备完成");

    Ok(serde_json::json!({
        "count": results.len(),
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreprocessedSubtitle {
    pub id: i32,
    pub start_time: f64,
    pub end_time: f64,
    pub preprocessed_text: String,
}

/// Stage 3.5: Initialize TTS items in DB (call before TTS generation).
#[tauri::command]
pub fn cmd_init_tts_items(
    db: State<'_, DbState>,
    job_id: String,
    subtitles: Vec<PreprocessedSubtitle>,
) -> Result<(), String> {
    let items: Vec<DubbingTtsItem> = subtitles.iter().map(|s| DubbingTtsItem {
        job_id: job_id.clone(),
        subtitle_index: s.id,
        preprocessed_text: s.preprocessed_text.clone(),
        start_ms: (s.start_time * 1000.0) as i64,
        end_ms: (s.end_time * 1000.0) as i64,
        reference_audio_path: None,
        tts_audio_path: None,
        tts_duration_ms: None,
        status: "pending".to_string(),
        retry_count: 0,
        error: None,
        completed_at: None,
    }).collect();

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    queries::bulk_upsert_tts_items(&conn, &items).map_err(|e| e.to_string())
}

/// Stage 4: TTS generation with per-item resume support.
#[tauri::command]
pub async fn cmd_run_tts_generation(
    app: AppHandle,
    db: State<'_, DbState>,
    cancel: State<'_, DubbingCancelState>,
    job_id: String,
    plugin_id: Option<String>,
    ncn_voice_id: Option<String>,
    work_dir: String,
) -> Result<serde_json::Value, String> {
    cancel.0.store(false, Ordering::Relaxed);
    emit_stage_change(&app, "tts", "running");
    set_stage_status(&db, &job_id, "tts", "running", None, None)?;

    // Build provider: plugin takes priority; fallback to built-in NCN
    let provider = match plugin_id.as_deref().filter(|s| !s.is_empty()) {
        Some(pid) => {
            let plugin = {
                let conn = db.0.lock().map_err(|e| e.to_string())?;
                queries::get_tts_plugin(&conn, pid)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("TTS 插件 {pid} 不存在"))?
            };
            super::tts_plugin::build_provider(&plugin)?
        }
        None => {
            use crate::tts::{NcnProvider, TtsProviderImpl};
            TtsProviderImpl::Ncn(NcnProvider::new(ncn_voice_id.unwrap_or_default()))
        }
    };

    // Load pending items
    let pending_items = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        queries::get_pending_tts_items(&conn, &job_id).map_err(|e| e.to_string())?
    };
    let total_items = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        queries::get_all_tts_items(&conn, &job_id)
            .map_err(|e| e.to_string())?
            .len()
    };

    let tts_dir = format!("{}/tts", &work_dir);
    std::fs::create_dir_all(&tts_dir)
        .map_err(|e| format!("创建 TTS 目录失败: {e}"))?;

    let mut completed = 0usize;
    let already_done = total_items - pending_items.len();

    for item in &pending_items {
        if cancel.0.load(Ordering::Relaxed) {
            set_stage_status(&db, &job_id, "tts", "failed", None, Some("已取消".to_string()))?;
            return Err("已取消".to_string());
        }

        let output_path = format!("{}/tts_{:04}.mp3", tts_dir, item.subtitle_index);
        let req = TtsSynthRequest {
            text: item.preprocessed_text.clone(),
            voice_id: None,
            reference_audio_path: item.reference_audio_path.clone(),
            output_path: output_path.clone(),
        };

        let mut success = false;
        for attempt in 0..3u32 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
            }
            match provider.synthesize(TtsSynthRequest {
                text: req.text.clone(),
                voice_id: req.voice_id.clone(),
                reference_audio_path: req.reference_audio_path.clone(),
                output_path: req.output_path.clone(),
            }).await {
                Ok(resp) => {
                    {
                        let conn = db.0.lock().map_err(|e| e.to_string())?;
                        queries::update_tts_item_completed(
                            &conn,
                            &job_id,
                            item.subtitle_index,
                            &resp.audio_path,
                            resp.duration_ms as i64,
                        ).map_err(|e| e.to_string())?;
                    }
                    emit_tts_item_done(&app, item.subtitle_index, "completed", Some(resp.audio_path));
                    success = true;
                    break;
                }
                Err(e) => {
                    if attempt == 2 {
                        let conn = db.0.lock().map_err(|e| e.to_string())?;
                        let _ = queries::update_tts_item_failed(
                            &conn, &job_id, item.subtitle_index, &e,
                        );
                        emit_tts_item_done(&app, item.subtitle_index, "failed", None);
                    }
                }
            }
        }
        if !success {
            // Continue with next item (partial failure allowed)
        }

        completed += 1;
        let percent = (already_done + completed) as f64 / total_items as f64 * 100.0;
        emit_progress(&app, "tts", percent,
            &format!("TTS生成: {}/{total_items}", already_done + completed));
    }

    set_stage_status(&db, &job_id, "tts", "completed", None, None)?;
    emit_stage_change(&app, "tts", "completed");
    emit_progress(&app, "tts", 100.0, "TTS生成完成");

    Ok(serde_json::json!({
        "completed": already_done + completed,
        "total": total_items,
    }))
}

/// Stages 5+6: Audio alignment and video composition.
#[tauri::command]
pub async fn cmd_run_alignment_and_compose(
    app: AppHandle,
    db: State<'_, DbState>,
    job_id: String,
    silent_video_path: String,
    work_dir: String,
    output_path: String,
) -> Result<serde_json::Value, String> {
    // Stage 5: alignment
    emit_stage_change(&app, "alignment", "running");
    set_stage_status(&db, &job_id, "alignment", "running", None, None)?;
    emit_progress(&app, "alignment", 10.0, "正在对齐音频...");

    // Load all TTS items from DB
    let tts_items = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        queries::get_all_tts_items(&conn, &job_id).map_err(|e| e.to_string())?
    };

    // Only use completed items
    let segments: Vec<aligner::AlignedSegment> = tts_items.iter()
        .filter(|item| item.status == "completed" && item.tts_audio_path.is_some())
        .map(|item| aligner::AlignedSegment {
            subtitle_index: item.subtitle_index,
            start_ms: item.start_ms,
            tts_audio_path: item.tts_audio_path.clone().unwrap(),
            tts_duration_ms: item.tts_duration_ms.unwrap_or(0),
        })
        .collect();

    if segments.is_empty() {
        let err = "没有可用的 TTS 音频（所有项目均失败）".to_string();
        set_stage_status(&db, &job_id, "alignment", "failed", None, Some(err.clone()))?;
        return Err(err);
    }

    // Get total duration from last item
    let total_ms = tts_items.last().map(|i| i.end_ms + 1000).unwrap_or(60000);

    let dubbed_audio = aligner::align_and_concat(&segments, total_ms, &work_dir).await
        .map_err(|e| {
            let _ = set_stage_status(&db, &job_id, "alignment", "failed", None, Some(e.clone()));
            e
        })?;

    set_stage_status(&db, &job_id, "alignment", "completed",
        Some(dubbed_audio.clone()), None)?;
    emit_stage_change(&app, "alignment", "completed");
    emit_progress(&app, "alignment", 100.0, "音频对齐完成");

    // Stage 6: compose
    emit_stage_change(&app, "compose", "running");
    set_stage_status(&db, &job_id, "compose", "running", None, None)?;
    emit_progress(&app, "compose", 10.0, "正在合成视频...");

    // Ensure output directory exists
    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建输出目录失败: {e}"))?;
    }

    composer::compose_video(&silent_video_path, &dubbed_audio, &output_path).await
        .map_err(|e| {
            let _ = set_stage_status(&db, &job_id, "compose", "failed", None, Some(e.clone()));
            e
        })?;

    set_stage_status(&db, &job_id, "compose", "completed",
        Some(output_path.clone()), None)?;
    emit_stage_change(&app, "compose", "completed");
    emit_progress(&app, "compose", 100.0, "视频合成完成");

    // Update job status to completed
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        queries::update_dubbing_job_status(&conn, &job_id, "completed", None, None)
            .map_err(|e| e.to_string())?;
    }

    Ok(serde_json::json!({ "outputPath": output_path }))
}
