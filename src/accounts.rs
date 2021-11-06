use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ImapAccountConfig {
    pub domain: String,
    pub port: u16,
    pub email: String,
    pub password: String
}

pub fn get_account_config() -> Result<ImapAccountConfig, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string("Accounts.toml")?;
    Ok(toml::from_str(&file_content)?)
}