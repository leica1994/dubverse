use super::{TtsSynthRequest, TtsSynthResponse, TtsVoice};
use chrono::Local;
use md5;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36";
const DOMAIN: &str = "https://bot.n.cn";
const HASH_MASK_1: u64 = 0x0FFFFFFF;
const HASH_MASK_2: u64 = 0x0FE00000;

pub struct NcnProvider {
    pub voice_id: String,
}

impl NcnProvider {
    pub fn new(voice_id: String) -> Self {
        NcnProvider { voice_id }
    }
}

// Port of Python _e() hash function
fn hash_e(input: &str) -> u64 {
    let chars: Vec<char> = input.chars().collect();
    let mut at: u64 = 0;
    for i in (0..chars.len()).rev() {
        let st = chars[i] as u64;
        at = (at << 6).wrapping_add(st).wrapping_add(st << 14) & HASH_MASK_1;
        let it = at & HASH_MASK_2;
        if it != 0 {
            at ^= it >> 21;
        }
    }
    at
}

fn generate_unique_hash() -> u64 {
    let lang = "zh-CN";
    let app_name = "chrome";
    let ver = 1.0_f64;
    let platform = "Win32";
    let width = 1920;
    let height = 1080;
    let color_depth = 24;
    let referrer = "https://bot.n.cn/chat";
    let mut nt = format!(
        "{}{}{}{}{}{}x{}{}{}",
        app_name, ver, lang, platform, UA, width, height, color_depth, referrer
    );
    let at = nt.len();
    nt.push_str(&(1_usize ^ at).to_string());
    let mut rng = rand::thread_rng();
    let random_val: u64 = rng.gen_range(0..2147483647);
    (random_val ^ hash_e(&nt)).wrapping_mul(2147483647)
}

fn generate_mid() -> String {
    let domain_hash = hash_e(DOMAIN);
    let unique_hash = generate_unique_hash();
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as f64;
    let mut rng = rand::thread_rng();
    let r1: f64 = rng.gen::<f64>();
    let r2: f64 = rng.gen::<f64>();
    let rt = format!("{}{}{}", domain_hash, unique_hash, now_ms + r1 + r2);
    let formatted = rt.replace('.', "e");
    formatted.chars().take(32).collect()
}

fn md5_hex(input: &str) -> String {
    format!("{:x}", md5::compute(input.as_bytes()))
}

fn build_auth_headers() -> Vec<(String, String)> {
    let device = "Web";
    let ver = "1.2";
    let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S+08:00").to_string();
    let access_token = generate_mid();
    let zm_ua = md5_hex(UA);
    let zm_token_str = format!("{}{}{}{}{}", device, timestamp, ver, access_token, zm_ua);
    let zm_token = md5_hex(&zm_token_str);
    vec![
        ("device-platform".to_string(), device.to_string()),
        ("timestamp".to_string(), timestamp),
        ("access-token".to_string(), access_token),
        ("zm-token".to_string(), zm_token),
        ("zm-ver".to_string(), ver.to_string()),
        ("zm-ua".to_string(), zm_ua),
        ("User-Agent".to_string(), UA.to_string()),
    ]
}

#[derive(serde::Deserialize)]
struct PlatformResponse {
    data: PlatformData,
}

#[derive(serde::Deserialize)]
struct PlatformData {
    list: Vec<PlatformItem>,
}

#[derive(serde::Deserialize)]
struct PlatformItem {
    tag: String,
    title: String,
}

impl NcnProvider {
    pub async fn list_voices(&self) -> Result<Vec<TtsVoice>, String> {
        let headers = build_auth_headers();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;
        let mut req = client.get("https://bot.n.cn/api/robot/platform");
        for (k, v) in &headers {
            req = req.header(k.as_str(), v.as_str());
        }
        let resp = req.send().await.map_err(|e| format!("请求失败: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("API错误: {}", resp.status()));
        }
        let data: PlatformResponse = resp.json().await.map_err(|e| format!("解析失败: {e}"))?;
        Ok(data.data.list.into_iter().map(|item| TtsVoice {
            id: item.tag,
            name: item.title,
            description: None,
        }).collect())
    }

    pub async fn synthesize(&self, req: TtsSynthRequest) -> Result<TtsSynthResponse, String> {
        let text = if req.text.chars().count() > 5000 {
            req.text.chars().take(5000).collect::<String>()
        } else {
            req.text.clone()
        };
        let voice = req.voice_id.as_deref().unwrap_or(&self.voice_id);
        if voice.is_empty() {
            return Err("未指定声音 ID".to_string());
        }
        let headers = build_auth_headers();
        let url = format!("https://bot.n.cn/api/tts/v1?roleid={}", voice);
        let encoded_text = urlencoding::encode(&text);
        let body = format!("&text={}&audio_type=mp3&format=stream", encoded_text);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| e.to_string())?;
        let mut r = client.post(&url);
        for (k, v) in &headers {
            r = r.header(k.as_str(), v.as_str());
        }
        r = r.header("Content-Type", "application/x-www-form-urlencoded");
        let resp = r.body(body).send().await.map_err(|e| format!("请求失败: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("TTS合成失败: {}", resp.status()));
        }
        let bytes = resp.bytes().await.map_err(|e| format!("读取响应失败: {e}"))?;
        if bytes.is_empty() {
            return Err("返回的音频数据为空".to_string());
        }
        // Write to file
        std::fs::write(&req.output_path, &bytes)
            .map_err(|e| format!("写入音频文件失败: {e}"))?;
        let duration_ms = super::get_audio_duration_ms(&req.output_path).unwrap_or(0);
        Ok(TtsSynthResponse {
            audio_path: req.output_path,
            duration_ms,
        })
    }
}
