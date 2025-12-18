use crate::services::command_context::CommandContext;
use crate::globals::GLOBAL_LOBBY;
use tracing::info;

pub async fn leave(context: CommandContext) -> Result<(), ()> {
    let sender = context.sender.to_string();
    let mut lobby = GLOBAL_LOBBY.lock().await;

    if let Some(pos) = lobby.iter().position(|x| *x == sender) {
        lobby.remove(pos);
        context.room_send(&format!("{} has left the party.", context.sender.localpart())).await.unwrap();
        info!("Player left: {}", context.sender);
    } else {
        context.room_send(&format!("{} is not currently in the party.", context.sender.localpart())).await.unwrap();
    }

    Ok(())
}
