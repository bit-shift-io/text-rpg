use kalosm::language::*;
use headjack::*;
use matrix_sdk::{
    media::{MediaFileHandle, MediaFormat, MediaRequest},
    room::MessagesOptions,
    ruma::{
        events::room::message::{MessageType, RoomMessageEventContent},
        OwnedUserId,
    },
    Room, RoomMemberships,
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), String> {
    let model = Llama::new_chat().await.unwrap();

    // construct LLM chat model
    let mut chat = Chat::builder(model)
        .with_system_prompt("The assistant will act like a pirate")
        .build();

    // see example usage here on how to load from config: https://github.com/arcuru/chaz/blob/main/src/main.rs
    let bot_config = BotConfig {
        login: Login {
            homeserver_url: "matrix.org".to_string(),
            username: "".to_string(),
            password: Some("".to_string()),
        },
        allow_list: None,
        state_dir: None,
        command_prefix: None,
        room_size_limit: None,
        name: None,
    };
    let mut bot = Bot::new(bot_config).await;

    if let Err(e) = bot.login().await {
        error!("Error logging in: {e}");
    }


    // React to invites.
    // We set this up before the initial sync so that we join rooms
    // even if they were invited before the bot was started.
    bot.join_rooms();

    // Syncs to the current state
    if let Err(e) = bot.sync().await {
        info!("Error syncing: {e}");
    }

    info!("The client is ready! Listening to new messages…");

    // The party command is from the matrix-rust-sdk examples
    // Keeping it as an easter egg
    // TODO: Remove `party` from the help text
    bot.register_text_command(
        "party",
        "".to_string(),
        "Party!".to_string(),
        |_, _, room| async move {
            let content = RoomMessageEventContent::notice_plain(".🎉🎊🥳 let's PARTY!! 🥳🎊🎉");
            room.send(content).await.unwrap();
            Ok(())
        },
    )
    .await;


    bot.register_text_command(
        "ask",
        "".to_string(),
        "Ask a question".to_string(),
        |sender, body, room| async move {

            let response = "qwe"; //chat.add_message(body).all_text().await;
/* 
                .to_std_out()
                .await
                .unwrap();
*/
            let content = RoomMessageEventContent::notice_plain(response);
            room.send(content).await.unwrap();
            Ok(())
        },
    )
    .await;


    // Run the bot, this should never return except on error
    if let Err(e) = bot.run().await {
        error!("Error running bot: {e}");
    }

    /*
    
    
    loop {
        chat.add_message(prompt_input("\n> ").unwrap())
            .to_std_out()
            .await
            .unwrap();
    }
     */

    Ok(())
}