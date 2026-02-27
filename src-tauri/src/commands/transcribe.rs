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

/// Runs `ffmpeg -y -i {video_path} -vn -acodec pcm_s16le -ar 16000 -ac 1 {output_path}`.
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
                "pcm_s16le",
                "-ar",
                "16000",
                "-ac",
                "1",
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
/// `api_key` empty → no auth header (free tier).
/// Returns JSON-serialized `Vec<SubtitleItem>`.
#[tauri::command]
pub async fn cmd_transcribe_elevenlabs(
    audio_path: String,
    model_id: String,
    language: String,
    num_speakers: u32,
    tag_audio_events: bool,
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
        .file_name("audio.wav")
        .mime_str("audio/wav")
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

    let client = reqwest::Client::new();
    let mut builder = client
        .post("https://api.elevenlabs.io/v1/speech-to-text")
        .multipart(form);

    if !api_key.is_empty() {
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

/// Segments ElevenLabs word-level response into subtitle entries.
/// Splits on gap > 1.0s between words OR accumulated char count > 60.
fn segment_elevenlabs_words(json: &serde_json::Value) -> Result<Vec<SubtitleItem>, String> {
    let words_arr = json["words"]
        .as_array()
        .ok_or_else(|| "ElevenLabs 响应缺少 words 数组".to_string())?;

    let mut subtitles: Vec<SubtitleItem> = Vec::new();
    // (start, end, word_text)
    let mut seg: Vec<(f64, f64, String)> = Vec::new();
    let mut seg_chars = 0usize;
    let mut prev_end = 0.0f64;
    let mut id = 1u32;

    for w in words_arr {
        if w["type"].as_str() != Some("word") {
            continue;
        }
        let text = w["text"].as_str().unwrap_or("").to_string();
        let start = w["start"].as_f64().unwrap_or(0.0);
        let end = w["end"].as_f64().unwrap_or(0.0);

        let gap = start - prev_end;
        if !seg.is_empty() && (gap > 1.0 || seg_chars > 60) {
            subtitles.push(flush_seg(&mut seg, &mut seg_chars, id));
            id += 1;
        }

        prev_end = end;
        seg_chars += text.len();
        seg.push((start, end, text));
    }

    if !seg.is_empty() {
        subtitles.push(flush_seg(&mut seg, &mut seg_chars, id));
    }

    Ok(subtitles)
}

fn flush_seg(seg: &mut Vec<(f64, f64, String)>, seg_chars: &mut usize, id: u32) -> SubtitleItem {
    let start_time = seg.first().unwrap().0;
    let end_time = seg.last().unwrap().1;
    let text = seg.iter().map(|(_, _, t)| t.as_str()).collect::<Vec<_>>().join(" ");
    seg.clear();
    *seg_chars = 0;
    SubtitleItem { id, start_time, end_time, text }
}

// ─── 4. Transcribe via bcut (bilibili) ──────────────────────────────────────────

/// Full bcut pipeline: create task → upload WAV → commit → poll (2s interval, 120s timeout).
/// Returns JSON-serialized `Vec<SubtitleItem>`.
#[tauri::command]
pub async fn cmd_transcribe_bcut(
    audio_path: String,
    language: String,
) -> Result<String, String> {
    let _ = language; // bcut detects language automatically

    let client = reqwest::Client::new();

    // Step 1: Create task
    let create_json: serde_json::Value = client
        .post("https://member.bilibili.com/x/bcut/whale/task")
        .json(&serde_json::json!({ "type": 2 }))
        .send()
        .await
        .map_err(|e| format!("bcut 创建任务请求失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("bcut 创建任务响应解析失败: {}", e))?;

    if create_json["code"].as_i64() != Some(0) {
        return Err(format!("bcut 创建任务错误: {}", create_json));
    }

    let upload_url = create_json["data"]["upload_url"]
        .as_str()
        .ok_or_else(|| "bcut 响应缺少 upload_url".to_string())?
        .to_string();
    let task_id = create_json["data"]["task_id"]
        .as_str()
        .ok_or_else(|| "bcut 响应缺少 task_id".to_string())?
        .to_string();

    // Step 2: Upload audio (raw bytes via PUT)
    let audio_bytes = tokio::task::spawn_blocking({
        let p = audio_path.clone();
        move || std::fs::read(&p)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| format!("读取音频文件失败: {}", e))?;

    let upload_resp = client
        .put(&upload_url)
        .body(audio_bytes)
        .send()
        .await
        .map_err(|e| format!("bcut 上传音频失败: {}", e))?;

    if !upload_resp.status().is_success() {
        return Err(format!("bcut 上传音频返回 HTTP {}", upload_resp.status()));
    }

    // Step 3: Commit task
    let commit_json: serde_json::Value = client
        .post("https://member.bilibili.com/x/bcut/whale/task/commit")
        .json(&serde_json::json!({ "task_id": task_id }))
        .send()
        .await
        .map_err(|e| format!("bcut 提交任务请求失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("bcut 提交任务响应解析失败: {}", e))?;

    if commit_json["code"].as_i64() != Some(0) {
        return Err(format!("bcut 提交任务错误: {}", commit_json));
    }

    // Step 4: Poll for result
    let poll_url = format!(
        "https://member.bilibili.com/x/bcut/whale/task/result?task_id={}",
        task_id
    );

    for _ in 0..60 {
        // up to 60 × 2s = 120s
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let poll_json: serde_json::Value = client
            .get(&poll_url)
            .send()
            .await
            .map_err(|e| format!("bcut 轮询请求失败: {}", e))?
            .json()
            .await
            .map_err(|e| format!("bcut 轮询响应解析失败: {}", e))?;

        if poll_json["code"].as_i64() != Some(0) {
            return Err(format!("bcut 轮询错误: {}", poll_json));
        }

        let state = poll_json["data"]["state"].as_i64().unwrap_or(0);
        match state {
            4 => {
                // Success
                let subtitles = parse_bcut_result(&poll_json["data"]["result"])?;
                return serde_json::to_string(&subtitles).map_err(|e| e.to_string());
            }
            3 => {
                return Err("bcut 转录失败（服务器端错误）".to_string());
            }
            _ => {
                // Still processing — keep polling
            }
        }
    }

    Err("bcut 转录超时（超过 120 秒）".to_string())
}

/// Parse bcut result value into subtitle items.
/// `result` may be a JSON object or a JSON-encoded string.
fn parse_bcut_result(result: &serde_json::Value) -> Result<Vec<SubtitleItem>, String> {
    let obj = if result.is_string() {
        serde_json::from_str::<serde_json::Value>(result.as_str().unwrap())
            .map_err(|e| format!("bcut result JSON 解析失败: {}", e))?
    } else {
        result.clone()
    };

    let lines = obj["body"]["lines"]
        .as_array()
        .ok_or_else(|| "bcut result 缺少 body.lines".to_string())?;

    let mut subtitles: Vec<SubtitleItem> = Vec::new();
    let mut id = 1u32;

    for line in lines {
        // Primary: lines[].wordsResult[].words[]
        if let Some(words_results) = line["wordsResult"].as_array() {
            for wr in words_results {
                if let Some(words) = wr["words"].as_array() {
                    if words.is_empty() {
                        continue;
                    }
                    let start_ms = parse_bcut_time(&words[0]["startTime"]);
                    let end_ms = parse_bcut_time(&words[words.len() - 1]["endTime"]);
                    let text: String = words
                        .iter()
                        .filter_map(|w| w["content"].as_str())
                        .collect();
                    if !text.is_empty() {
                        subtitles.push(SubtitleItem {
                            id,
                            start_time: start_ms as f64 / 1000.0,
                            end_time: end_ms as f64 / 1000.0,
                            text,
                        });
                        id += 1;
                    }
                }
            }
        } else if let Some(words) = line["words"].as_array() {
            // Fallback: lines[].words[]
            if words.is_empty() {
                continue;
            }
            let start_ms = parse_bcut_time(&words[0]["startTime"]);
            let end_ms = parse_bcut_time(&words[words.len() - 1]["endTime"]);
            let text: String = words.iter().filter_map(|w| w["content"].as_str()).collect();
            if !text.is_empty() {
                subtitles.push(SubtitleItem {
                    id,
                    start_time: start_ms as f64 / 1000.0,
                    end_time: end_ms as f64 / 1000.0,
                    text,
                });
                id += 1;
            }
        }
    }

    Ok(subtitles)
}

/// Extract milliseconds from a bcut time value (may be u64 or string).
fn parse_bcut_time(v: &serde_json::Value) -> u64 {
    v.as_u64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0)
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
