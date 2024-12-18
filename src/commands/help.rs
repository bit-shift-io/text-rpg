use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

use crate::lib::command_context::CommandContext;


const HELP_STR: &str = r#"
.start {optional text feed to modify the game} - Start a new game.
.act {text} - Perform an action.
.ask {text} - Ask a question.

.dump - Dump the game state for debugging.
"#;

pub async fn help(context: CommandContext) -> Result<(), ()> {
    context.room.send(RoomMessageEventContent::notice_plain(HELP_STR)).await.unwrap();
    Ok(())
}