use crate::config::Config;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatRequest};
use genai::Client;
use genai::resolver::AuthData;
use genai::chat::printer::print_chat_stream;
use tracing::{info, warn};

pub struct LlmClient {
    client: Client,
    available_models: Vec<String>,
    adapter_kinds: Vec<AdapterKind>
}

impl LlmClient {
    pub fn new(config: Config) -> Self {
        let client_builder = Client::builder();
        let client = Client::default();
        let mut available_models = vec![];
        let mut adapter_kinds = vec![];

        if config.models.is_some() {
            available_models = config.models.unwrap();             
        }

        if config.gemini_api_key.is_some() {
            unsafe {
                std::env::set_var("GEMINI_API_KEY", config.gemini_api_key.unwrap());
            }
            adapter_kinds.push(AdapterKind::Gemini);
        }

        if config.groq_api_key.is_some() {
            unsafe {
                std::env::set_var("GROQ_API_KEY", config.groq_api_key.unwrap());
            }
            adapter_kinds.push(AdapterKind::Groq);
        }

        if config.openai_api_key.is_some() {
            unsafe {
                std::env::set_var("OPENAI_API_KEY", config.openai_api_key.unwrap());
            }
            adapter_kinds.push(AdapterKind::OpenAI);
        }
        
        LlmClient {
             client: client_builder.build(),
             available_models,
             adapter_kinds,
        }
    }

    pub async fn populate_models(&mut self) {
        // https://github.com/jeremychone/rust-genai/blob/main/examples/c05-model-names.rs
        for kind in &self.adapter_kinds {
            let models = match self.client.all_model_names(*kind).await {
                Ok(response) => response,
                Err(e) => {
                    warn!("[populate_models] all_model_names failed for kind '{}': {}", kind, e);
                    return ();
                },
            };

            self.available_models.extend(models);
        }
    }

    pub async fn chat(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // https://github.com/jeremychone/rust-genai/tree/main/examples
        let chat_req = ChatRequest::new(vec![
            ChatMessage::user(prompt.to_string()),
        ]);

        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;

        for model in &self.available_models {
            match self.client.exec_chat_stream(model, chat_req.clone(), None).await {
                Ok(response) => {
                    match print_chat_stream(response, None).await {
                        Ok(content) => return Ok(content),
                        Err(e) => {
                            warn!("Failed to read stream from model '{}': {}", model, e);
                            last_error = Some(Box::new(e));
                        }
                    }
                },
                Err(e) => {
                    warn!("Model '{}' failed: {}", model, e);
                    // Check if we should retry based on error type? 
                    // For now, we assume standard "try next" behavior for robustness.
                    last_error = Some(e.into());
                }
            }
        }

        if let Some(e) = last_error {
             return Err(format!("All models failed. Last error: {}", e).into());
        }

        Err("No models configured or available.".into())
    }
}
