use crate::services::command_context::CommandContext;
use tracing::info;

pub async fn leave(context: CommandContext) -> Result<(), ()> {
    let sender = context.sender.to_string();
    
    let left = context.update_room_state(|state| {
        if let Some(pos) = state.lobby.iter().position(|x| *x == sender) {
            state.lobby.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }).await?;

    if left {
        context.room_send(&format!("{} has left the party.", context.sender.localpart())).await.unwrap();
        info!("Player left: {}", context.sender);
    } else {
        context.room_send(&format!("{} is not currently in the party.", context.sender.localpart())).await.unwrap();
    }

    Ok(())
}
