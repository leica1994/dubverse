use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsVoice {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

pub struct TtsSynthRequest {
    pub text: String,
    pub voice_id: Option<String>,
    pub reference_audio_path: Option<String>,
    pub output_path: String,
}

pub struct TtsSynthResponse {
    pub audio_path: String,
    pub duration_ms: u64,
}

// Enum dispatch to avoid dyn Trait + async complexity
pub mod ncn;
pub mod gradio;
pub mod http_rest;

pub use ncn::NcnProvider;
pub use gradio::GradioProvider;
pub use http_rest::HttpRestProvider;

pub enum TtsProviderImpl {
    Ncn(NcnProvider),
    Gradio(GradioProvider),
    HttpRest(HttpRestProvider),
}

impl TtsProviderImpl {
    #[allow(dead_code)]
    pub fn plugin_type(&self) -> &str {
        match self {
            Self::Ncn(_) => "ncn",
            Self::Gradio(_) => "gradio",
            Self::HttpRest(_) => "http_rest",
        }
    }

    #[allow(dead_code)]
    pub fn requires_reference_audio(&self) -> bool {
        match self {
            Self::Ncn(_) => false,
            Self::Gradio(_) => true,
            Self::HttpRest(p) => p.requires_ref,
        }
    }

    pub async fn list_voices(&self) -> Result<Vec<TtsVoice>, String> {
        match self {
            Self::Ncn(p) => p.list_voices().await,
            Self::Gradio(p) => p.list_voices().await,
            Self::HttpRest(p) => p.list_voices().await,
        }
    }

    pub async fn synthesize(&self, req: TtsSynthRequest) -> Result<TtsSynthResponse, String> {
        match self {
            Self::Ncn(p) => p.synthesize(req).await,
            Self::Gradio(p) => p.synthesize(req).await,
            Self::HttpRest(p) => p.synthesize(req).await,
        }
    }
}

/// Get audio duration in milliseconds via ffprobe.
pub fn get_audio_duration_ms(audio_path: &str) -> Result<u64, String> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            audio_path,
        ])
        .output()
        .map_err(|e| format!("ffprobe 启动失败: {e}"))?;

    if !output.status.success() {
        // Fallback: estimate from file size at ~128kbps
        if let Ok(meta) = std::fs::metadata(audio_path) {
            let file_size = meta.len();
            return Ok((file_size * 8 * 1000) / (128 * 1024));
        }
        return Ok(0);
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("ffprobe JSON 解析失败: {e}"))?;

    let duration_str = json["format"]["duration"]
        .as_str()
        .unwrap_or("0");
    let duration_secs: f64 = duration_str.parse().unwrap_or(0.0);
    Ok((duration_secs * 1000.0).round() as u64)
}
