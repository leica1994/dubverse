use super::run_ffmpeg_async;

pub struct MediaSeparationResult {
    pub vocal_audio_path: String,
    pub silent_video_path: String,
}

/// Separate a video into:
/// - vocal_audio: WAV audio track (for reference audio extraction)
/// - silent_video: video stream without audio (for final composition)
pub async fn separate_media(
    video_path: &str,
    work_dir: &str,
) -> Result<MediaSeparationResult, String> {
    let vocal_path = format!("{}/vocals.wav", work_dir);
    let silent_path = format!("{}/silent_video.mp4", work_dir);

    // Extract audio as WAV for reference extraction
    run_ffmpeg_async(vec![
        "-y".to_string(),
        "-i".to_string(), video_path.to_string(),
        "-vn".to_string(),
        "-acodec".to_string(), "pcm_s16le".to_string(),
        "-ar".to_string(), "44100".to_string(),
        "-ac".to_string(), "1".to_string(),
        vocal_path.clone(),
    ]).await?;

    // Extract video without audio (copy codec to avoid re-encoding)
    run_ffmpeg_async(vec![
        "-y".to_string(),
        "-i".to_string(), video_path.to_string(),
        "-an".to_string(),
        "-vcodec".to_string(), "copy".to_string(),
        silent_path.clone(),
    ]).await?;

    Ok(MediaSeparationResult {
        vocal_audio_path: vocal_path,
        silent_video_path: silent_path,
    })
}
