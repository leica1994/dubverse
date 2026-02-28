use super::{TtsSynthRequest, TtsSynthResponse, TtsVoice};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::collections::HashMap;

pub struct HttpRestProvider {
    pub url: String,
    pub method: String,
    pub text_key: String,
    pub reference_audio_key: Option<String>,
    pub voice_key: Option<String>,
    pub voice_id: Option<String>,
    pub response_type: String, // "json_base64" | "binary" | "file_url"
    pub response_key: Option<String>,
    pub headers: HashMap<String, String>,
    #[allow(dead_code)]
    pub requires_ref: bool,
}

impl HttpRestProvider {
    pub fn from_json(config_json: &str, requires_ref: bool) -> Result<Self, String> {
        let v: serde_json::Value = serde_json::from_str(config_json)
            .map_err(|e| format!("解析 config_json 失败: {e}"))?;

        let headers = if let Some(h) = v["headers"].as_object() {
            h.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        } else {
            HashMap::new()
        };

        Ok(HttpRestProvider {
            url: v["url"].as_str().unwrap_or("").to_string(),
            method: v["method"].as_str().unwrap_or("POST").to_string(),
            text_key: v["textKey"].as_str().unwrap_or("text").to_string(),
            reference_audio_key: v["referenceAudioKey"].as_str().map(|s| s.to_string()),
            voice_key: v["voiceKey"].as_str().map(|s| s.to_string()),
            voice_id: v["voiceId"].as_str().map(|s| s.to_string()),
            response_type: v["responseType"].as_str().unwrap_or("json_base64").to_string(),
            response_key: v["responseKey"].as_str().map(|s| s.to_string()),
            headers,
            requires_ref,
        })
    }

    pub async fn list_voices(&self) -> Result<Vec<TtsVoice>, String> {
        Ok(vec![])
    }

    pub async fn synthesize(&self, req: TtsSynthRequest) -> Result<TtsSynthResponse, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| e.to_string())?;

        let mut body = serde_json::json!({});
        body[&self.text_key] = serde_json::Value::String(req.text.clone());

        if let Some(ref vk) = self.voice_key {
            if let Some(ref vid) = self.voice_id.as_ref().or(req.voice_id.as_ref()) {
                body[vk] = serde_json::Value::String(vid.to_string());
            }
        }

        if let (Some(ref rak), Some(ref rpath)) = (&self.reference_audio_key, &req.reference_audio_path) {
            let bytes = std::fs::read(rpath)
                .map_err(|e| format!("读取参考音频失败: {e}"))?;
            body[rak] = serde_json::Value::String(STANDARD.encode(&bytes));
        }

        let mut builder = if self.method.to_uppercase() == "GET" {
            client.get(&self.url)
        } else {
            client.post(&self.url)
        };

        for (k, v) in &self.headers {
            builder = builder.header(k.as_str(), v.as_str());
        }

        let resp = builder
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP TTS 请求失败: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("HTTP TTS API 错误 {status}: {text}"));
        }

        let audio_bytes = match self.response_type.as_str() {
            "binary" => {
                resp.bytes().await.map_err(|e| format!("读取响应失败: {e}"))?.to_vec()
            }
            "json_base64" => {
                let json: serde_json::Value = resp.json().await
                    .map_err(|e| format!("JSON 解析失败: {e}"))?;
                let key = self.response_key.as_deref().unwrap_or("audio");
                let b64 = json[key].as_str()
                    .ok_or_else(|| format!("响应缺少 '{key}' 字段"))?;
                STANDARD.decode(b64).map_err(|e| format!("base64 解码失败: {e}"))?
            }
            "file_url" => {
                let json: serde_json::Value = resp.json().await
                    .map_err(|e| format!("JSON 解析失败: {e}"))?;
                let key = self.response_key.as_deref().unwrap_or("url");
                let url = json[key].as_str()
                    .ok_or_else(|| format!("响应缺少 '{key}' 字段"))?;
                let file_resp = client.get(url).send().await
                    .map_err(|e| format!("下载音频文件失败: {e}"))?;
                file_resp.bytes().await.map_err(|e| format!("读取音频失败: {e}"))?.to_vec()
            }
            other => return Err(format!("不支持的 response_type: {other}")),
        };

        std::fs::write(&req.output_path, &audio_bytes)
            .map_err(|e| format!("写入音频文件失败: {e}"))?;

        let duration_ms = super::get_audio_duration_ms(&req.output_path).unwrap_or(0);
        Ok(TtsSynthResponse {
            audio_path: req.output_path,
            duration_ms,
        })
    }
}
