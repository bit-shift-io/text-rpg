//use bevy_ecs::{entity::Entity, system::{Commands, Query, SystemState}};
//use bevy_reflect::Reflect;
use tracing::{error, info};
use matrix_sdk::{
    media::{MediaFileHandle, MediaFormat, MediaRequest},
    room::{MessagesOptions, RoomMember},
    ruma::{
        api::client::membership::joined_members, events::room::message::{MessageType, RoomMessageEventContent}, OwnedUserId
    },
    Room as MatrixRoom, RoomMemberships,
};
use serde::{de::{DeserializeOwned, IntoDeserializer}, Deserialize, Serialize};
use serde_diff::{Apply, Diff, SerdeDiff};
use regex::Regex;

use crate::{commands::monster_act::monster_act, components::game_info_container::GameInfo, get_ai_chat, lib::extract_json_from_response::extract_json_from_response};
use crate::globals::*;

pub struct CommandContext {
    pub sender: OwnedUserId,
    pub text: String,
    pub room: MatrixRoom,
    pub verbose: bool,
}

impl CommandContext {
    pub fn new(sender: OwnedUserId, text: String, room: MatrixRoom) -> Self {
        let verbose = text.contains("verbose");
        Self {
            sender,
            text,
            room,
            verbose
        }
    }

    // Are we allowed to respond to this command?
    pub fn handles_room(&self) -> bool {
        let room_name = self.room.name().unwrap_or_default();
        let config = GLOBAL_CONFIG.lock().unwrap().clone().unwrap();

        match config.rooms {
            Some(rooms) => rooms.iter().any(|room| *room == room_name),
            None => true,
        }
    }

    pub async fn notify_typing(&self) {
        self.room.typing_notice(true).await.unwrap();
    }

    pub async fn sender_room_member(&self) -> Result<RoomMember, ()> {
        let joined_members = match self.room.members(RoomMemberships::JOIN).await {
            Ok(members) => members,
            Err(e) => {
                error!("Error fetching joined members: {}", e);
                return Err(());
            }
        };
        let member_idx = joined_members.iter().position(|player_member| player_member.user_id() == self.sender).unwrap();
        let acting_player_member = &joined_members[member_idx];
        Ok(acting_player_member.clone())
    }

    pub async fn all_player_room_members(&self) -> Vec<RoomMember> {
        let joined_members = self.room.members(RoomMemberships::JOIN).await.unwrap();
        let player_members: Vec<RoomMember> = joined_members.into_iter().filter(|member| !member.is_account_user()).collect();
        player_members
    }

    pub async fn room_send(&self, msg: &str) -> Result<String, ()> {
        match self.room.send(RoomMessageEventContent::notice_plain(msg.clone())).await {
            Ok(response) => Ok("".to_string()),
            Err(e) => {
                error!("Error sending message: {}", e);
                Err(())
            }
        }
    }

    pub async fn clone_game_info(&self) -> Result<GameInfo, ()> {
        // todo: https://stackoverflow.com/questions/68976937/rust-future-cannot-be-sent-between-threads-safely
        // need to put something in the chat to say to start the game!

        let mutex_guard = GLOBAL_GAME_INFO.lock().unwrap();
        let game_info_option = mutex_guard.as_ref();

        if game_info_option.is_none() {
            error!("No game in progress. Please run \"DM start\".");
            //self.room_send("No game in progress. Please run \"DM start\".").await?;
        }

        match game_info_option {
            Some(game_info) => Ok(game_info.clone()),
            None => Err(())
        }
    }

    pub async fn execute_story_prompt(&self, prompt: String) -> Result<String, ()> {
        self.notify_typing().await;

        let r = get_ai_chat().execute(&None, prompt, Vec::new());
        match r {
            Ok(result) => {
                self.notify_typing().await;

                info!( "[execute_story_prompt] result: {}", result);
                self.room.send(RoomMessageEventContent::notice_plain(result.clone())).await.unwrap();

                Ok(result)
            },
            Err(e) => {
                error!("[execute_story_prompt] Failed to execute prompt: {}", e);
                self.room.send(RoomMessageEventContent::notice_plain(format!("[execute_story_prompt] Failed to execute prompt: {}", e))).await.unwrap();
                Err(())
            }
        }
    }

    pub async fn execute_json_prompt<T: DeserializeOwned + Clone>(&self, prompt: String) -> Result<T, ()> {
        self.notify_typing().await;

        let r = get_ai_chat().execute(&None, prompt, Vec::new());
        match r {
            Ok(result) => {
                self.notify_typing().await;

                info!( "[execute_json_prompt] result: {}", result);

                if self.verbose {
                    self.room.send(RoomMessageEventContent::notice_plain(result.clone())).await.unwrap();
                }
                
                let json_strs = extract_json_from_response(&result);
                if json_strs.len() == 0 {
                    self.room.send(RoomMessageEventContent::notice_plain("[execute_json_prompt] Failed to get JSON from response.")).await.unwrap();
                }

                let owned_string = json_strs[0].to_string(); // Create an owned copy
                let r = match serde_json::from_str::<T>(&owned_string) {
                    Ok(obj) => {
                        let c = obj.clone();
                        Ok(c)
                    },
                    Err(err) => {
                        error!("Error parsing json: {err}");
                        self.room.send(RoomMessageEventContent::notice_plain("[execute_json_prompt] Failed to parse the map info.")).await.unwrap();
                        //Err("Failed to execute AI chat.".to_string())
                        Err(())
                    }
                };
                r
            },
            Err(e) => {
                error!("[execute_json_prompt] Failed to execute prompt: {}", e);
                self.room.send(RoomMessageEventContent::notice_plain(format!("[execute_json_prompt] Failed to execute prompt: {}", e))).await.unwrap();
                Err(())
            }
        }
    }
}