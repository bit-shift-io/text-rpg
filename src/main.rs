#![allow(dead_code, unused_variables, unused_imports)]
#![feature(test)]

use std::{fmt, fs::{self, File}};
use std::{collections::HashMap, io::Read, path::PathBuf, sync::Mutex};

use config::Config;
use headjack::*;
use services::{bot_ext::BotExt, command_context::CommandContext};
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

mod services;
mod globals;
mod config;
mod commands;

use globals::*;
use commands::{act::act, dump::dump, help::help, start::start};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> { //anyhow::Error> {
    tracing_subscriber::fmt::init();

    // first try to read from the docker config location, else fallback to the local dev version
    // also assume if we are in docker, then the aichat_config_file can also automatically be configured.
    let mut aichat_config_file: Option<String> = None;
    let file_contents = match fs::read_to_string("/data/config.yml") {
        Ok(contents) => {
            aichat_config_file = Some("/data/aichat.config.yml".to_string());
            contents
        },
        Err(e) => {
            fs::read_to_string("config.yml").expect("Unable to read config.yml")
        }
    };

    let mut config: Config = serde_yml::from_str(&file_contents).unwrap();
    config.aichat_config_file = aichat_config_file;
    
    *GLOBAL_CONFIG.lock().await = Some(config.clone());
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

    bot.register_command(".help", help);
    bot.register_command(".start", start);
    bot.register_command(".act", act);
    bot.register_command(".dump", dump);
    //bot.register_command(".ask", dump_world);

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
            if let Ok(result) = get_ai_chat().await.execute(&None, input.to_string(), Vec::new()) {
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