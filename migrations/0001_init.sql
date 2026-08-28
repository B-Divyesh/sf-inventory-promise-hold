PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS app_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
  singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
  location_name TEXT NOT NULL,
  supervisor_pin_hash TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
  token_hash TEXT PRIMARY KEY,
  expires_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS inventory (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  sku TEXT NOT NULL COLLATE NOCASE UNIQUE,
  name TEXT NOT NULL,
  on_hand INTEGER NOT NULL CHECK (on_hand >= 0),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS holds (
  id TEXT PRIMARY KEY,
  inventory_id INTEGER NOT NULL REFERENCES inventory(id),
  quantity INTEGER NOT NULL CHECK (quantity > 0),
  customer TEXT NOT NULL,
  order_note TEXT NOT NULL DEFAULT '',
  operator_name TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('active', 'converted', 'released', 'expired')),
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  resolved_at INTEGER,
  resolved_by TEXT
);

CREATE INDEX IF NOT EXISTS idx_holds_active_inventory
ON holds(inventory_id, status, expires_at);

CREATE INDEX IF NOT EXISTS idx_holds_created
ON holds(created_at DESC);

CREATE TABLE IF NOT EXISTS audit_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  event TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  actor TEXT NOT NULL,
  details_json TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_created
ON audit_log(created_at DESC);

