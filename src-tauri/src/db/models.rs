use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub riot_id: String,
    pub tagline: String,
    pub username: Option<String>,
    #[serde(skip_serializing)]
    pub encrypted_password: Vec<u8>,
    pub has_password: bool,
    pub rank: Option<String>,
    pub is_active: bool,
    pub data_folder: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountWithPassword {
    pub id: i64,
    pub riot_id: String,
    pub tagline: String,
    pub username: Option<String>,
    pub password: String,
    pub rank: Option<String>,
    pub is_active: bool,
    pub data_folder: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAccount {
    pub riot_id: String,
    pub tagline: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub rank: Option<String>,
    pub use_current_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub id: i64,
    pub active_account_id: Option<i64>,
    pub riot_client_service_path: Option<String>,
    pub riot_client_data_path: Option<String>,
    pub account_data_path: Option<String>,
    pub henrikdev_api_key: Option<String>,
    pub region: Option<String>,
    pub launched: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccount {
    pub id: i64,
    pub riot_id: String,
    pub tagline: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub rank: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettings {
    pub active_account_id: Option<i64>,
    pub riot_client_service_path: Option<String>,
    pub riot_client_data_path: Option<String>,
    pub account_data_path: Option<String>,
    pub henrikdev_api_key: Option<String>,
    pub region: Option<String>,
}
