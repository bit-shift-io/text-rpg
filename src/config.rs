use serde::Deserialize;
use std::{fmt};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub rooms: Option<Vec<String>>,
    pub aichat_config_file: Option<String>,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rooms_joined = self.rooms.clone().unwrap_or_default().join(", ");
        write!(f, "username: {}, password len: {}, room_names: {}, aichat_config_file: {}", self.username, self.password.len(), rooms_joined, self.aichat_config_file.clone().unwrap_or("".to_string()))
    }
}