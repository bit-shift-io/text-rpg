use std::{fmt, fs::{self, File}};
use std::{collections::HashMap, io::Read, path::PathBuf, sync::Mutex};

use headjack::*;
//use kalosm::{language::{Chat, Llama}, sound::TextStream};
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
use lazy_static::lazy_static;

mod aichat;
use aichat::AiChat;

mod start_a_new_game;
use start_a_new_game::start_a_new_game;

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

lazy_static! {
    /// Holds the config for the bot
    static ref GLOBAL_CONFIG: Mutex<Option<Config>> = Mutex::new(None);

    /// Count of the global messages per user
    static ref GLOBAL_MESSAGES: Mutex<HashMap<String, u64>> = Mutex::new(HashMap::new());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> { //anyhow::Error> {
    tracing_subscriber::fmt::init();

    let mut file_contents = fs::read_to_string("config.yml").expect("Unable to read config.yml");
    let config: Config = serde_yml::from_str(&file_contents).unwrap();
    *GLOBAL_CONFIG.lock().unwrap() = Some(config.clone());

    //println!("config: {}", config);

    /// Returns the backend based on the global config
    fn get_backend() -> AiChat {
        let config = GLOBAL_CONFIG.lock().unwrap().clone().unwrap();
        AiChat::new("aichat".to_string(), None) //Some("/Users/fabian/Library/Application Support/aichat/".to_owned())) //config.aichat_config_dir.clone())
    }

    // see example usage here on how to load from config: https://github.com/arcuru/chaz/blob/main/src/main.rs
    let bot_config = BotConfig {
        login: Login {
            homeserver_url: "https://matrix.org".to_string(),
            username: config.username,
            password: Some(config.password),
        },
        allow_list: Some("(.*)".to_owned()),
        state_dir: Some("~/text-rpg".to_string()),
        command_prefix: Some("dm".to_string()),
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
    let model = Llama::new_chat().await.unwrap();

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
        "start",
        "".to_string(),
        "Start / restart a new game".to_string(),
        start_a_new_game,
    )
    .await;
 
    bot.register_text_command(
        "ask",
        "".to_string(),
        "Ask a question".to_string(),
        |sender, body, room| async move {

            // Skip over the command, which is "!chaz ask"
            let input = body
                .split_whitespace()
                .skip(1)
                .collect::<Vec<&str>>()
                .join(" ");
            /* *
            let model = Llama::new_chat().await.unwrap();

            // construct LLM chat model
            let mut chat = Chat::builder(model)
                .with_system_prompt("The assistant will act like a pirate")
                .build();

            //let response = format!("You asked: {}", body); 
            let response = chat.add_message(body).all_text().await;
            */


            //let model = Some("gemini".to_owned()); // todo: move to config

            info!(
                "Request: {} - {}",
                sender.as_str(),
                input.replace('\n', " ")
            );
            if let Ok(result) = get_backend().execute(&None, input.to_string(), Vec::new()) {
                // Add the prefix ".response:\n" to the result
                // That way we can identify our own responses and ignore them for context
                info!(
                    "Response: {} - {}",
                    sender.as_str(),
                    result.replace('\n', " ")
                );
                let content = RoomMessageEventContent::notice_plain(result);

                room.send(content).await.unwrap();
            }
            /* 
            let content = RoomMessageEventContent::notice_plain(response);
            room.send(content).await.unwrap();
            */
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