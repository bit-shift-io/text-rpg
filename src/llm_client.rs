use crate::config::Config;
use genai::chat::{ChatMessage, ChatRequest};
use genai::Client;
use genai::resolver::AuthData;
use genai::chat::printer::print_chat_stream;
pub struct LlmClient {
    client: Client,
    default_model: String,
}

impl LlmClient {
    pub fn new(config: Config) -> Self {
        let client_builder = Client::builder();
        let mut available_models = vec![];

        if config.gemini_api_key.is_some() {
            unsafe {
                std::env::set_var("GEMINI_API_KEY", config.gemini_api_key.unwrap());
            }

            if config.gemini_models.is_some() {
                let models = config.gemini_models.unwrap();
                available_models.extend(models);
            }
        }

        if config.groq_api_key.is_some() {
            unsafe {
                std::env::set_var("GROQ_API_KEY", config.groq_api_key.unwrap());
            }

            if config.groq_models.is_some() {
                let models = config.groq_models.unwrap();
                available_models.extend(models);
            }
        }

        LlmClient {
             client: client_builder.build(),
             default_model: available_models[0].clone(),
        }
    }

    pub async fn chat(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // https://github.com/jeremychone/rust-genai/tree/main/examples
        let chat_req = ChatRequest::new(vec![
            ChatMessage::user(prompt.to_string()),
        ]);

        let response = self.client.exec_chat_stream(&self.default_model, chat_req, None).await?;
        let content = print_chat_stream(response, None).await?;
        Ok(content)
    }
}
