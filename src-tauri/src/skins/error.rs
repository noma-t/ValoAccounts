#[derive(Debug)]
pub enum SkinsError {
    Http(reqwest::Error),
    Database(String),
    ApiFailed(String),
}

impl std::fmt::Display for SkinsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(e) => write!(f, "HTTP error: {}", e),
            Self::Database(msg) => write!(f, "Database error: {}", msg),
            Self::ApiFailed(msg) => write!(f, "API failed: {}", msg),
        }
    }
}

impl std::error::Error for SkinsError {}

impl From<reqwest::Error> for SkinsError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

impl From<rusqlite::Error> for SkinsError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e.to_string())
    }
}
