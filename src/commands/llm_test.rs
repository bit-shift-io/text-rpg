use crate::services::command_context::CommandContext;
use crate::globals::GLOBAL_CONFIG;
use crate::llm_client::LlmClient;
use tracing::{info, error};

pub async fn llm_test(context: CommandContext) -> Result<(), ()> {

    let config = GLOBAL_CONFIG.lock().await.clone().unwrap();
    let client = LlmClient::new(config);
    let prompt = "Say hello from Rust LLM integration!";
    match client.chat(prompt).await {
        Ok(response) => {
            context.room_send(&format!("LLM Response: {}", response)).await.unwrap();
        },
        Err(e) => {
            error!("LLM Chat Error: {}", e);
            context.room_send(&format!("Error: {}", e)).await.unwrap();
        }
    }

    
    Ok(())
}
