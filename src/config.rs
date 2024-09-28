use serde::Deserialize;
use std::{fmt};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub username: String,
    pub password: String,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "username: {}, password len: {}", self.username, self.password.len())
    }
}