use crate::db::connection::DbState;
use crate::db::queries::{self, TtsPlugin};
use crate::tts::{NcnProvider, GradioProvider, HttpRestProvider, TtsSynthRequest, TtsVoice, TtsProviderImpl};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsPluginDto {
    pub id: String,
    pub name: String,
    pub plugin_type: String,
    pub config_json: String,
    pub requires_ref: bool,
    pub is_enabled: bool,
    pub sort_order: i32,
    pub created_at: String,
}

fn to_dto(p: TtsPlugin) -> TtsPluginDto {
    TtsPluginDto {
        id: p.id,
        name: p.name,
        plugin_type: p.plugin_type,
        config_json: p.config_json,
        requires_ref: p.requires_ref,
        is_enabled: p.is_enabled,
        sort_order: p.sort_order,
        created_at: p.created_at,
    }
}

fn from_dto(dto: &TtsPluginDto) -> TtsPlugin {
    TtsPlugin {
        id: dto.id.clone(),
        name: dto.name.clone(),
        plugin_type: dto.plugin_type.clone(),
        config_json: dto.config_json.clone(),
        requires_ref: dto.requires_ref,
        is_enabled: dto.is_enabled,
        sort_order: dto.sort_order,
        created_at: dto.created_at.clone(),
    }
}

/// Build a TtsProviderImpl from a plugin record.
pub fn build_provider(plugin: &TtsPlugin) -> Result<TtsProviderImpl, String> {
    match plugin.plugin_type.as_str() {
        "ncn" => {
            let config: serde_json::Value = serde_json::from_str(&plugin.config_json)
                .unwrap_or(serde_json::json!({}));
            let voice_id = config["voiceId"].as_str().unwrap_or("").to_string();
            Ok(TtsProviderImpl::Ncn(NcnProvider::new(voice_id)))
        }
        "gradio" => {
            let config: serde_json::Value = serde_json::from_str(&plugin.config_json)
                .unwrap_or(serde_json::json!({}));
            let endpoint = config["endpoint"].as_str()
                .ok_or_else(|| "Gradio 插件缺少 endpoint 配置".to_string())?
                .to_string();
            Ok(TtsProviderImpl::Gradio(GradioProvider::new(endpoint)))
        }
        "http_rest" => {
            let provider = HttpRestProvider::from_json(&plugin.config_json, plugin.requires_ref)?;
            Ok(TtsProviderImpl::HttpRest(provider))
        }
        other => Err(format!("未知 plugin_type: {other}")),
    }
}

#[tauri::command]
pub fn cmd_get_tts_plugins(db: State<'_, DbState>) -> Result<Vec<TtsPluginDto>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let plugins = queries::get_all_tts_plugins(&conn).map_err(|e| e.to_string())?;
    Ok(plugins.into_iter().map(to_dto).collect())
}

#[tauri::command]
pub fn cmd_create_tts_plugin(
    db: State<'_, DbState>,
    plugin: TtsPluginDto,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let p = from_dto(&plugin);
    queries::create_tts_plugin(&conn, &p).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_update_tts_plugin(
    db: State<'_, DbState>,
    plugin: TtsPluginDto,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let p = from_dto(&plugin);
    queries::update_tts_plugin(&conn, &p).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_delete_tts_plugin(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    queries::delete_tts_plugin(&conn, &id).map_err(|e| e.to_string())
}

/// List voices from the built-in NCN provider (no plugin registration needed).
#[tauri::command]
pub async fn cmd_list_ncn_voices() -> Result<Vec<TtsVoice>, String> {
    let provider = NcnProvider::new(String::new());
    provider.list_voices().await
}

#[tauri::command]
pub async fn cmd_list_tts_voices(
    db: State<'_, DbState>,
    plugin_id: String,
) -> Result<Vec<TtsVoice>, String> {
    let plugin = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        queries::get_tts_plugin(&conn, &plugin_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("未找到插件 {plugin_id}"))?
    };
    let provider = build_provider(&plugin)?;
    provider.list_voices().await
}

#[tauri::command]
pub async fn cmd_test_tts_plugin(
    db: State<'_, DbState>,
    plugin_id: String,
    sample_text: String,
) -> Result<String, String> {
    let plugin = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        queries::get_tts_plugin(&conn, &plugin_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("未找到插件 {plugin_id}"))?
    };
    let provider = build_provider(&plugin)?;

    let tmp_path = std::env::temp_dir()
        .join(format!("tts_test_{}.mp3", uuid::Uuid::new_v4()));
    let output_path = tmp_path.to_string_lossy().to_string();

    let req = TtsSynthRequest {
        text: sample_text,
        voice_id: None,
        reference_audio_path: None,
        output_path: output_path.clone(),
    };

    provider.synthesize(req).await?;

    // Read result, encode as base64 for preview
    let bytes = std::fs::read(&output_path)
        .map_err(|e| format!("读取测试音频失败: {e}"))?;
    let _ = std::fs::remove_file(&output_path);
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}
