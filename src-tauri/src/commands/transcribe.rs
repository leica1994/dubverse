use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::State;

// Subtitle item — serialized as camelCase to match the frontend TypeScript interface.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleItem {
    pub id: u32,
    pub start_time: f64, // seconds
    pub end_time: f64,   // seconds
    pub text: String,
}

// ─── 1. Create project directory ────────────────────────────────────────────────

/// Returned by `cmd_create_project_dir`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDirs {
    /// `{data_dir}/projects/{stem}_{ms}/` — subtitle outputs live here.
    pub project_dir: String,
    /// `{data_dir}/cache/{stem}_{ms}/` — temporary audio file lives here, auto-deleted after save.
    pub cache_dir: String,
}

/// Creates both the project dir and cache dir for a transcription run.
#[tauri::command]
pub async fn cmd_create_project_dir(
    data_dir: State<'_, crate::DataDirState>,
    video_stem: String,
) -> Result<ProjectDirs, String> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    let dir_name = format!("{}_{}", sanitize_stem(&video_stem), ts);
    let project_dir = data_dir.0.join("projects").join(&dir_name);
    let cache_dir = data_dir.0.join("cache").join(&dir_name);
    std::fs::create_dir_all(&project_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    Ok(ProjectDirs {
        project_dir: project_dir.to_string_lossy().into_owned(),
        cache_dir: cache_dir.to_string_lossy().into_owned(),
    })
}

fn sanitize_stem(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

// ─── 2. Extract audio via ffmpeg ────────────────────────────────────────────────

/// Runs `ffmpeg -y -i {video_path} -vn -acodec libmp3lame -ar 16000 -ac 1 -q:a 4 {output_path}`.
#[tauri::command]
pub async fn cmd_extract_audio(
    video_path: String,
    output_path: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let output = std::process::Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                &video_path,
                "-vn",
                "-acodec",
                "libmp3lame",
                "-ar",
                "16000",
                "-ac",
                "1",
                "-q:a",
                "4",
                &output_path,
            ])
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    "ffmpeg 未安装或未加入系统 PATH，请先安装 ffmpeg".to_string()
                } else {
                    format!("启动 ffmpeg 失败: {}", e)
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ffmpeg 返回错误: {}", stderr));
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

// ─── 3. Transcribe via ElevenLabs STT ───────────────────────────────────────────

/// Calls ElevenLabs `/v1/speech-to-text` (multipart).
/// - `api_key` empty → unauthenticated free tier (requires `allow_unauthenticated=1` + browser headers)
/// - `api_key` set   → authenticated paid tier (`xi-api-key` header)
/// Returns JSON-serialized `Vec<SubtitleItem>`.
#[tauri::command]
pub async fn cmd_transcribe_elevenlabs(
    audio_path: String,
    model_id: String,
    language: String,
    num_speakers: u32,
    tag_audio_events: bool,
    enable_diarization: bool,
    api_key: String,
) -> Result<String, String> {
    let bytes = tokio::task::spawn_blocking({
        let p = audio_path.clone();
        move || std::fs::read(&p)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| format!("读取音频文件失败: {}", e))?;

    let file_part = reqwest::multipart::Part::bytes(bytes)
        .file_name("audio.mp3")
        .mime_str("audio/mpeg")
        .map_err(|e| e.to_string())?;

    let mut form = reqwest::multipart::Form::new()
        .part("file", file_part)
        .text("model_id", model_id);

    if language != "auto" && !language.is_empty() {
        form = form.text("language_code", language);
    }
    if num_speakers > 0 {
        form = form.text("num_speakers", num_speakers.to_string());
    }
    form = form.text("tag_audio_events", tag_audio_events.to_string());

    let is_free = api_key.is_empty();

    if is_free {
        // Free tier: diarize must be true for unauthenticated requests
        form = form.text("diarize", "true");
    } else {
        // Paid tier: word-level timestamps (required for segmentation) + caller's diarization choice
        form = form.text("timestamps_granularity", "word");
        form = form.text("diarize", enable_diarization.to_string());
    }

    let mut builder = reqwest::Client::new()
        .post("https://api.elevenlabs.io/v1/speech-to-text")
        .multipart(form);

    if is_free {
        // Free tier: unauthenticated access via query param + browser-like headers
        builder = builder
            .query(&[("allow_unauthenticated", "1")])
            .header("Origin", "https://elevenlabs.io")
            .header("Referer", "https://elevenlabs.io/")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
            .header("Accept", "*/*")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-site");
    } else {
        // Paid tier: API key authentication
        builder = builder.header("xi-api-key", api_key);
    }

    let resp = builder
        .send()
        .await
        .map_err(|e| format!("ElevenLabs 请求失败: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("ElevenLabs API 错误 {}: {}", status, body));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let subtitles = segment_elevenlabs_words(&json)?;
    serde_json::to_string(&subtitles).map_err(|e| e.to_string())
}

/// Returns true if `c` is a CJK character (Chinese/Japanese/Korean).
fn is_cjk_char(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}'   // CJK Unified Ideographs (汉字)
        | '\u{3040}'..='\u{309F}' // Hiragana
        | '\u{30A0}'..='\u{30FF}' // Katakana
        | '\u{AC00}'..='\u{D7AF}' // Korean Hangul
    )
}

/// Joins word tokens intelligently:
/// - No space between two adjacent CJK chars (handles per-char diarize output)
/// - No space before CJK punctuation
/// - Space between non-CJK tokens (English words, numbers)
fn join_words_smart(words: &[(f64, f64, String)]) -> String {
    let mut result = String::new();
    for (i, (_, _, word)) in words.iter().enumerate() {
        if i == 0 {
            result.push_str(word);
            continue;
        }
        let prev_last_cjk = words[i - 1].2.chars().last().map_or(false, is_cjk_char);
        let curr_first_cjk = word.chars().next().map_or(false, is_cjk_char);
        let curr_is_cjk_punct = word.chars().next()
            .map_or(false, |c| "，。！？；、…·—".contains(c));
        if !(prev_last_cjk && curr_first_cjk) && !curr_is_cjk_punct {
            result.push(' ');
        }
        result.push_str(word);
    }
    result
}

/// Segments ElevenLabs word-level response into subtitle entries.
///
/// Split rules (in priority order):
///   1. Time gap > 1.0s between words → hard split before current word
///   2. Word ends with sentence-final punctuation (。！？…) → split after current word
///   3. Word ends with pause punctuation (，、；,;) AND seg >= SOFT_CHARS → split after
///   4. Adding current word would exceed MAX_CHARS → hard split before current word
fn segment_elevenlabs_words(json: &serde_json::Value) -> Result<Vec<SubtitleItem>, String> {
    const GAP_THRESHOLD: f64 = 1.0;   // seconds — hard split on silence
    const MAX_CHARS: usize = 50;       // Unicode chars — hard line limit (incl. spaces)
    const SOFT_CHARS: usize = 33;      // Unicode chars — soft limit for pause-based split (incl. spaces)

    let sentence_end: &[char] = &['。', '！', '？', '…', '.', '!', '?'];
    let sentence_pause: &[char] = &['，', '、', '；', ',', ';'];

    let words_arr = json["words"]
        .as_array()
        .ok_or_else(|| "ElevenLabs 响应缺少 words 数组".to_string())?;

    let mut subtitles: Vec<SubtitleItem> = Vec::new();
    let mut seg: Vec<(f64, f64, String)> = Vec::new();
    let mut seg_chars = 0usize; // Unicode char count
    let mut prev_end = 0.0f64;
    let mut id = 1u32;

    for w in words_arr {
        if w["type"].as_str() != Some("word") {
            continue;
        }
        let text = w["text"].as_str().unwrap_or("").to_string();
        if text.is_empty() {
            continue;
        }
        let start = w["start"].as_f64().unwrap_or(0.0);
        let end = w["end"].as_f64().unwrap_or(0.0);
        let char_count = text.chars().count();

        let gap = start - prev_end;

        // Hard split: silence gap or line would exceed MAX_CHARS
        // +1 accounts for the space join_words_smart inserts between words
        let space = if seg.is_empty() { 0 } else { 1 };
        if !seg.is_empty() && (gap > GAP_THRESHOLD || seg_chars + space + char_count > MAX_CHARS) {
            subtitles.push(flush_seg(&mut seg, &mut seg_chars, id));
            id += 1;
        }

        prev_end = end;
        let space = if seg.is_empty() { 0 } else { 1 };
        seg_chars += space + char_count;
        let ends_sentence = text.chars().last().map_or(false, |c| sentence_end.contains(&c));
        let ends_pause = text.chars().last().map_or(false, |c| sentence_pause.contains(&c));
        seg.push((start, end, text));

        // Soft split: after sentence-final or pause punctuation
        if ends_sentence || (ends_pause && seg_chars >= SOFT_CHARS) {
            subtitles.push(flush_seg(&mut seg, &mut seg_chars, id));
            id += 1;
        }
    }

    if !seg.is_empty() {
        subtitles.push(flush_seg(&mut seg, &mut seg_chars, id));
    }

    Ok(subtitles)
}

fn flush_seg(seg: &mut Vec<(f64, f64, String)>, seg_chars: &mut usize, id: u32) -> SubtitleItem {
    let start_time = seg.first().unwrap().0;
    let end_time = seg.last().unwrap().1;
    let text = join_words_smart(seg);
    seg.clear();
    *seg_chars = 0;
    SubtitleItem { id, start_time, end_time, text }
}

// ─── 4. Transcribe via bcut (bilibili) ──────────────────────────────────────────

const BCUT_BASE: &str = "https://member.bilibili.com/x/bcut/rubick-interface";

/// Full bcut pipeline (rubick-interface API):
///   申请上传 → 分片上传(收 ETag) → 提交上传(拿 download_url) → 创建任务 → 轮询结果
/// Returns JSON-serialized `Vec<SubtitleItem>`.
#[tauri::command]
pub async fn cmd_transcribe_bcut(
    audio_path: String,
    language: String,
) -> Result<String, String> {
    let _ = language; // bcut detects language automatically

    // Read audio file
    let audio_bytes = tokio::task::spawn_blocking({
        let p = audio_path.clone();
        move || std::fs::read(&p)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| format!("读取音频文件失败: {}", e))?;

    let file_size = audio_bytes.len();
    let client = reqwest::Client::builder()
        .user_agent("Bilibili/1.0.0 (https://www.bilibili.com)")
        .build()
        .map_err(|e| e.to_string())?;

    // ── Step 1: 申请上传 ────────────────────────────────────────────────────────
    let create_resp: serde_json::Value = client
        .post(format!("{}/resource/create", BCUT_BASE))
        .json(&serde_json::json!({
            "type": 2,
            "name": "audio.mp3",
            "size": file_size,
            "ResourceFileType": "mp3",
            "model_id": "8"
        }))
        .send()
        .await
        .map_err(|e| format!("bcut 申请上传失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("bcut 申请上传响应解析失败: {}", e))?;

    if create_resp["code"].as_i64() != Some(0) {
        return Err(format!("bcut 申请上传错误: {}", create_resp));
    }

    let data = &create_resp["data"];
    let in_boss_key = data["in_boss_key"].as_str()
        .ok_or("bcut 响应缺少 in_boss_key")?.to_string();
    let resource_id = data["resource_id"].as_str()
        .ok_or("bcut 响应缺少 resource_id")?.to_string();
    let upload_id = data["upload_id"].as_str()
        .ok_or("bcut 响应缺少 upload_id")?.to_string();
    let upload_urls: Vec<String> = data["upload_urls"]
        .as_array()
        .ok_or("bcut 响应缺少 upload_urls")?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let per_size = data["per_size"].as_u64().unwrap_or(file_size as u64) as usize;

    // ── Step 2: 分片上传，收集 ETag ─────────────────────────────────────────────
    let mut etags: Vec<String> = Vec::new();
    for (i, url) in upload_urls.iter().enumerate() {
        let start = i * per_size;
        let end = std::cmp::min(start + per_size, file_size);
        let chunk = audio_bytes[start..end].to_vec();

        let upload_resp = client
            .put(url)
            .body(chunk)
            .send()
            .await
            .map_err(|e| format!("bcut 上传分片 {} 失败: {}", i, e))?;

        if !upload_resp.status().is_success() {
            return Err(format!("bcut 上传分片 {} 返回 HTTP {}", i, upload_resp.status()));
        }

        let etag = upload_resp
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        etags.push(etag);
    }

    // ── Step 3: 提交上传，拿 download_url ───────────────────────────────────────
    let commit_resp: serde_json::Value = client
        .post(format!("{}/resource/create/complete", BCUT_BASE))
        .json(&serde_json::json!({
            "InBossKey": in_boss_key,
            "ResourceId": resource_id,
            "Etags": etags.join(","),
            "UploadId": upload_id,
            "model_id": "8"
        }))
        .send()
        .await
        .map_err(|e| format!("bcut 提交上传失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("bcut 提交上传响应解析失败: {}", e))?;

    if commit_resp["code"].as_i64() != Some(0) {
        return Err(format!("bcut 提交上传错误: {}", commit_resp));
    }

    let download_url = commit_resp["data"]["download_url"]
        .as_str()
        .ok_or("bcut 响应缺少 download_url")?
        .to_string();

    // ── Step 4: 创建转录任务 ─────────────────────────────────────────────────────
    let task_resp: serde_json::Value = client
        .post(format!("{}/task", BCUT_BASE))
        .json(&serde_json::json!({
            "resource": download_url,
            "model_id": "8"
        }))
        .send()
        .await
        .map_err(|e| format!("bcut 创建任务失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("bcut 创建任务响应解析失败: {}", e))?;

    if task_resp["code"].as_i64() != Some(0) {
        return Err(format!("bcut 创建任务错误: {}", task_resp));
    }

    let task_id = task_resp["data"]["task_id"]
        .as_str()
        .ok_or("bcut 响应缺少 task_id")?
        .to_string();

    // ── Step 5: 轮询结果（2s 间隔，最多 120 次 = 240s）─────────────────────────
    for _ in 0..120 {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let poll_resp: serde_json::Value = client
            .get(format!("{}/task/result", BCUT_BASE))
            .query(&[("model_id", "7"), ("task_id", &task_id)])
            .send()
            .await
            .map_err(|e| format!("bcut 轮询失败: {}", e))?
            .json()
            .await
            .map_err(|e| format!("bcut 轮询响应解析失败: {}", e))?;

        if poll_resp["code"].as_i64() != Some(0) {
            return Err(format!("bcut 轮询错误: {}", poll_resp));
        }

        let state = poll_resp["data"]["state"].as_i64().unwrap_or(0);
        match state {
            4 => {
                let subtitles = parse_bcut_result(&poll_resp["data"]["result"])?;
                return serde_json::to_string(&subtitles).map_err(|e| e.to_string());
            }
            3 => return Err("bcut 转录失败（服务端错误）".to_string()),
            _ => {} // still processing
        }
    }

    Err("bcut 转录超时（超过 240 秒）".to_string())
}

/// Parse bcut rubick-interface result: `data.result` is a JSON string containing `utterances[]`.
fn parse_bcut_result(result: &serde_json::Value) -> Result<Vec<SubtitleItem>, String> {
    let obj = if result.is_string() {
        serde_json::from_str::<serde_json::Value>(result.as_str().unwrap())
            .map_err(|e| format!("bcut result JSON 解析失败: {}", e))?
    } else {
        result.clone()
    };

    let utterances = obj["utterances"]
        .as_array()
        .ok_or_else(|| "bcut result 缺少 utterances".to_string())?;

    let mut subtitles = Vec::new();
    for (idx, u) in utterances.iter().enumerate() {
        let text = u["transcript"].as_str().unwrap_or("").to_string();
        if text.is_empty() {
            continue;
        }
        let start_ms = u["start_time"].as_u64().unwrap_or(0);
        let end_ms = u["end_time"].as_u64().unwrap_or(0);
        subtitles.push(SubtitleItem {
            id: (idx + 1) as u32,
            start_time: start_ms as f64 / 1000.0,
            end_time: end_ms as f64 / 1000.0,
            text,
        });
    }

    Ok(subtitles)
}

/// Opens a native file picker dialog and returns the selected video file's absolute path,
/// or `None` if the user cancelled.
#[tauri::command]
pub async fn cmd_pick_video_file() -> Result<Option<String>, String> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter(
            "视频文件",
            &["mp4", "mov", "avi", "mkv", "webm", "flv", "wmv", "ts", "m4v"],
        )
        .pick_file()
        .await;
    Ok(file.map(|f| f.path().to_string_lossy().into_owned()))
}

// ─── 5. Save subtitles to disk ───────────────────────────────────────────────────

/// Writes `{project_dir}/subtitles.json` and `{project_dir}/subtitles.srt`,
/// then removes `cache_dir` (the temporary audio directory).
#[tauri::command]
pub fn cmd_save_subtitles(
    project_dir: String,
    cache_dir: String,
    subtitles_json: String,
) -> Result<(), String> {
    let dir = Path::new(&project_dir);

    std::fs::write(dir.join("subtitles.json"), &subtitles_json)
        .map_err(|e| format!("写入 subtitles.json 失败: {}", e))?;

    let subtitles: Vec<SubtitleItem> = serde_json::from_str(&subtitles_json)
        .map_err(|e| format!("解析字幕 JSON 失败: {}", e))?;

    let srt = build_srt(&subtitles);
    std::fs::write(dir.join("subtitles.srt"), srt)
        .map_err(|e| format!("写入 subtitles.srt 失败: {}", e))?;

    // Clean up temporary audio cache
    let cache = Path::new(&cache_dir);
    if cache.exists() {
        std::fs::remove_dir_all(cache)
            .map_err(|e| format!("删除缓存目录失败: {}", e))?;
    }

    Ok(())
}

fn build_srt(subtitles: &[SubtitleItem]) -> String {
    let mut out = String::new();
    for sub in subtitles {
        out.push_str(&sub.id.to_string());
        out.push('\n');
        out.push_str(&srt_time(sub.start_time));
        out.push_str(" --> ");
        out.push_str(&srt_time(sub.end_time));
        out.push('\n');
        out.push_str(&sub.text);
        out.push_str("\n\n");
    }
    out
}

fn srt_time(secs: f64) -> String {
    let total_ms = (secs * 1000.0).round() as u64;
    let ms = total_ms % 1000;
    let s = (total_ms / 1000) % 60;
    let m = (total_ms / 60_000) % 60;
    let h = total_ms / 3_600_000;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, ms)
}
