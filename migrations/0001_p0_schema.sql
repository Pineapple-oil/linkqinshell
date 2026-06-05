CREATE TABLE IF NOT EXISTS connections (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  host TEXT NOT NULL,
  port INTEGER NOT NULL,
  username TEXT NOT NULL,
  favorite INTEGER NOT NULL DEFAULT 0,
  recent_at TEXT,
  deleted_at TEXT,
  recover_until TEXT,
  payload TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_connections_deleted_at ON connections(deleted_at);
CREATE INDEX IF NOT EXISTS idx_connections_name ON connections(name);

CREATE TABLE IF NOT EXISTS connection_groups (
  id TEXT PRIMARY KEY,
  parent_id TEXT,
  name TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS connection_tags (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  color TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS connection_tag_links (
  connection_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  PRIMARY KEY (connection_id, tag_id)
);

CREATE TABLE IF NOT EXISTS host_keys (
  id TEXT PRIMARY KEY,
  host TEXT NOT NULL,
  port INTEGER NOT NULL,
  key_type TEXT NOT NULL,
  fingerprint_sha256 TEXT NOT NULL,
  fingerprint_md5 TEXT,
  public_key TEXT NOT NULL,
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  trusted INTEGER NOT NULL DEFAULT 0,
  UNIQUE(host, port)
);

CREATE TABLE IF NOT EXISTS task_runs (
  id TEXT PRIMARY KEY,
  task_type TEXT NOT NULL,
  title TEXT NOT NULL,
  status TEXT NOT NULL,
  progress REAL NOT NULL DEFAULT 0,
  error TEXT,
  started_at TEXT NOT NULL,
  finished_at TEXT
);

CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_events (
  id TEXT PRIMARY KEY,
  action TEXT NOT NULL,
  connection_id TEXT,
  message TEXT NOT NULL,
  created_at TEXT NOT NULL
);
