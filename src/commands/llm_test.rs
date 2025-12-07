use crate::services::command_context::CommandContext;
use crate::globals::GLOBAL_CONFIG;
use crate::llm_config::LlmConfig;
use crate::llm_client::LlmClient;
use tracing::{info, error};

pub async fn llm_test(context: CommandContext) -> Result<(), ()> {
    let config = {
        let guard = GLOBAL_CONFIG.lock().await;
        guard.as_ref().cloned()
    };

    // if let Some(config) = config {
    //     if let Some(path) = config.aichat_config_file {
            let path = "aichat.config.yml";

            info!("Loading LLM config from: {}", path);
            match LlmConfig::load_from_path(&path) {
                Ok(llm_config) => {
                    let client = LlmClient::new(llm_config);
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
                },
                Err(e) => {
                     error!("Failed to load LLM config: {}", e);
                     context.room_send("Failed to load aichat config.").await.unwrap();
                }
            }
    //     } else {
    //         context.room_send("No aichat_config_file configured.").await.unwrap();
    //     }
    // } else {
    //     context.room_send("Global config not loaded.").await.unwrap();
    // }
    
    Ok(())
}
