use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub rooms: Option<Vec<String>>,
    pub gemini_api_key: Option<String>,
    pub groq_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub models: Option<Vec<String>>,
    pub image_generation_enabled: Option<bool>,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rooms_joined = self.rooms.clone().unwrap_or_default().join(", ");
        write!(f, "username: {}, password len: {}, room_names: {}", self.username, self.password.len(), rooms_joined)
    }
}