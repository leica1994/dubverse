use crate::ai_pool::AiPoolManager;
use crate::db::connection::DbState;
use crate::db::queries::{
    create_ai_config, delete_ai_config, get_all_ai_configs, set_default_ai_config,
    update_ai_config, AiConfig,
};
use serde_json::json;
use tauri::State;

#[tauri::command]
pub async fn cmd_get_ai_configs(db: State<'_, DbState>) -> Result<Vec<AiConfig>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    get_all_ai_configs(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_create_ai_config(
    db: State<'_, DbState>,
    config: AiConfig,
) -> Result<String, String> {
    let id = config.id.clone();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    create_ai_config(&conn, &config).map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn cmd_update_ai_config(
    db: State<'_, DbState>,
    pool: State<'_, AiPoolManager>,
    config: AiConfig,
) -> Result<(), String> {
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        update_ai_config(&conn, &config).map_err(|e| e.to_string())?;
    }
    // Keep pool in sync — update concurrency/rate limits and invalidate cached client
    pool.update_controller(
        &config.id,
        config.concurrent_limit as u32,
        config.rate_limit as u32,
    )
    .await;
    pool.update_timeout(&config.id).await;
    Ok(())
}

#[tauri::command]
pub async fn cmd_delete_ai_config(
    db: State<'_, DbState>,
    pool: State<'_, AiPoolManager>,
    id: String,
) -> Result<(), String> {
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        delete_ai_config(&conn, &id).map_err(|e| e.to_string())?;
    }
    pool.remove(&id).await;
    Ok(())
}

#[tauri::command]
pub async fn cmd_set_default_ai_config(
    db: State<'_, DbState>,
    id: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    set_default_ai_config(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_test_ai_connection(
    base_url: String,
    api_key: String,
    model: String,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": "hi"}],
        "max_tokens": 1
    });

    let resp = client
        .post(&url)
        .bearer_auth(&api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;

    let status = resp.status();
    if status.is_success() {
        Ok(format!("连接成功，模型: {model}"))
    } else {
        let text = resp
            .text()
            .await
            .unwrap_or_else(|_| status.to_string());
        Err(format!("HTTP {}: {}", status.as_u16(), text))
    }
}
