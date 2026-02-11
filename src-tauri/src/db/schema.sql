-- Valorant Accounts Database Schema

-- Accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    riot_id TEXT NOT NULL,
    tagline TEXT NOT NULL,
    username TEXT,
    encrypted_password BLOB NOT NULL,
    rank TEXT,
    is_active BOOLEAN DEFAULT 0,
    data_folder TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Settings table
CREATE TABLE IF NOT EXISTS settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    active_account_id INTEGER,
    riot_client_service_path TEXT,
    riot_client_data_path TEXT,
    account_data_path TEXT,
    henrikdev_api_key TEXT,
    launched INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (active_account_id) REFERENCES accounts(id) ON DELETE SET NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_accounts_username ON accounts(username);
CREATE INDEX IF NOT EXISTS idx_accounts_active ON accounts(is_active);

-- Triggers for automatic timestamp updates
CREATE TRIGGER IF NOT EXISTS update_accounts_timestamp
AFTER UPDATE ON accounts
FOR EACH ROW
BEGIN
    UPDATE accounts SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS update_settings_timestamp
AFTER UPDATE ON settings
FOR EACH ROW
BEGIN
    UPDATE settings SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Initial settings record
INSERT OR IGNORE INTO settings (id)
VALUES (1);
