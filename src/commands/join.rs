use crate::services::command_context::CommandContext;
use tracing::info;

pub async fn join(context: CommandContext) -> Result<(), ()> {
    let sender = context.sender.to_string();
    
    let joined = context.update_room_state(|state| {
        if state.lobby.contains(&sender) {
            Ok(false)
        } else {
            state.lobby.push(sender);
            Ok(true)
        }
    }).await?;

    if joined {
        context.room_send(&format!("{} has joined the party.", context.sender.localpart())).await.unwrap();
        info!("Player joined: {}", context.sender);
    } else {
        context.room_send(&format!("{} is already in the adventure party.", context.sender.localpart())).await.unwrap();
    }

    Ok(())
}
