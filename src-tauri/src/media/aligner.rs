use super::run_ffmpeg_async;

pub struct AlignedSegment {
    #[allow(dead_code)]
    pub subtitle_index: i32,
    pub start_ms: i64,
    pub tts_audio_path: String,
    pub tts_duration_ms: i64,
}

/// Align TTS audio segments and concatenate into a full dubbed audio track.
///
/// Algorithm (for each segment):
/// - If tts_dur <= subtitle_dur: pad with silence to fill the slot
/// - If tts_dur > subtitle_dur but <= gap to next: allow natural extension
/// - If tts_dur > available space: trim to min(tts_dur, subtitle_dur * 1.3)
///
/// Fills gaps between segments with silence from the original audio start.
pub async fn align_and_concat(
    segments: &[AlignedSegment],
    total_duration_ms: i64,
    work_dir: &str,
) -> Result<String, String> {
    if segments.is_empty() {
        return Err("没有 TTS 音频片段".to_string());
    }

    let concat_dir = format!("{}/aligned", work_dir);
    std::fs::create_dir_all(&concat_dir)
        .map_err(|e| format!("创建对齐目录失败: {e}"))?;

    let mut concat_list_entries: Vec<String> = Vec::new();
    let mut cursor_ms: i64 = 0;

    for (i, seg) in segments.iter().enumerate() {
        // Fill gap before this segment with silence
        let gap_before = seg.start_ms - cursor_ms;
        if gap_before > 10 {
            let silence_path = format!("{}/silence_{:04}.wav", concat_dir, i);
            generate_silence(gap_before as f64 / 1000.0, &silence_path).await?;
            concat_list_entries.push(format!("file '{}'", silence_path.replace('\\', "/")));
        }

        // Determine next segment start for overflow check
        let next_start = if i + 1 < segments.len() {
            segments[i + 1].start_ms
        } else {
            total_duration_ms
        };
        let available_ms = next_start - seg.start_ms;
        let subtitle_dur_ms = (i + 1 < segments.len())
            .then(|| segments[i + 1].start_ms - seg.start_ms)
            .unwrap_or(seg.tts_duration_ms);

        let aligned_path = format!("{}/aligned_{:04}.wav", concat_dir, i);

        if seg.tts_duration_ms <= available_ms {
            // Case 1 & 2: fits in available space, optionally pad silence at end
            if seg.tts_duration_ms < subtitle_dur_ms {
                let pad_ms = subtitle_dur_ms - seg.tts_duration_ms;
                pad_audio_with_silence(
                    &seg.tts_audio_path,
                    pad_ms as f64 / 1000.0,
                    &aligned_path,
                ).await?;
            } else {
                // Natural extension (no padding needed)
                std::fs::copy(&seg.tts_audio_path, &aligned_path)
                    .map_err(|e| format!("复制 TTS 音频失败: {e}"))?;
            }
            cursor_ms = seg.start_ms + seg.tts_duration_ms.max(subtitle_dur_ms);
        } else {
            // Case 3: trim to max allowed (subtitle_dur * 1.3 or available)
            let max_ms = (subtitle_dur_ms as f64 * 1.3).min(available_ms as f64) as i64;
            trim_audio(&seg.tts_audio_path, max_ms as f64 / 1000.0, &aligned_path).await?;
            cursor_ms = seg.start_ms + max_ms;
        }

        concat_list_entries.push(format!("file '{}'", aligned_path.replace('\\', "/")));
    }

    // Fill remaining silence at the end
    if cursor_ms < total_duration_ms {
        let tail_ms = total_duration_ms - cursor_ms;
        let tail_path = format!("{}/tail_silence.wav", concat_dir);
        generate_silence(tail_ms as f64 / 1000.0, &tail_path).await?;
        concat_list_entries.push(format!("file '{}'", tail_path.replace('\\', "/")));
    }

    // Write concat list file
    let list_path = format!("{}/concat_list.txt", work_dir);
    std::fs::write(&list_path, concat_list_entries.join("\n"))
        .map_err(|e| format!("写入 concat 列表失败: {e}"))?;

    // Run ffmpeg concat
    let output_path = format!("{}/dubbed_audio.wav", work_dir);
    run_ffmpeg_async(vec![
        "-y".to_string(),
        "-f".to_string(), "concat".to_string(),
        "-safe".to_string(), "0".to_string(),
        "-i".to_string(), list_path,
        "-acodec".to_string(), "pcm_s16le".to_string(),
        "-ar".to_string(), "44100".to_string(),
        output_path.clone(),
    ]).await?;

    Ok(output_path)
}

async fn generate_silence(duration_secs: f64, output_path: &str) -> Result<(), String> {
    run_ffmpeg_async(vec![
        "-y".to_string(),
        "-f".to_string(), "lavfi".to_string(),
        "-i".to_string(), format!("anullsrc=r=44100:cl=mono:d={:.3}", duration_secs),
        "-acodec".to_string(), "pcm_s16le".to_string(),
        output_path.to_string(),
    ]).await
}

async fn pad_audio_with_silence(
    audio_path: &str,
    pad_secs: f64,
    output_path: &str,
) -> Result<(), String> {
    run_ffmpeg_async(vec![
        "-y".to_string(),
        "-i".to_string(), audio_path.to_string(),
        "-af".to_string(), format!("apad=pad_dur={:.3}", pad_secs),
        "-acodec".to_string(), "pcm_s16le".to_string(),
        output_path.to_string(),
    ]).await
}

async fn trim_audio(
    audio_path: &str,
    max_secs: f64,
    output_path: &str,
) -> Result<(), String> {
    run_ffmpeg_async(vec![
        "-y".to_string(),
        "-i".to_string(), audio_path.to_string(),
        "-t".to_string(), format!("{:.3}", max_secs),
        "-acodec".to_string(), "pcm_s16le".to_string(),
        output_path.to_string(),
    ]).await
}
