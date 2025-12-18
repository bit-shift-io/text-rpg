use crate::services::command_context::CommandContext;
use crate::globals::GLOBAL_LOBBY;
use tracing::info;

pub async fn join(context: CommandContext) -> Result<(), ()> {
    let sender = context.sender.to_string();
    let mut lobby = GLOBAL_LOBBY.lock().await;

    if lobby.contains(&sender) {
        context.room_send(&format!("{} is already in the adventure party.", context.sender.localpart())).await.unwrap();
    } else {
        lobby.push(sender);
        context.room_send(&format!("{} has joined the party.", context.sender.localpart())).await.unwrap();
        info!("Player joined: {}", context.sender);
    }

    Ok(())
}
