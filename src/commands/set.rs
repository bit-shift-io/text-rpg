use crate::services::command_context::CommandContext;
use crate::globals::GLOBAL_SETTINGS;
use tracing::info;

pub async fn set(context: CommandContext) -> Result<(), ()> {
    let clean_input = context.clean_text();
    let args: Vec<&str> = clean_input.split_whitespace().collect();
    if args.len() < 2 {
        context.room_send("Usage: .set <key> <value>").await.unwrap();
        return Ok(());
    }

    let key = args[0];
    let value = args[1];

    let mut settings = GLOBAL_SETTINGS.lock().await;
    match settings.set(key, value) {
        Ok(msg) => {
            context.room_send(&msg).await.unwrap();
            info!("Updated setting: {} = {}", key, value);
        },
        Err(err) => {
            context.room_send(&format!("Error: {}", err)).await.unwrap();
        }
    }

    Ok(())
}
