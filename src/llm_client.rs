use crate::config::Config;
use crate::discovery::list_gemini_models;
use rig::providers::openai::Client as OpenAIClient;
use rig::providers::gemini::Client as GeminiClient;
use rig::providers::openai::Client as GroqClient; // Rig uses OpenAI client for Groq too
use rig::completion::message::Message as ChatCompletionMessage;
use rig::providers::openai::responses_api::Role;
use rig::client::{ProviderClient, CompletionClient};
use rig::completion::{CompletionModel, Completion}; // CompletionModel for trait bounds, Completion for the trait itself
use rig::OneOrMany; // Import OneOrMany
use rig::agent::Agent;
use async_trait::async_trait;
use tracing::{info, warn, error};
use std::fmt;
use crate::discovery::list_all_models;

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
    models: Vec<String>,
    model_idx: usize,
    agent: Option<Box<dyn AbstractAgent>>,
}

impl LlmClient {
    pub async fn new(config: Config) -> Self {
        if config.gemini_api_key.is_some() {
            unsafe {
                std::env::set_var("GEMINI_API_KEY", config.gemini_api_key.as_ref().unwrap());
            }
        }

        if config.groq_api_key.is_some() {
            unsafe {
                std::env::set_var("GROQ_API_KEY", config.groq_api_key.as_ref().unwrap());
            }
        }

        if config.openai_api_key.is_some() {
            unsafe {
                std::env::set_var("OPENAI_API_KEY", config.openai_api_key.as_ref().unwrap());
            }
        }

        let mut discovered_models = list_all_models(config.clone()).await;
        
        // If config has specific models, prepend them to the list or use them as primary
        let mut final_models = Vec::new();
        if let Some(config_models) = &config.models {
            for m in config_models {
                if !final_models.contains(m) {
                    final_models.push(m.clone());
                }
            }
        }
        
        for m in discovered_models {
            if !final_models.contains(&m) {
                final_models.push(m);
            }
        }

        let mut client = LlmClient {
            config,
            models: final_models,
            model_idx: 0,
            agent: None,
        };

        client.update_agent();
        client
    }

    fn update_agent(&mut self) {
        if self.models.is_empty() {
            self.agent = None;
            return;
        }

        let model_full_name = &self.models[self.model_idx];
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
            // "groq" => {
            //     // Groq uses OpenAI protocol but different base URL and env var
            //     // Rig might have specific Groq support or we use OpenAI with custom config
            //     // For now assuming Rig has a way to handle Groq if we set the env var
            //     // and use the right client if they provided one, but based on docs
            //     // it might just be another provider.
            //     // Let's check if Rig has a Groq provider. If not, we might need more config.
            //     // Re-using OpenAIClient for now as a placeholder or if it's compatible.
            //     let rig_client = GroqClient::new(
            //         &self.config.groq_api_key.as_ref().unwrap(),
            //         "https://api.groq.com/openai/v1"
            //     );
            //     let agent_instance = rig_client.agent(model_name).build();
            //     self.agent = Some(Box::new(agent_instance));
            // },
            _ => {
                warn!("Unsupported provider: {}", provider);
                self.agent = None;
            }
        }
    }

    pub fn model(&self) -> String {
        self.models.get(self.model_idx).cloned().unwrap_or_else(|| "none".to_string())
    }

    pub fn move_to_next_model(&mut self) {
        if self.models.is_empty() {
            return;
        }
        self.model_idx = (self.model_idx + 1) % self.models.len();
        self.update_agent();
    }

    pub async fn chat(&mut self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;
        let start_idx = self.model_idx;

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
            if self.model_idx == start_idx {
                break;
            }
        }

        Err(last_error.unwrap_or_else(|| "No models available or all failed".into()))
    }
}

impl fmt::Display for LlmClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "model: {}", self.model())
    }
}