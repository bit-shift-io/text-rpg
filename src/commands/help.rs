use crate::services::command_context::CommandContext;


const HELP_STR: &str = r#"
**.start** {optional text feed to modify the game} - Start a new game with joined players.
**.join** - Join the adventure party lobby.
**.leave** - Leave the adventure party lobby.
**.act** {text} - Perform an action.
**.ask** {text} - Ask a question.
**.set** {key} {value} - Set a game setting (e.g. .set timeout 30s).
**.end** - End the current game.
**.help** - Show this help message.
**.dump** - Dump the game state for debugging.
"#;

pub async fn help(context: CommandContext) -> Result<(), ()> {
    context.room_send(HELP_STR).await.unwrap();
    Ok(())
}