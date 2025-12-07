use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct LlmConfig {
    pub model: String,
    pub clients: Vec<ClientConfig>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ClientConfig {
    OpenaiCompatible {
        name: String,
        api_base: String,
        api_key: String,
    },
    Gemini {
        api_key: String,
        // Gemini config in aichat might not have name/api_base, just api_key
    },
}

impl LlmConfig {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(path)?;
        let config: LlmConfig = serde_yml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let yaml = r#"
model: groq
clients:
  - type: openai-compatible
    name: groq
    api_base: https://api.groq.com/openai/v1
    api_key: gsk_test
  - type: gemini
    api_key: AIzaTest
"#;
        let config: LlmConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.model, "groq");
        assert_eq!(config.clients.len(), 2);
    }
}
