CREATE TABLE IF NOT EXISTS app_config (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS provider_secrets (
    provider_id TEXT PRIMARY KEY,
    secret_json TEXT NOT NULL,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ai_configs (
    id               TEXT PRIMARY KEY,
    title            TEXT NOT NULL,
    base_url         TEXT NOT NULL DEFAULT 'https://api.openai.com/v1',
    api_key          TEXT NOT NULL DEFAULT '',
    model            TEXT NOT NULL DEFAULT 'gpt-4o-mini',
    sort_order       INTEGER NOT NULL DEFAULT 0,
    is_default       INTEGER NOT NULL DEFAULT 0,
    concurrent_limit INTEGER NOT NULL DEFAULT 5,
    request_timeout  INTEGER NOT NULL DEFAULT 180,
    rate_limit       INTEGER NOT NULL DEFAULT 60
);

CREATE TABLE IF NOT EXISTS translation_progress (
    project_dir    TEXT NOT NULL,
    subtitle_index INTEGER NOT NULL,
    phase          TEXT NOT NULL,
    result_text    TEXT NOT NULL,
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (project_dir, subtitle_index, phase)
);

CREATE TABLE IF NOT EXISTS dubbing_jobs (
    id                   TEXT PRIMARY KEY,
    project_dir          TEXT NOT NULL UNIQUE,
    video_path           TEXT NOT NULL,
    subtitle_count       INTEGER NOT NULL DEFAULT 0,
    reference_mode       TEXT NOT NULL DEFAULT 'none',
    reference_audio_path TEXT,
    tts_plugin_id        TEXT,
    status               TEXT NOT NULL DEFAULT 'pending',
    current_stage        TEXT,
    error                TEXT,
    created_at           TEXT NOT NULL,
    updated_at           TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS dubbing_stage_states (
    job_id       TEXT NOT NULL,
    stage        TEXT NOT NULL,
    status       TEXT NOT NULL DEFAULT 'pending',
    progress     INTEGER DEFAULT 0,
    output_path  TEXT,
    error        TEXT,
    completed_at TEXT,
    PRIMARY KEY (job_id, stage)
);

CREATE TABLE IF NOT EXISTS dubbing_tts_items (
    job_id               TEXT NOT NULL,
    subtitle_index       INTEGER NOT NULL,
    preprocessed_text    TEXT NOT NULL,
    start_ms             INTEGER NOT NULL,
    end_ms               INTEGER NOT NULL,
    reference_audio_path TEXT,
    tts_audio_path       TEXT,
    tts_duration_ms      INTEGER,
    status               TEXT NOT NULL DEFAULT 'pending',
    retry_count          INTEGER DEFAULT 0,
    error                TEXT,
    completed_at         TEXT,
    PRIMARY KEY (job_id, subtitle_index)
);

CREATE TABLE IF NOT EXISTS tts_plugins (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    plugin_type TEXT NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    requires_ref INTEGER NOT NULL DEFAULT 0,
    is_enabled   INTEGER NOT NULL DEFAULT 1,
    sort_order   INTEGER DEFAULT 0,
    created_at   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS workbench_tasks (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    project_dir     TEXT NOT NULL UNIQUE,
    video_path      TEXT NOT NULL,
    video_name      TEXT NOT NULL,
    video_size      INTEGER NOT NULL DEFAULT 0,
    video_duration  REAL    NOT NULL DEFAULT 0,
    video_width     INTEGER NOT NULL DEFAULT 0,
    video_height    INTEGER NOT NULL DEFAULT 0,
    current_step    INTEGER NOT NULL DEFAULT 0,
    step_statuses   TEXT    NOT NULL DEFAULT '["completed","ready","idle","idle","idle"]',
    source_language TEXT    NOT NULL DEFAULT 'auto',
    target_language TEXT    NOT NULL DEFAULT 'zh',
    status          TEXT    NOT NULL DEFAULT 'active',
    created_at      TEXT    NOT NULL,
    updated_at      TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS workbench_step_transcribe (
    task_id        TEXT PRIMARY KEY REFERENCES workbench_tasks(id) ON DELETE CASCADE,
    config_json    TEXT NOT NULL DEFAULT '{}',
    subtitles_path TEXT,
    subtitle_count INTEGER NOT NULL DEFAULT 0,
    completed_at   TEXT
);

CREATE TABLE IF NOT EXISTS workbench_step_translate (
    task_id                   TEXT PRIMARY KEY REFERENCES workbench_tasks(id) ON DELETE CASCADE,
    config_json               TEXT NOT NULL DEFAULT '{}',
    translated_subtitles_path TEXT,
    subtitle_count            INTEGER NOT NULL DEFAULT 0,
    completed_at              TEXT
);

CREATE INDEX IF NOT EXISTS idx_workbench_tasks_created ON workbench_tasks(created_at DESC);
