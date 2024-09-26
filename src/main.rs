use std::{fmt, fs::{self, File}};

use headjack::*;
//use kalosm::language::{Chat, Llama};
use matrix_sdk::{
    media::{MediaFileHandle, MediaFormat, MediaRequest},
    room::MessagesOptions,
    ruma::{
        events::room::message::{MessageType, RoomMessageEventContent},
        OwnedUserId,
    },
    Room, RoomMemberships,
};
use serde::Deserialize;
use tracing::{error, info};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    username: String,
    password: String,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "username: {}, password: {}", self.username, self.password)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> { //anyhow::Error> {
    
    let mut file_contents = fs::read_to_string("config.yml").expect("Unable to read config.yml");
    let config: Config = serde_yml::from_str(&file_contents).unwrap();

    //println!("config: {}", config);

    //let model = Llama::new_chat().await.unwrap();

    // see example usage here on how to load from config: https://github.com/arcuru/chaz/blob/main/src/main.rs
    let bot_config = BotConfig {
        login: Login {
            homeserver_url: "https://matrix.org".to_string(),
            username: config.username,
            password: Some(config.password),
        },
        allow_list: Some("(.*)".to_owned()),
        state_dir: Some("~/text-rpg".to_string()),
        command_prefix: Some("!".to_string()),
        room_size_limit: None,
        name: None,
    };
    let mut bot = Bot::new(bot_config).await;

    if let Err(e) = bot.login().await {
        println!("Error logging in: {e}");
        return Err(e.into()); // Return the error
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
/*
    // construct LLM chat model
    let mut chat = Chat::builder(model)
        .with_system_prompt("The assistant will act like a pirate")
        .build();
*/
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

            let response = format!("You asked: {}", body); //chat.add_message(body).all_text().await;

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
        

    }*/
/* 
    let prompt = prompt_input("\n> ").unwrap();

        let mut response_stream = chat.add_message(prompt);
        // And then stream the result to std out
        response_stream.to_std_out().await.unwrap();
*/
    //Ok(())
    Ok(())
}