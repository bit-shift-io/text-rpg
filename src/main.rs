#![allow(dead_code, unused_variables, unused_imports)]
#![feature(test)]

use std::{fmt, fs::{self, File}};
use std::{collections::HashMap, io::Read, path::PathBuf, sync::Mutex};

use config::Config;
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
use serde::Deserialize;
use tracing::{error, info};
use lazy_static::lazy_static;


mod lib {
    pub mod aichat;
    pub mod extract_json_from_response;
}

mod globals;
use globals::*;

mod config;

mod commands {
    pub mod start_a_new_game;
    pub mod dump_world;
    pub mod act;
}

mod components {
    pub mod player_character;
    pub mod player;
    pub mod monster;
    pub mod room_connection;
    pub mod room;
    pub mod room_location;
    pub mod item;
    pub mod inventory;
    pub mod health;
    pub mod game_info_container;
}

use commands::{act::act, dump_world::dump_world, start_a_new_game::start_a_new_game};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> { //anyhow::Error> {
    tracing_subscriber::fmt::init();

    let file_contents = fs::read_to_string("config.yml").expect("Unable to read config.yml");
    let config: Config = serde_yml::from_str(&file_contents).unwrap();
    *GLOBAL_CONFIG.lock().unwrap() = Some(config.clone());

    info!("config: {}", config);


    // see example usage here on how to load from config: https://github.com/arcuru/chaz/blob/main/src/main.rs
    let bot_config = BotConfig {
        login: Login {
            homeserver_url: "https://matrix.org".to_string(),
            username: config.username,
            password: Some(config.password),
        },
        allow_list: Some("(.*)".to_owned()),
        state_dir: Some("~/text-rpg".to_string()),
        command_prefix: Some("DM".to_string()), // can we make it case insensiive?
        room_size_limit: None,
        name: None,
    };
    let mut bot = Bot::new(bot_config).await;

    if let Err(e) = bot.login().await {
        error!("Error logging in: {e}");
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
        "Start a new game".to_string(),
        start_a_new_game,
    )
    .await;

    bot.register_text_command(
        "act",
        "".to_string(),
        "Perform some action".to_string(),
        act,
    )
    .await;

    bot.register_text_command(
        "dump",
        "".to_string(),
        "Dump world".to_string(),
        dump_world,
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

            info!(
                "Request: {} - {}",
                sender.as_str(),
                input.replace('\n', " ")
            );
            if let Ok(result) = get_ai_chat().execute(&None, input.to_string(), Vec::new()) {
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

            Ok(())
        },
    )
    .await;


    // Run the bot, this should never return except on error
    if let Err(e) = bot.run().await {
        error!("Error running bot: {e}");
    }

    Ok(())
}