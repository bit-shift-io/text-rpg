use crate::config::Config;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatRequest};
use genai::Client;
use genai::resolver::AuthData;
use genai::chat::printer::print_chat_stream;
use tracing::{info, warn};
use std::fmt;

#[derive(Debug)]
pub struct LlmClient {
    client: Client,
    models: Vec<String>,
    adapter_kinds: Vec<AdapterKind>,
    model_idx: usize,
}

impl LlmClient {
    pub fn new(config: Config) -> Self {
        let client_builder = Client::builder();
        let client = Client::default();
        let mut models = vec![];
        let mut adapter_kinds = vec![];

        if config.models.is_some() {
            models = config.models.unwrap();             
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
             models,
             adapter_kinds,
             model_idx: 0,
        }
    }

    pub fn model(&self) -> &String {
        &self.models[self.model_idx]
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

            self.models.extend(models);
        }
    }

    pub async fn chat(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // https://github.com/jeremychone/rust-genai/tree/main/examples
        let chat_req = ChatRequest::new(vec![
            ChatMessage::user(prompt.to_string()),
        ]);

        //let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;
        let model = &self.models[self.model_idx];
        match self.client.exec_chat(model, chat_req.clone(), None).await {
            Ok(response) => {
                return Ok(response.first_text().unwrap_or("NO ANSWER").to_string())

                // match print_chat_stream(response, None).await {
                //     Ok(content) => return Ok(content),
                //     Err(e) => {
                //         warn!("Failed to read stream from model '{}': {}", model, e);
                //         //last_error = Some(Box::new(e));
                //         return Err(e.into());
                //     }
                // }
            },
            Err(e) => {
                warn!("Model '{}' failed: {}", model, e);
                // Check if we should retry based on error type? 
                // For now, we assume standard "try next" behavior for robustness.
                //last_error = Some(e.into());
                return Err(e.into());
            }
        }

        // if let Some(e) = last_error {
        //      return Err(format!("All models failed. Last error: {}", e).into());
        // }

        // Err("No models configured or available.".into())
    }

    pub async fn chat_with_retry(&mut self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let model_idx = self.model_idx;
        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>>;// = None;

        loop {
            let res = self.chat(&prompt).await;
            match res {
                Ok(result) => {
                    //info!("[chat_with_retry] result: {}", result);
                    return Ok(result)
                },
                Err(e) => {
                    //error!("[execute_prompt] Failed to execute prompt: {}", e);
                    self.move_to_next_adapter_or_model();
                    last_error = Some(e.into());
                    //self.room.send(RoomMessageEventContent::notice_plain(format!("[execute_prompt] Failed to execute prompt: {}", e))).await.unwrap();
                    //Err(())
                }
            }

            if model_idx == self.model_idx {
                break;
            }
        }

        if let Some(e) = last_error {
             return Err(format!("All models failed. Last error: {}", e).into());
        }

        Err("No models configured or available.".into())
    }

    pub fn move_to_next_adapter_or_model(&mut self) {
        self.model_idx += 1;
        if self.model_idx >= self.models.len() {
            self.model_idx = 0;
        }
    }
}

impl fmt::Display for LlmClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //let rooms_joined = self.rooms.clone().unwrap_or_default().join(", ");
        let models = self.models.join(", ");
        //let adapters = self.adapter_kinds.join(", ");
        write!(f, "models: {}", models) //, adapters)
    }
}