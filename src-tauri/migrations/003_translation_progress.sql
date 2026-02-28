CREATE TABLE IF NOT EXISTS translation_progress (
    project_dir    TEXT NOT NULL,
    subtitle_index INTEGER NOT NULL,
    phase          TEXT NOT NULL,
    result_text    TEXT NOT NULL,
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (project_dir, subtitle_index, phase)
);
