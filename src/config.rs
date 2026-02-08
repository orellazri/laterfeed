use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub cors_allowed_origins: Vec<String>,
    pub base_url: String,
}
