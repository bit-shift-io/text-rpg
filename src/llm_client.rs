use crate::config::Config;
use rig::providers::openai::Client as OpenAIClient;
use rig::providers::gemini::Client as GeminiClient;
use rig::providers::huggingface::Client as HFClient;
use rig::completion::message::Message as ChatCompletionMessage;
use rig::providers::openai::responses_api::Role;
use rig::client::{ProviderClient, CompletionClient};
use rig::client::image_generation::ImageGenerationClient;
use rig::image_generation::ImageGenerationModel;
use rig::providers::openai;
use rig::providers::huggingface;
use rig::completion::{CompletionModel, Completion};
use rig::OneOrMany;
use rig::agent::Agent;
use async_trait::async_trait;
use tracing::{info, warn, error};
use std::fmt;
use crate::discovery::{list_all_models, ModelType};

#[async_trait]
pub trait AbstractAgent: Send + Sync {
    fn model_name(&self) -> &String;
    async fn perform_chat_completion(
        &self,
        messages: Vec<ChatCompletionMessage>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
impl<M> AbstractAgent for Agent<M>
where
    M: CompletionModel + Send + Sync + 'static,
    Agent<M>: Completion<M>,
{
    fn model_name(&self) -> &String {
        unimplemented!("model_name should be retrieved from LlmClient's model field")
    }

    async fn perform_chat_completion(
        &self,
        messages: Vec<ChatCompletionMessage>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let first_message = messages.first().unwrap().clone();
        let chat_history = messages[1..].to_vec();

        let completion_request_builder = self.completion(
            first_message,
            chat_history,
        ).await?;

        let completion_response = completion_request_builder.send().await?;

        let assistant_content_one_or_many = &completion_response.choice;

        // Iterate over the OneOrMany struct
        if let Some(first_text_content) = assistant_content_one_or_many.iter().find_map(|ac| {
            match ac {
                rig::completion::AssistantContent::Text(text) => Some(text.clone()),
                rig::completion::AssistantContent::ToolCall(tool_call) => {
                    warn!("Received a tool call instead of text: {:?}", tool_call);
                    None // Skip tool calls
                },
                rig::completion::AssistantContent::Reasoning(reasoning) => {
                    warn!("Received a reasoning content, skipping: {:?}", reasoning);
                    None // Skip reasoning
                },
                rig::completion::AssistantContent::Image(image) => {
                    warn!("Received an image content, skipping: {:?}", image);
                    None // Skip image
                }
            }
        }) {
            Ok(first_text_content.to_string())
        } else {
            warn!("Received assistant content, but no text content found.");
            Err(Box::<dyn std::error::Error + Send + Sync>::from("Received assistant content, but no text content found."))
        }
    }
}

pub struct LlmClient {
    config: Config,
    text_models: Vec<String>,
    text_model_idx: usize,
    image_models: Vec<String>,
    image_model_idx: usize,
    agent: Option<Box<dyn AbstractAgent>>,
}

impl LlmClient {
    pub async fn new(config: Config) -> Self {
        if let Some(key) = &config.gemini_api_key {
            unsafe { std::env::set_var("GEMINI_API_KEY", key); }
        }
        if let Some(key) = &config.groq_api_key {
            unsafe { std::env::set_var("GROQ_API_KEY", key); }
        }
        if let Some(key) = &config.openai_api_key {
            unsafe { std::env::set_var("OPENAI_API_KEY", key); }
        }
        if let Some(key) = &config.huggingface_api_key {
            unsafe { std::env::set_var("HUGGINGFACE_API_KEY", key); }
        }

        let discovered_models = list_all_models(config.clone()).await;
        
        let mut final_text_models = Vec::new();
        let mut final_image_models = Vec::new();

        // Handle text models from config
        if let Some(config_models) = &config.models {
            for m in config_models {
                if !final_text_models.contains(m) {
                    final_text_models.push(m.clone());
                }
            }
        }
        
        // Handle image models from config
        if let Some(config_image_models) = &config.image_models {
            for m in config_image_models {
                if !final_image_models.contains(m) {
                    final_image_models.push(m.clone());
                }
            }
        }

        for m in discovered_models {
            match m.model_type {
                ModelType::Text => {
                    if !final_text_models.contains(&m.name) {
                        final_text_models.push(m.name);
                    }
                },
                ModelType::Image => {
                    if !final_image_models.contains(&m.name) {
                        final_image_models.push(m.name);
                    }
                }
            }
        }

        let mut client = LlmClient {
            config,
            text_models: final_text_models,
            text_model_idx: 0,
            image_models: final_image_models,
            image_model_idx: 0,
            agent: None,
        };

        client.update_agent();
        client
    }

    fn update_agent(&mut self) {
        if self.text_models.is_empty() {
            self.agent = None;
            return;
        }

        let model_full_name = &self.text_models[self.text_model_idx];
        let parts: Vec<&str> = model_full_name.split('/').collect();
        
        if parts.len() < 2 {
            warn!("Invalid model name format: {}", model_full_name);
            self.agent = None;
            return;
        }

        let provider = parts[0];
        let model_name = parts[1];

        match provider {
            "openai" => {
                let rig_client = OpenAIClient::from_env();
                let agent_instance = rig_client.agent(model_name).build();
                self.agent = Some(Box::new(agent_instance));
            },
            "gemini" => {
                let rig_client = GeminiClient::from_env();
                let agent_instance = rig_client.agent(model_name).build();
                self.agent = Some(Box::new(agent_instance));
            },
            "groq" => {
                // For Groq, the user might have set GROQ_API_KEY.
                // If Rig's OpenAIClient uses OPENAI_API_KEY, we might need to swap it.
                // However, let's try to just use from_env and assume it picks up the right ones
                // if we set them in the LlmClient::new.
                let rig_client = OpenAIClient::from_env();
                let agent_instance = rig_client.agent(model_name).build();
                self.agent = Some(Box::new(agent_instance));
            },
            _ => {
                warn!("Unsupported provider for chat: {}", provider);
                self.agent = None;
            }
        }
    }

    pub fn model(&self) -> String {
        self.text_models.get(self.text_model_idx).cloned().unwrap_or_else(|| "none".to_string())
    }

    pub fn move_to_next_model(&mut self) {
        if self.text_models.is_empty() {
            return;
        }
        self.text_model_idx = (self.text_model_idx + 1) % self.text_models.len();
        self.update_agent();
    }

    pub async fn chat(&mut self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;
        let start_idx = self.text_model_idx;

        loop {
            if let Some(agent) = &self.agent {
                let messages = vec![ChatCompletionMessage::user(prompt.to_string())];
                info!("Trying model: {}", self.model());
                match agent.perform_chat_completion(messages).await {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        warn!("Model {} failed: {}", self.model(), e);
                        last_error = Some(e);
                    }
                }
            } else {
                warn!("No agent available for model: {}", self.model());
            }

            self.move_to_next_model();
            if self.text_model_idx == start_idx {
                break;
            }
        }

        Err(last_error.unwrap_or_else(|| "No models available or all failed".into()))
    }

    // Based on https://github.com/0xPlaygrounds/rig/blob/main/rig/rig-core/examples/huggingface_image_generation.rs
    pub async fn generate_image(&mut self, prompt: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        if self.image_models.is_empty() {
            return Err("No image models discovered".into());
        }

        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;
        let start_idx = self.image_model_idx;

        loop {
            let model_full_name = &self.image_models[self.image_model_idx];
            info!("Trying image model: {}", model_full_name);
            
            let parts: Vec<&str> = model_full_name.split('/').collect();
            if parts.len() >= 2 {
                let provider = parts[0];
                let model_name = parts[1..].join("/");

                let result: Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> = match provider {
                    "openai" => {
                        let client = OpenAIClient::from_env();
                        let model = client.image_generation_model(&model_name);
                        model.image_generation_request().prompt(prompt).send().await
                            .map(|resp| resp.image)
                            .map_err(|e| e.into())
                    },
                    "hf" => {
                        let client = HFClient::from_env();
                        let model = client.image_generation_model(&model_name);

                        // let response = model
                        //     .image_generation_request()
                        //     .prompt("A castle sitting upon a large mountain, overlooking the water.")
                        //     .width(1024)
                        //     .height(1024)
                        //     .send()
                        //     .await
                        //     .expect("Failed to generate image");

                        // Ok(response.image)
                        model.image_generation_request().prompt(prompt).send().await
                            .map(|resp| resp.image)
                            .map_err(|e| e.into())
                    },
                    _ => Err(format!("Unsupported image provider: {}", provider).into())
                };

                match result {
                    Ok(image_bytes) => return Ok(image_bytes),
                    Err(e) => {
                        warn!("Image model {} failed: {}", model_full_name, e);
                        last_error = Some(e);
                    }
                }
            }

            self.image_model_idx = (self.image_model_idx + 1) % self.image_models.len();
            if self.image_model_idx == start_idx {
                break;
            }
        }

        Err(last_error.unwrap_or_else(|| "All image providers failed".into()))
    }
}

impl fmt::Display for LlmClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "model: {}", self.model())
    }
}