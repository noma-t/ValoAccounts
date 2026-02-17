-- Skins Database Schema (skins.db)

CREATE TABLE IF NOT EXISTS info (
    version TEXT
);
INSERT OR IGNORE INTO info (rowid, version) VALUES (1, NULL);

CREATE TABLE IF NOT EXISTS tiers (
    uuid TEXT PRIMARY KEY,
    color TEXT,
    rank INTEGER,
    displayIcon TEXT
);

CREATE TABLE IF NOT EXISTS weapons (
    uuid TEXT PRIMARY KEY,
    displayName TEXT NOT NULL,
    displayIcon TEXT,
    tierUuid TEXT,
    FOREIGN KEY (tierUuid) REFERENCES tiers(uuid)
);

CREATE TABLE IF NOT EXISTS levels (
    uuid TEXT PRIMARY KEY,
    weaponUuid TEXT NOT NULL,
    displayName TEXT,
    displayIcon TEXT,
    streamedVideo TEXT,
    FOREIGN KEY (weaponUuid) REFERENCES weapons(uuid)
);

CREATE TABLE IF NOT EXISTS chromas (
    uuid TEXT PRIMARY KEY,
    weaponUuid TEXT NOT NULL,
    displayName TEXT,
    displayIcon TEXT,
    streamedVideo TEXT,
    swatch TEXT,
    FOREIGN KEY (weaponUuid) REFERENCES weapons(uuid)
);

CREATE INDEX IF NOT EXISTS idx_levels_weapon ON levels(weaponUuid);
CREATE INDEX IF NOT EXISTS idx_chromas_weapon ON chromas(weaponUuid);
