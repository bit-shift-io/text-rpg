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
mod settings;
pub mod llm_client;

use globals::*;
use commands::*;

use crate::llm_client::LlmClient;

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
    // first try to read from the docker config location, then local data folder, else fallback to the local dev version
    let file_contents = match fs::read_to_string("/data/config.yml") {
        Ok(contents) => {
            contents
        },
        Err(e) => {
            match fs::read_to_string("data/config.yml") {
                Ok(contents) => contents,
                Err(e) => {
                     fs::read_to_string("config.yml").expect("Unable to read config.yml from /data, data/ or .")
                }
            }
        }
    };
    let config: Config = serde_yml::from_str(&file_contents).unwrap();
    
    *GLOBAL_CONFIG.lock().await = Some(config.clone());
    info!("[main] config: {}", config);

    let mut llm_client = LlmClient::new(config.clone());
    llm_client.populate_models().await;
    info!("[main] llm: {}", llm_client);
    *GLOBAL_LLM_CLIENT.lock().await = Some(llm_client);

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

    // Load all room states
    load_all_room_states().await;

    // Announce entry in all currently joined rooms
    let announcement = "Hello! I am online and ready to facilitate your text-based RPG adventures.\nUse the **.help** command to see available commands.";
    let content = RoomMessageEventContent::notice_markdown(announcement);
    
    let allowed_rooms = config.rooms.clone().unwrap_or_default();
    
    for room in bot.client().joined_rooms() {
        let room_name = room.name().unwrap_or_else(|| room.room_id().to_string());
        if !allowed_rooms.iter().any(|r| *r == room_name) {
            continue;
        }

        let room_id = room.room_id().to_string();
        let states = GLOBAL_ROOM_STATES.lock().await;
        let game_info_opt = states.get(&room_id).and_then(|s| s.game_info.clone());
        drop(states);

        let room_clone = room.clone();
        let announcement = announcement.to_string();
        
        tokio::spawn(async move {
            info!("Announcing entry in room {}", room_id);
            let mut final_msg = announcement;

            if let Some(game_info) = game_info_opt {
                // Get the bot name from the room or default
                let mut bot_name = "Dungeon Master".to_string();
                if let Ok(members) = room_clone.members(RoomMemberships::JOIN).await {
                    if let Some(m) = members.iter().find(|m| m.is_account_user()) {
                        if let Some(dn) = m.display_name() {
                            bot_name = dn.to_string();
                        }
                    }
                }

                if let Some(summary) = crate::services::summary::generate_story_summary(&game_info, &bot_name).await {
                    final_msg = format!("{}\n\n**The Story So Far:**\n{}", final_msg, summary);
                }
            }

            if let Err(e) = room_clone.send(RoomMessageEventContent::notice_markdown(final_msg)).await {
                error!("Failed to send announcement to room {}: {:?}", room_id, e);
            }
        });
    }

    // Register handler for future room joins
    //bot.announce_on_join();

    bot.register_command(".help", help::help);
    bot.register_command(".start", start::start);
    bot.register_command(".act", act::act);
    bot.register_command(".ask", ask::ask);
    bot.register_command(".dump", dump::dump);
    bot.register_command(".llm", llm_test::llm_test);
    bot.register_command(".end", end::end);
    bot.register_command(".join", join::join);
    bot.register_command(".leave", leave::leave);
    bot.register_command(".set", set::set);

    

    // Run the bot, this should never return except on error
    if let Err(e) = bot.run().await {
        error!("Error running bot: {e}");
    }

    Ok(())
}