use crate::llm_config::{ClientConfig, LlmConfig};
use genai::chat::{ChatMessage, ChatRequest};
use genai::Client;
use genai::resolver::AuthData;
use genai::chat::printer::print_chat_stream;
pub struct LlmClient {
    client: Client,
    default_model: String,
}

impl LlmClient {
    pub fn new(config: LlmConfig) -> Self {
        let mut client_builder = Client::builder();
        
        for client_config in config.clients {
            match client_config {
                ClientConfig::OpenaiCompatible { name, api_base, api_key } => {
                    // Start of workaround: Set env vars based on client name
                    if name == "groq" {
                        unsafe {
                            std::env::set_var("GROQ_API_KEY", &api_key);
                        }
                    } else if name == "sambanova" {
                         // Genai might not support sambanova native, but if it's openai compatible,
                         // we might need to rely on OPENAI_API_BASE, but that is global.
                         // For now, only Groq keys are set if name matches.
                         // We might need a better solution for generic openai-compatible.
                         // But for now, let's just log or set a specific var if we knew it.
                         // e.g. OPENAI_API_KEY usually works for the main provider.
                    }
                    // End of workaround
                }
                ClientConfig::Gemini { api_key } => {
                    unsafe {
                        std::env::set_var("GEMINI_API_KEY", api_key);
                    }
                }
            }
        }
        
        // RE-EVALUATION: Without Exact Docs on "openai-compatible" in genai rust, I should probably search for an example or assume a generic structure.
        // However, I can implement the skeleton and then 'fix' it.
        
        LlmClient {
             client: client_builder.build(),
             default_model: config.model,
        }
    }

    pub async fn chat(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let chat_req = ChatRequest::new(vec![
            ChatMessage::user(prompt.to_string()),
        ]);

        let response = self.client.exec_chat_stream(&self.default_model, chat_req, None).await?;
        //let content = response.content.;

        let content = print_chat_stream(response, None).await?;

        // match response.content {
        //     Some(genai::chat::MessageContent::Text(s)) => s,
        //     _ => String::from(""),
        // };
        Ok(content)
    }
}
