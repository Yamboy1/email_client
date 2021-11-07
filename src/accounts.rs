use std::fs;
use serde::{Deserialize};

#[derive(Deserialize)]
pub struct ImapAccountConfig {
    pub domain: String,
    pub port: u16,
    pub email: String,
    pub password: String
}

pub fn get_account_config() -> crate::Result<ImapAccountConfig> {
    let file_content = fs::read_to_string("Accounts.toml")?;
    Ok(toml::from_str(&file_content)?)
}