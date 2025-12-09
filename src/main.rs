#![allow(dead_code, unused_variables, unused_imports)]
//#![feature(test)]

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
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::filter::EnvFilter;

mod services;
mod globals;
mod config;
mod commands;
mod themes;
pub mod llm_client;

use globals::*;
use commands::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> { //anyhow::Error> {
    // Setup tracing to only show messages from our crate.
    // https://stackoverflow.com/questions/73247589/how-to-turn-off-tracing-events-emitted-by-other-crates
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::OFF.into())
        .from_env()?
        .add_directive("text_rpg=info".parse()?);
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();

    // first try to read from the docker config location, else fallback to the local dev version
    let file_contents = match fs::read_to_string("/data/config.yml") {
        Ok(contents) => {
            contents
        },
        Err(e) => {
            fs::read_to_string("config.yml").expect("Unable to read config.yml")
        }
    };
    let config: Config = serde_yml::from_str(&file_contents).unwrap();
    
    *GLOBAL_CONFIG.lock().await = Some(config.clone());
    info!("[main] config: {}", config);

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

    bot.register_command(".help", help::help);
    bot.register_command(".start", start::start);
    bot.register_command(".act", act::act);
    bot.register_command(".ask", ask::ask);
    bot.register_command(".dump", dump::dump);
    bot.register_command(".llm", llm_test::llm_test);
    
    // Run the bot, this should never return except on error
    if let Err(e) = bot.run().await {
        error!("Error running bot: {e}");
    }

    Ok(())
}