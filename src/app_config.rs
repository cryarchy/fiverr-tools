use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub browser_ws_url: String,
    pub log_level: String,
    pub database_url: String,
}
