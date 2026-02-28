pub mod separator;
pub mod reference;
pub mod aligner;
pub mod composer;

/// Run an FFmpeg command synchronously (blocking).
/// Returns Err with stderr if the command fails.
pub fn run_ffmpeg(args: &[&str]) -> Result<(), String> {
    let output = std::process::Command::new("ffmpeg")
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "ffmpeg 未安装或未加入系统 PATH，请先安装 ffmpeg".to_string()
            } else {
                format!("启动 ffmpeg 失败: {e}")
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg 错误: {stderr}"));
    }
    Ok(())
}

/// Run an FFmpeg command asynchronously via spawn_blocking.
pub async fn run_ffmpeg_async(args: Vec<String>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        run_ffmpeg(&arg_refs)
    })
    .await
    .map_err(|e| e.to_string())?
}
