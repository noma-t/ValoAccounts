# Database Documentation

## Overview

valo-accounts uses SQLite3 for data persistence with AES-256-GCM encryption for passwords. Encryption keys are securely stored in Windows Credential Manager.

## Database Location

Default: Same directory as the application executable (`accounts.db`)

Example paths:
- Development: `src-tauri/target/debug/accounts.db`
- Release: `<install_dir>/accounts.db`

## Tables

### accounts

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Primary key |
| riot_id | TEXT | Riot ID |
| tagline | TEXT | Tagline |
| email | TEXT | Email/Account ID (UNIQUE) |
| encrypted_password | BLOB | AES-256-GCM encrypted password |
| rank | TEXT | Valorant rank (nullable) |
| is_active | BOOLEAN | Currently active account |
| created_at | DATETIME | Creation timestamp |
| updated_at | DATETIME | Last update timestamp |

Constraints:
- UNIQUE (riot_id, tagline)
- UNIQUE (email)

### settings

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Primary key (always 1) |
| active_account_id | INTEGER | FK to accounts.id |
| database_path | TEXT | Custom database path |
| riot_client_path | TEXT | Riot Client executable path |
| account_data_path | TEXT | Account data directory |
| created_at | DATETIME | Creation timestamp |
| updated_at | DATETIME | Last update timestamp |

Constraints:
- CHECK (id = 1) - Only one settings record allowed
- FK: active_account_id -> accounts.id ON DELETE SET NULL

## Security

### Encryption

Passwords are encrypted using AES-256-GCM with:
- 256-bit random encryption key
- 96-bit random nonce per encryption
- Authenticated encryption (prevents tampering)

### Key Storage

Encryption keys are stored in **Windows Credential Manager**:
- Service Name: `valo-accounts`
- Key Name: `encryption_key`
- Format: Base64-encoded 32-byte key

Keys are automatically generated on first run and persist across application reinstalls.

## Usage

### Initialize Database

```rust
use valo_accounts_lib::db::initialize_database;

// Use default path (%APPDATA%\valo-accounts\accounts.db)
let conn = initialize_database(None)?;

// Or specify custom path
let conn = initialize_database(Some(PathBuf::from("custom.db")))?;
```

### Get Connection

```rust
use valo_accounts_lib::db::get_connection;

let conn = get_connection(None)?;
```

### Encryption/Decryption

```rust
use valo_accounts_lib::crypto::{
    get_or_create_encryption_key,
    encrypt_password,
    decrypt_password,
};

// Get or create encryption key from Windows Credential Manager
let key = get_or_create_encryption_key()?;

// Encrypt password
let password = "MySecurePassword123!";
let encrypted = encrypt_password(password, &key)?;

// Decrypt password
let decrypted = decrypt_password(&encrypted, &key)?;
assert_eq!(password, decrypted);
```

## Models

### Account

```rust
pub struct Account {
    pub id: i64,
    pub riot_id: String,
    pub tagline: String,
    pub email: String,
    pub encrypted_password: Vec<u8>,  // Not serialized to JSON
    pub rank: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
```

### AccountWithPassword

```rust
pub struct AccountWithPassword {
    pub id: i64,
    pub riot_id: String,
    pub tagline: String,
    pub email: String,
    pub password: String,  // Decrypted password
    pub rank: Option<String>,
    pub is_active: bool,
}
```

### NewAccount

```rust
pub struct NewAccount {
    pub riot_id: String,
    pub tagline: String,
    pub email: String,
    pub password: String,  // Plain text (will be encrypted)
    pub rank: Option<String>,
}
```

### Settings

```rust
pub struct Settings {
    pub id: i64,
    pub active_account_id: Option<i64>,
    pub database_path: Option<String>,
    pub riot_client_path: Option<String>,
    pub account_data_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
```

### UpdateSettings

```rust
pub struct UpdateSettings {
    pub active_account_id: Option<i64>,
    pub database_path: Option<String>,
    pub riot_client_path: Option<String>,
    pub account_data_path: Option<String>,
}
```

## Next Steps

To implement CRUD operations, create:
- `src-tauri/src/db/accounts.rs` - Account CRUD
- `src-tauri/src/db/settings.rs` - Settings CRUD

Example account operations to implement:
- `insert_account(NewAccount) -> Account`
- `get_account_by_id(id) -> Account`
- `get_all_accounts() -> Vec<Account>`
- `update_account(id, updates) -> Account`
- `delete_account(id) -> Result<()>`
- `set_active_account(id) -> Result<()>`

## Testing

Run tests:
```bash
cd src-tauri
cargo test
```

Current tests:
- Database initialization
- Encryption/Decryption
- Nonce randomness
