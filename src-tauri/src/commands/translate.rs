use crate::ai_pool::AiPoolManager;
use crate::db::connection::DbState;
use crate::db::queries;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

// ── State & Types ────────────────────────────────────────────────────────────

pub struct TranslateCancelState(pub Arc<AtomicBool>);

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleItem {
    pub id: i32,
    pub start_time: f64,
    pub end_time: f64,
    pub text: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TranslateProgress {
    phase: String,
    batch: u32,
    total_batches: u32,
    skipped: u32,
    percent: f64,
    message: String,
}

struct TranslateOpts {
    target_language: String,
    correction: bool,
    optimization: bool,
    prompt_type: String,
    batch_size: usize,
    world_building: String,
    writing_style: String,
    glossary: String,
    forbidden: String,
    examples: String,
    custom_prompt: String,
    prompt_correction: String,
    prompt_standard: String,
    prompt_reflective: String,
    prompt_optimize: String,
}

// AI config resolved from DB
struct ResolvedConfig {
    id: String,
    base_url: String,
    api_key: String,
    model: String,
    concurrent_limit: u32,
    request_timeout: u64,
    rate_limit: u32,
}

// ── Prompts ──────────────────────────────────────────────────────────────────

const JSON_RULES: &str = r#"
## Rules
- Input/output format: JSON object {"index": "text", ...}
- Output count MUST exactly match input count
- Never merge, split, or omit any entry
- Output ONLY the JSON object, no extra text or explanation"#;

const DEFAULT_CORRECTION_CORE: &str = "You are a subtitle correction assistant. Fix ASR transcription errors including:\n- Misrecognized words and homophones\n- Missing or incorrect punctuation\n- Obvious spelling mistakes\nKeep the ORIGINAL language — do NOT translate.";

const DEFAULT_STANDARD_CORE: &str = "You are a professional subtitle translator.\n- Preserve the original meaning, tone, and style\n- Use natural expressions appropriate for the target language\n- Keep proper nouns unless they have well-known translations";

const DEFAULT_REFLECTIVE_CORE: &str = "You are an expert subtitle translator. For each subtitle, internally perform these 4 steps (do NOT output intermediate steps):\n1. Literal translation\n2. Free/idiomatic translation\n3. Compare both, revise for accuracy and naturalness\n4. Produce the final polished translation\n\nOutput ONLY the final result.";

const DEFAULT_OPTIMIZE_CORE: &str = "You are a subtitle polishing assistant. Improve the already-translated subtitles:\n- Enhance fluency and naturalness\n- Fix awkward phrasing\n- Ensure consistency in terminology and style\n- Do NOT change the meaning";

#[derive(Clone, Copy)]
enum Phase {
    Correction,
    Standard,
    Reflective,
    Optimize,
}

// ── Prompt Builder ───────────────────────────────────────────────────────────

fn build_configurable_section(opts: &TranslateOpts) -> String {
    let mut parts = Vec::new();
    if !opts.world_building.is_empty() {
        parts.push(format!("[Background / World Building]\n{}", opts.world_building));
    }
    if !opts.writing_style.is_empty() {
        parts.push(format!("[Writing Style]\n{}", opts.writing_style));
    }
    if !opts.glossary.is_empty() {
        parts.push(format!(
            "[Glossary - Use these exact translations]\n{}",
            opts.glossary
        ));
    }
    if !opts.forbidden.is_empty() {
        parts.push(format!(
            "[Forbidden - Never use these words/translations]\n{}",
            opts.forbidden
        ));
    }
    if !opts.examples.is_empty() {
        parts.push(format!("[Examples - Follow this translation style]\n{}", opts.examples));
    }
    if !opts.custom_prompt.is_empty() {
        parts.push(format!("[Additional Instructions]\n{}", opts.custom_prompt));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("\n{}", parts.join("\n\n"))
    }
}

fn build_phase_prompt(phase: Phase, opts: &TranslateOpts) -> String {
    let section = build_configurable_section(opts);
    let user_core = match phase {
        Phase::Correction => {
            if opts.prompt_correction.is_empty() { DEFAULT_CORRECTION_CORE } else { &opts.prompt_correction }
        }
        Phase::Standard => {
            if opts.prompt_standard.is_empty() { DEFAULT_STANDARD_CORE } else { &opts.prompt_standard }
        }
        Phase::Reflective => {
            if opts.prompt_reflective.is_empty() { DEFAULT_REFLECTIVE_CORE } else { &opts.prompt_reflective }
        }
        Phase::Optimize => {
            if opts.prompt_optimize.is_empty() { DEFAULT_OPTIMIZE_CORE } else { &opts.prompt_optimize }
        }
    };

    let prefix = match phase {
        Phase::Correction => String::new(),
        Phase::Standard | Phase::Reflective => {
            format!("Translate the following subtitles to {}.\n\n", opts.target_language)
        }
        Phase::Optimize => {
            format!("Target language: {}\n\n", opts.target_language)
        }
    };

    format!("{prefix}{user_core}\n{section}{JSON_RULES}")
}

// ── API Call ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

async fn call_chat_api(
    client: &reqwest::Client,
    cfg: &ResolvedConfig,
    system_prompt: &str,
    user_content: &str,
    temperature: f64,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": cfg.model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_content },
        ],
        "temperature": temperature,
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", cfg.api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP请求失败: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("API返回 {status}: {text}"));
    }

    let chat: ChatResponse = resp.json().await.map_err(|e| format!("解析响应失败: {e}"))?;
    chat.choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| "API返回空choices".to_string())
}

// ── JSON Parse & Validate ────────────────────────────────────────────────────

/// Repair common LLM JSON output issues before parsing.
fn repair_json(s: &str) -> String {
    let mut out = s.to_string();
    // Replace Chinese/curly quotes with ASCII quotes
    out = out
        .replace('\u{201C}', "\"")
        .replace('\u{201D}', "\"")
        .replace('\u{2018}', "'")
        .replace('\u{2019}', "'");
    // Remove trailing commas before } or ]
    let mut result = String::with_capacity(out.len());
    let chars: Vec<char> = out.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == ',' {
            // Peek ahead (skip whitespace) for } or ]
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                i += 1;
                continue; // skip trailing comma
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Check map content quality beyond count matching.
fn validate_content(map: &HashMap<String, String>) -> bool {
    // Reject empty/whitespace translations
    if map.values().any(|v| v.trim().is_empty()) {
        return false;
    }
    // Reject LLM merge-annotation patterns
    const MERGE_PATTERNS: &[&str] = &["已合并", "已并入", "同上", "（见第", "(见第", "合并至"];
    if map
        .values()
        .any(|v| MERGE_PATTERNS.iter().any(|p| v.contains(p)))
    {
        return false;
    }
    true
}

fn try_parse_map(s: &str, expected_count: usize) -> Result<HashMap<String, String>, String> {
    if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(s) {
        if map.len() != expected_count {
            return Err(format!(
                "条目数不匹配: 期望{expected_count}, 实际{}",
                map.len()
            ));
        }
        if !validate_content(&map) {
            return Err("响应内容验证失败（空值或合并标注）".to_string());
        }
        return Ok(map);
    }
    // Try JSON repair
    let repaired = repair_json(s);
    if repaired != s {
        if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&repaired) {
            if map.len() != expected_count {
                return Err(format!(
                    "条目数不匹配: 期望{expected_count}, 实际{}",
                    map.len()
                ));
            }
            if !validate_content(&map) {
                return Err("响应内容验证失败（空值或合并标注）".to_string());
            }
            return Ok(map);
        }
    }
    Err(format!("JSON解析失败: {}", &s[..s.len().min(200)]))
}

fn parse_and_validate(
    raw: &str,
    expected_count: usize,
) -> Result<HashMap<String, String>, String> {
    // Try direct parse first
    if let Ok(map) = try_parse_map(raw, expected_count) {
        return Ok(map);
    }
    // Fallback: extract JSON object between first { and last }
    if let (Some(start), Some(end)) = (raw.find('{'), raw.rfind('}')) {
        let slice = &raw[start..=end];
        return try_parse_map(slice, expected_count);
    }
    Err(format!("无法解析JSON: {}", &raw[..raw.len().min(200)]))
}

// ── Retry with batch splitting ───────────────────────────────────────────────

fn build_user_content(items: &[(usize, &str)]) -> String {
    let map: HashMap<String, &str> = items.iter().map(|(i, t)| (i.to_string(), *t)).collect();
    serde_json::to_string(&map).unwrap_or_default()
}

async fn call_with_retry(
    client: &reqwest::Client,
    cfg: &ResolvedConfig,
    system_prompt: &str,
    items: &[(usize, &str)],
    temperature: f64,
) -> Result<HashMap<String, String>, String> {
    // Phase 1: retry full batch up to 3 times
    for attempt in 0..3u32 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(1 << attempt)).await;
        }
        let content = build_user_content(items);
        match call_chat_api(client, cfg, system_prompt, &content, temperature).await {
            Ok(raw) => match parse_and_validate(&raw, items.len()) {
                Ok(map) => return Ok(map),
                Err(_) => {}
            },
            Err(_) => {}
        }
    }

    // Phase 2: split in half, retry each half (max 3 splits)
    if items.len() > 1 {
        let mut depth = 0;
        let mut chunks: Vec<&[(usize, &str)]> = vec![items];
        while depth < 3 {
            let mut next_chunks = Vec::new();
            let mut all_ok = true;
            for chunk in &chunks {
                if chunk.len() > 1 {
                    let mid = chunk.len() / 2;
                    next_chunks.push(&chunk[..mid]);
                    next_chunks.push(&chunk[mid..]);
                    all_ok = false;
                } else {
                    next_chunks.push(chunk);
                }
            }
            chunks = next_chunks;
            if all_ok {
                break;
            }
            depth += 1;

            // Try each sub-chunk
            let mut combined = HashMap::new();
            let mut failed = false;
            for chunk in &chunks {
                let content = build_user_content(chunk);
                let mut ok = false;
                for retry in 0..3u32 {
                    if retry > 0 {
                        tokio::time::sleep(std::time::Duration::from_secs(1 << retry)).await;
                    }
                    if let Ok(raw) = call_chat_api(client, cfg, system_prompt, &content, temperature).await {
                        if let Ok(map) = parse_and_validate(&raw, chunk.len()) {
                            combined.extend(map);
                            ok = true;
                            break;
                        }
                    }
                }
                if !ok {
                    failed = true;
                    break;
                }
            }
            if !failed {
                return Ok(combined);
            }
        }
    }

    // Phase 3: single-item fallback — translate one by one, keep original on failure
    let mut fallback: HashMap<String, String> = HashMap::new();
    for (idx, text) in items {
        let content = build_user_content(&[(*idx, *text)]);
        let mut translated = false;
        for retry in 0..2u32 {
            if retry > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            if let Ok(raw) = call_chat_api(client, cfg, system_prompt, &content, temperature).await {
                if let Ok(map) = parse_and_validate(&raw, 1) {
                    if let Some(result) = map.into_values().next() {
                        fallback.insert(idx.to_string(), result);
                        translated = true;
                        break;
                    }
                }
            }
        }
        if !translated {
            // Keep original text — do not fail the whole batch
            fallback.insert(idx.to_string(), text.to_string());
        }
    }
    Ok(fallback)
}

// ── Batch Processing ─────────────────────────────────────────────────────────

async fn process_batches(
    app: &AppHandle,
    db: &DbState,
    pool: &AiPoolManager,
    client: &reqwest::Client,
    cfg: &ResolvedConfig,
    cancel: &Arc<AtomicBool>,
    system_prompt: &str,
    texts: &HashMap<usize, String>,
    project_dir: &str,
    phase: &str,
    phase_label: &str,
    phase_base_percent: f64,
    phase_weight: f64,
    batch_size: usize,
    temperature: f64,
) -> Result<HashMap<usize, String>, String> {
    // Load existing progress for resume
    let existing = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        queries::get_translation_progress(&conn, project_dir, phase).map_err(|e| e.to_string())?
    };

    // Filter out already-done items
    let mut todo: Vec<(usize, String)> = texts
        .iter()
        .filter(|(idx, _)| !existing.contains_key(&(**idx as i32)))
        .map(|(idx, text)| (*idx, text.clone()))
        .collect();
    todo.sort_by_key(|(idx, _)| *idx);

    let skipped = existing.len() as u32;
    let mut results: HashMap<usize, String> = existing
        .into_iter()
        .map(|(k, v)| (k as usize, v))
        .collect();

    if todo.is_empty() {
        let _ = app.emit(
            "translate:progress",
            TranslateProgress {
                phase: phase_label.to_string(),
                batch: 0,
                total_batches: 0,
                skipped,
                percent: phase_base_percent + phase_weight,
                message: format!("{phase_label}: 全部已完成（断点续传）"),
            },
        );
        return Ok(results);
    }

    let batches: Vec<Vec<(usize, String)>> = todo.chunks(batch_size).map(|c| c.to_vec()).collect();
    let total_batches = batches.len() as u32;

    for (batch_idx, batch) in batches.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err("已取消".to_string());
        }

        // Acquire pool permit
        let _permit = pool
            .acquire(&cfg.id, cfg.concurrent_limit, cfg.rate_limit, cancel)
            .await?;

        // Build items slice for API call
        let items: Vec<(usize, &str)> = batch.iter().map(|(i, t)| (*i, t.as_str())).collect();
        let mut map = call_with_retry(client, cfg, system_prompt, &items, temperature).await?;

        // Retry any missing keys individually (silent fallback prevention)
        let missing: Vec<(usize, &str)> = items
            .iter()
            .filter(|(i, _)| !map.contains_key(&i.to_string()))
            .copied()
            .collect();
        for (idx, text) in &missing {
            let content = build_user_content(&[(*idx, *text)]);
            let mut recovered = false;
            for retry in 0..2u32 {
                if retry > 0 {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                if let Ok(raw) = call_chat_api(client, cfg, system_prompt, &content, temperature).await {
                    if let Ok(m) = parse_and_validate(&raw, 1) {
                        if let Some(result) = m.into_values().next() {
                            map.insert(idx.to_string(), result);
                            recovered = true;
                            break;
                        }
                    }
                }
            }
            if !recovered {
                map.insert(idx.to_string(), text.to_string());
            }
        }

        // Save results to DB and collect
        {
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            for (idx, text) in &items {
                let key = idx.to_string();
                if let Some(result_text) = map.get(&key) {
                    queries::save_translation_progress(
                        &conn,
                        project_dir,
                        *idx as i32,
                        phase,
                        result_text,
                    )
                    .map_err(|e| e.to_string())?;
                    results.insert(*idx, result_text.clone());
                } else {
                    // Fallback: keep original text
                    results.insert(*idx, text.to_string());
                }
            }
        }

        // Emit progress
        let batch_num = batch_idx as u32 + 1;
        let percent =
            phase_base_percent + phase_weight * (batch_num as f64 / total_batches as f64);
        let _ = app.emit(
            "translate:progress",
            TranslateProgress {
                phase: phase_label.to_string(),
                batch: batch_num,
                total_batches,
                skipped,
                percent,
                message: format!("{phase_label}: {batch_num}/{total_batches}"),
            },
        );
    }

    Ok(results)
}

// ── Language Detection ───────────────────────────────────────────────────────

/// Detect dominant script of a text using Unicode block heuristics.
/// Returns a rough "language family" tag for consistency checking.
fn detect_script(text: &str) -> &'static str {
    let mut cjk = 0usize;
    let mut latin = 0usize;
    let mut arabic = 0usize;
    let mut cyrillic = 0usize;
    for c in text.chars() {
        let cp = c as u32;
        if (0x4E00..=0x9FFF).contains(&cp)
            || (0x3040..=0x30FF).contains(&cp)
            || (0xAC00..=0xD7AF).contains(&cp)
        {
            cjk += 1;
        } else if (0x0041..=0x007A).contains(&cp) {
            latin += 1;
        } else if (0x0600..=0x06FF).contains(&cp) {
            arabic += 1;
        } else if (0x0400..=0x04FF).contains(&cp) {
            cyrillic += 1;
        }
    }
    let max = cjk.max(latin).max(arabic).max(cyrillic);
    if max == 0 {
        return "unknown";
    }
    if max == cjk { "cjk" }
    else if max == latin { "latin" }
    else if max == arabic { "arabic" }
    else { "cyrillic" }
}

fn dominant_script(map: &HashMap<usize, String>) -> &'static str {
    let sample: String = map.values().take(10).cloned().collect::<Vec<_>>().join(" ");
    detect_script(&sample)
}

// ── Pipeline ─────────────────────────────────────────────────────────────────

async fn run_pipeline(
    app: &AppHandle,
    db: &DbState,
    pool: &AiPoolManager,
    cancel: &Arc<AtomicBool>,
    subtitles: &[SubtitleItem],
    project_dir: &str,
    opts: &TranslateOpts,
    cfg: &ResolvedConfig,
) -> Result<Vec<SubtitleItem>, String> {
    let client = pool
        .get_or_create_client(&cfg.id, cfg.request_timeout)
        .await;

    // Count enabled phases for percent distribution
    let phase_count =
        opts.correction as u32 + 1 /* translation always */ + opts.optimization as u32;
    let phase_weight = 100.0 / phase_count as f64;
    let mut phase_idx = 0u32;

    // Current texts (evolves through phases)
    let mut current: HashMap<usize, String> = subtitles
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.text.clone()))
        .collect();

    // Phase 1: Correction (conservative temperature)
    if opts.correction {
        let prompt = build_phase_prompt(Phase::Correction, opts);
        let base = phase_idx as f64 * phase_weight;
        current = process_batches(
            app, db, pool, &client, cfg, cancel, &prompt, &current, project_dir,
            "correction", "校正", base, phase_weight, opts.batch_size, 0.1,
        )
        .await?;
        phase_idx += 1;
    }

    // Phase 2: Translation (standard temperature)
    {
        let phase = if opts.prompt_type == "reflective" {
            Phase::Reflective
        } else {
            Phase::Standard
        };
        let prompt = build_phase_prompt(phase, opts);
        let base = phase_idx as f64 * phase_weight;
        current = process_batches(
            app, db, pool, &client, cfg, cancel, &prompt, &current, project_dir,
            "translation", "翻译", base, phase_weight, opts.batch_size, 0.3,
        )
        .await?;
        phase_idx += 1;
    }

    // Phase 3: Optimization (creative temperature)
    if opts.optimization {
        let translation_script = dominant_script(&current);
        let prompt = build_phase_prompt(Phase::Optimize, opts);
        let base = phase_idx as f64 * phase_weight;
        let optimized = process_batches(
            app, db, pool, &client, cfg, cancel, &prompt, &current, project_dir,
            "optimization", "优化", base, phase_weight, opts.batch_size, 0.5,
        )
        .await?;

        // Language consistency guard: revert to translation result if script changed
        let optimized_script = dominant_script(&optimized);
        if optimized_script != "unknown"
            && translation_script != "unknown"
            && optimized_script != translation_script
        {
            // Script mismatch detected — discard optimization, keep translation phase results
        } else {
            current = optimized;
        }
    }

    // Assemble result
    let result: Vec<SubtitleItem> = subtitles
        .iter()
        .enumerate()
        .map(|(i, s)| SubtitleItem {
            id: s.id,
            start_time: s.start_time,
            end_time: s.end_time,
            text: current.remove(&i).unwrap_or_else(|| s.text.clone()),
        })
        .collect();

    Ok(result)
}

// ── Tauri Commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn cmd_start_translation(
    app: AppHandle,
    db: State<'_, DbState>,
    pool: State<'_, AiPoolManager>,
    cancel: State<'_, TranslateCancelState>,
    subtitles: Vec<SubtitleItem>,
    project_dir: String,
    target_language: String,
    correction: bool,
    optimization: bool,
    prompt_type: String,
    batch_size: u32,
    world_building: String,
    writing_style: String,
    glossary: String,
    forbidden: String,
    examples: String,
    custom_prompt: String,
    prompt_correction: String,
    prompt_standard: String,
    prompt_reflective: String,
    prompt_optimize: String,
) -> Result<Vec<SubtitleItem>, String> {
    // Reset cancel flag
    cancel.0.store(false, Ordering::Relaxed);

    // Get default AI config
    let cfg = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let configs = queries::get_all_ai_configs(&conn).map_err(|e| e.to_string())?;
        configs
            .into_iter()
            .find(|c| c.is_default)
            .ok_or_else(|| "未配置默认 AI 模型，请先在设置中添加".to_string())?
    };

    let resolved = ResolvedConfig {
        id: cfg.id,
        base_url: cfg.base_url,
        api_key: cfg.api_key,
        model: cfg.model,
        concurrent_limit: cfg.concurrent_limit as u32,
        request_timeout: cfg.request_timeout as u64,
        rate_limit: cfg.rate_limit as u32,
    };

    let opts = TranslateOpts {
        target_language,
        correction,
        optimization,
        prompt_type,
        batch_size: batch_size as usize,
        world_building,
        writing_style,
        glossary,
        forbidden,
        examples,
        custom_prompt,
        prompt_correction,
        prompt_standard,
        prompt_reflective,
        prompt_optimize,
    };

    run_pipeline(&app, &db, &pool, &cancel.0, &subtitles, &project_dir, &opts, &resolved).await
}

#[tauri::command]
pub async fn cmd_cancel_translation(
    cancel: State<'_, TranslateCancelState>,
) -> Result<(), String> {
    cancel.0.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn cmd_clear_translation_progress(
    db: State<'_, DbState>,
    project_dir: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    queries::clear_translation_progress(&conn, &project_dir).map_err(|e| e.to_string())
}
