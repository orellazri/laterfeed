use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub base_url: String,
    pub auth_token: String,
}
