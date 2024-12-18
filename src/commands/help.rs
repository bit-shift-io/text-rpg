use crate::services::command_context::CommandContext;


const HELP_STR: &str = r#"
.start {optional text feed to modify the game} - Start a new game.
.act {text} - Perform an action.
.ask {text} - Ask a question.

.dump - Dump the game state for debugging.
"#;

pub async fn help(context: CommandContext) -> Result<(), ()> {
    context.room_send(HELP_STR).await.unwrap();
    Ok(())
}