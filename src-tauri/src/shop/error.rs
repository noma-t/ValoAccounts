#[derive(Debug)]
pub enum ShopError {
    Http(reqwest::Error),
    AuthFailed(String),
    ParseError(String),
    StorefrontFailed,
    VersionFetchFailed(String),
}

impl std::fmt::Display for ShopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(e) => write!(f, "HTTP error: {}", e),
            Self::AuthFailed(msg) => write!(f, "Authentication failed: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::StorefrontFailed => write!(f, "All storefront endpoints failed"),
            Self::VersionFetchFailed(msg) => write!(f, "Version fetch failed: {}", msg),
        }
    }
}

impl std::error::Error for ShopError {}

impl From<reqwest::Error> for ShopError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}
