use super::run_ffmpeg_async;

pub struct SubtitleEntry {
    pub index: i32,
    pub start_ms: i64,
    pub end_ms: i64,
}

/// Extract reference audio clips from the vocal track for each subtitle segment.
/// For 'clone' mode: slice from the original vocal audio.
/// Returns list of (subtitle_index, reference_audio_path).
pub async fn extract_reference_clips(
    vocal_audio_path: &str,
    subtitles: &[SubtitleEntry],
    work_dir: &str,
) -> Result<Vec<(i32, String)>, String> {
    let ref_dir = format!("{}/reference", work_dir);
    std::fs::create_dir_all(&ref_dir)
        .map_err(|e| format!("创建参考音频目录失败: {e}"))?;

    let mut results = Vec::new();
    for entry in subtitles {
        let start_secs = entry.start_ms as f64 / 1000.0;
        let dur_secs = (entry.end_ms - entry.start_ms) as f64 / 1000.0;
        if dur_secs <= 0.0 {
            continue;
        }
        let out_path = format!("{}/ref_{:04}.wav", ref_dir, entry.index);
        run_ffmpeg_async(vec![
            "-y".to_string(),
            "-i".to_string(), vocal_audio_path.to_string(),
            "-ss".to_string(), format!("{:.3}", start_secs),
            "-t".to_string(), format!("{:.3}", dur_secs),
            "-acodec".to_string(), "pcm_s16le".to_string(),
            out_path.clone(),
        ]).await?;
        results.push((entry.index, out_path));
    }
    Ok(results)
}

/// Copy a user-provided custom reference audio to the work directory.
pub async fn prepare_custom_reference(
    source_path: &str,
    work_dir: &str,
) -> Result<String, String> {
    let ref_dir = format!("{}/reference", work_dir);
    std::fs::create_dir_all(&ref_dir)
        .map_err(|e| format!("创建参考音频目录失败: {e}"))?;

    let ext = std::path::Path::new(source_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("wav");
    let dest = format!("{}/custom_ref.{}", ref_dir, ext);
    std::fs::copy(source_path, &dest)
        .map_err(|e| format!("复制参考音频失败: {e}"))?;
    Ok(dest)
}
