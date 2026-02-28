use super::run_ffmpeg_async;

/// Combine a silent video with a dubbed audio track into the final output file.
pub async fn compose_video(
    silent_video_path: &str,
    dubbed_audio_path: &str,
    output_path: &str,
) -> Result<(), String> {
    run_ffmpeg_async(vec![
        "-y".to_string(),
        "-i".to_string(), silent_video_path.to_string(),
        "-i".to_string(), dubbed_audio_path.to_string(),
        "-map".to_string(), "0:v:0".to_string(),
        "-map".to_string(), "1:a:0".to_string(),
        "-c:v".to_string(), "copy".to_string(),
        "-c:a".to_string(), "aac".to_string(),
        "-b:a".to_string(), "192k".to_string(),
        "-shortest".to_string(),
        output_path.to_string(),
    ]).await
}
