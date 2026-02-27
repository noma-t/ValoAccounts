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

CREATE TABLE IF NOT EXISTS buddies (
    uuid TEXT PRIMARY KEY,
    displayName TEXT NOT NULL,
    displayIcon TEXT,
    assetPath TEXT
);

CREATE TABLE IF NOT EXISTS buddy_levels (
    uuid TEXT PRIMARY KEY,
    buddyUuid TEXT NOT NULL,
    charmLevel INTEGER,
    displayName TEXT,
    displayIcon TEXT,
    assetPath TEXT,
    FOREIGN KEY (buddyUuid) REFERENCES buddies(uuid)
);

CREATE INDEX IF NOT EXISTS idx_buddy_levels_buddy ON buddy_levels(buddyUuid);

CREATE TABLE IF NOT EXISTS flex (
    uuid TEXT PRIMARY KEY,
    displayName TEXT NOT NULL,
    displayIcon TEXT,
    assetPath TEXT
);

CREATE TABLE IF NOT EXISTS playercards (
    uuid TEXT PRIMARY KEY,
    displayName TEXT NOT NULL,
    displayIcon TEXT,
    smallArt TEXT,
    wideArt TEXT,
    largeArt TEXT,
    assetPath TEXT
);

CREATE TABLE IF NOT EXISTS sprays (
    uuid TEXT PRIMARY KEY,
    displayName TEXT NOT NULL,
    displayIcon TEXT,
    fullTransparentIcon TEXT,
    animationGif TEXT,
    assetPath TEXT
);

CREATE TABLE IF NOT EXISTS spray_levels (
    uuid TEXT PRIMARY KEY,
    sprayUuid TEXT NOT NULL,
    sprayLevel INTEGER,
    displayName TEXT,
    displayIcon TEXT,
    assetPath TEXT,
    FOREIGN KEY (sprayUuid) REFERENCES sprays(uuid)
);

CREATE INDEX IF NOT EXISTS idx_spray_levels_spray ON spray_levels(sprayUuid);
