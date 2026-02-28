-- 配音任务主表（每个 project_dir 对应一个）
CREATE TABLE IF NOT EXISTS dubbing_jobs (
  id             TEXT PRIMARY KEY,
  project_dir    TEXT NOT NULL UNIQUE,
  video_path     TEXT NOT NULL,
  subtitle_count INTEGER NOT NULL DEFAULT 0,
  -- 参考音频模式: 'none' | 'custom' | 'clone'
  reference_mode TEXT NOT NULL DEFAULT 'none',
  reference_audio_path TEXT,
  tts_plugin_id  TEXT,
  status         TEXT NOT NULL DEFAULT 'pending',
  current_stage  TEXT,
  error          TEXT,
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL
);

-- 各阶段状态
CREATE TABLE IF NOT EXISTS dubbing_stage_states (
  job_id    TEXT NOT NULL,
  stage     TEXT NOT NULL,
  status    TEXT NOT NULL DEFAULT 'pending',
  progress  INTEGER DEFAULT 0,
  output_path TEXT,
  error     TEXT,
  completed_at TEXT,
  PRIMARY KEY (job_id, stage)
);

-- 每行字幕的 TTS 处理状态（断点续传粒度：单行）
CREATE TABLE IF NOT EXISTS dubbing_tts_items (
  job_id              TEXT NOT NULL,
  subtitle_index      INTEGER NOT NULL,
  preprocessed_text   TEXT NOT NULL,
  start_ms            INTEGER NOT NULL,
  end_ms              INTEGER NOT NULL,
  reference_audio_path TEXT,
  tts_audio_path      TEXT,
  tts_duration_ms     INTEGER,
  status              TEXT NOT NULL DEFAULT 'pending',
  retry_count         INTEGER DEFAULT 0,
  error               TEXT,
  completed_at        TEXT,
  PRIMARY KEY (job_id, subtitle_index)
);

-- TTS 插件/提供商配置
CREATE TABLE IF NOT EXISTS tts_plugins (
  id              TEXT PRIMARY KEY,
  name            TEXT NOT NULL,
  plugin_type     TEXT NOT NULL,
  config_json     TEXT NOT NULL DEFAULT '{}',
  requires_ref    INTEGER NOT NULL DEFAULT 0,
  is_enabled      INTEGER NOT NULL DEFAULT 1,
  sort_order      INTEGER DEFAULT 0,
  created_at      TEXT NOT NULL
);
