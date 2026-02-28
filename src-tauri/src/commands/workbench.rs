use tauri::State;
use crate::db::connection::DbState;
use crate::db::queries::{
    WorkbenchTask, WorkbenchStepTranscribe, WorkbenchStepTranslate, WorkbenchTaskFull,
    create_workbench_task, update_workbench_task_progress, list_workbench_tasks,
    get_workbench_task_full, delete_workbench_task, upsert_step_transcribe, upsert_step_translate,
};

#[tauri::command]
pub fn cmd_create_workbench_task(
    state: State<DbState>,
    video_path: String,
    video_name: String,
    video_size: i64,
    video_duration: f64,
    video_width: i32,
    video_height: i32,
    project_dir: String,
    source_language: String,
    target_language: String,
) -> Result<WorkbenchTask, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let name = video_name
        .rsplit_once('.')
        .map(|(n, _)| n.to_string())
        .unwrap_or_else(|| video_name.clone());
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let task = WorkbenchTask {
        id,
        name,
        project_dir,
        video_path,
        video_name,
        video_size,
        video_duration,
        video_width,
        video_height,
        current_step: 0,
        step_statuses: "[\"completed\",\"ready\",\"idle\",\"idle\",\"idle\"]".to_string(),
        source_language,
        target_language,
        status: "active".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    create_workbench_task(&conn, &task).map_err(|e| e.to_string())?;
    Ok(task)
}

#[tauri::command]
pub fn cmd_update_workbench_task_progress(
    state: State<DbState>,
    task_id: String,
    current_step: i32,
    step_statuses: Vec<String>,
    source_language: String,
    target_language: String,
    status: String,
) -> Result<(), String> {
    let statuses_json = serde_json::to_string(&step_statuses).map_err(|e| e.to_string())?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    update_workbench_task_progress(
        &conn, &task_id, current_step, &statuses_json, &source_language, &target_language, &status,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_save_transcribe_step(
    state: State<DbState>,
    task_id: String,
    config_json: String,
    subtitles_path: String,
    subtitle_count: i32,
) -> Result<(), String> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let step = WorkbenchStepTranscribe {
        task_id,
        config_json,
        subtitles_path: Some(subtitles_path),
        subtitle_count,
        completed_at: Some(now),
    };
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    upsert_step_transcribe(&conn, &step).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_save_translate_step(
    state: State<DbState>,
    task_id: String,
    config_json: String,
    translated_subtitles_path: String,
    subtitles_json: String,
    subtitle_count: i32,
) -> Result<(), String> {
    // Write translated subtitles to file
    std::fs::write(&translated_subtitles_path, &subtitles_json)
        .map_err(|e| format!("写入翻译字幕失败: {}", e))?;

    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let step = WorkbenchStepTranslate {
        task_id,
        config_json,
        translated_subtitles_path: Some(translated_subtitles_path),
        subtitle_count,
        completed_at: Some(now),
    };
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    upsert_step_translate(&conn, &step).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_load_subtitles(subtitles_path: String) -> Result<String, String> {
    std::fs::read_to_string(&subtitles_path)
        .map_err(|e| format!("读取字幕文件失败: {}", e))
}

#[tauri::command]
pub fn cmd_list_workbench_tasks(state: State<DbState>) -> Result<Vec<WorkbenchTask>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    list_workbench_tasks(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_get_workbench_task_full(
    state: State<DbState>,
    task_id: String,
) -> Result<Option<WorkbenchTaskFull>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_workbench_task_full(&conn, &task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_delete_workbench_task(
    state: State<DbState>,
    task_id: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    // 1. Query project_dir before deleting DB record
    let project_dir: Option<String> = conn
        .query_row(
            "SELECT project_dir FROM workbench_tasks WHERE id = ?1",
            rusqlite::params![&task_id],
            |row| row.get(0),
        )
        .ok();

    // 2. Delete DB record (CASCADE removes child table rows)
    delete_workbench_task(&conn, &task_id).map_err(|e| e.to_string())?;

    // 3. Remove disk directory (ignore errors, don't block the operation)
    if let Some(dir) = project_dir {
        let _ = std::fs::remove_dir_all(&dir);
    }

    Ok(())
}
