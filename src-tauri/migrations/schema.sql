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
