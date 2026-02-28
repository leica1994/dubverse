use super::{TtsSynthRequest, TtsSynthResponse, TtsVoice};
use base64::{Engine as _, engine::general_purpose::STANDARD};

pub struct GradioProvider {
    pub endpoint: String,
}

impl GradioProvider {
    pub fn new(endpoint: String) -> Self {
        GradioProvider { endpoint }
    }

    pub async fn list_voices(&self) -> Result<Vec<TtsVoice>, String> {
        // Gradio voice cloning doesn't have a discrete voice list
        Ok(vec![])
    }

    pub async fn synthesize(&self, req: TtsSynthRequest) -> Result<TtsSynthResponse, String> {
        let reference_path = req.reference_audio_path
            .ok_or_else(|| "Gradio TTS 需要参考音频文件".to_string())?;

        // Read reference audio and encode as base64
        let ref_bytes = std::fs::read(&reference_path)
            .map_err(|e| format!("读取参考音频失败: {e}"))?;
        let ref_base64 = STANDARD.encode(&ref_bytes);

        // Determine MIME type from extension
        let ref_ext = std::path::Path::new(&reference_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("wav");
        let ref_data_url = format!("data:audio/{};base64,{}", ref_ext, ref_base64);

        // Call Gradio API predict endpoint
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| e.to_string())?;

        let endpoint = self.endpoint.trim_end_matches('/');
        let url = format!("{}/run/predict", endpoint);
        let body = serde_json::json!({
            "data": [req.text, ref_data_url]
        });

        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Gradio 请求失败: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Gradio API 错误 {status}: {text}"));
        }

        let json: serde_json::Value = resp.json().await
            .map_err(|e| format!("Gradio 响应解析失败: {e}"))?;

        // Extract audio from response data
        let data = json["data"].as_array()
            .ok_or_else(|| "Gradio 响应缺少 data 字段".to_string())?;

        let audio_value = data.first()
            .ok_or_else(|| "Gradio 响应 data 为空".to_string())?;

        let audio_bytes = if let Some(s) = audio_value.as_str() {
            // Could be base64 or a data URL
            let base64_str = if s.starts_with("data:") {
                s.splitn(2, ',').nth(1).unwrap_or(s)
            } else {
                s
            };
            STANDARD.decode(base64_str)
                .map_err(|e| format!("Gradio base64 解码失败: {e}"))?
        } else if let Some(obj) = audio_value.as_object() {
            // Some Gradio versions return { "name": "...", "data": "..." }
            if let Some(d) = obj.get("data").and_then(|v| v.as_str()) {
                STANDARD.decode(d).map_err(|e| format!("解码失败: {e}"))?
            } else {
                return Err("Gradio 响应格式不支持".to_string());
            }
        } else {
            return Err("Gradio 响应格式不支持".to_string());
        };

        std::fs::write(&req.output_path, &audio_bytes)
            .map_err(|e| format!("写入音频失败: {e}"))?;

        let duration_ms = super::get_audio_duration_ms(&req.output_path).unwrap_or(0);
        Ok(TtsSynthResponse {
            audio_path: req.output_path,
            duration_ms,
        })
    }
}
