use bevy_ecs::{entity::Entity, system::{Commands, Query, SystemState}};
use bevy_reflect::Reflect;
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

use crate::{commands::monster_act::monster_act, components::{game_info_container::{GameInfo, GameInfoContainer}, health::Health, inventory::Inventory, item::Item, monster::Monster, player_character::PlayerCharacter, room_connection::RoomConnection, room_location::RoomLocation}, get_ai_chat, lib::extract_json_from_response::extract_json_from_response};
use crate::globals::*;
use crate::components::room::Room;

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

    pub async fn notify_typing(&self) {
        self.room.typing_notice(true).await.unwrap();
    }

    pub async fn all_player_room_members(&self) -> Vec<RoomMember> {
        let joined_members = self.room.members(RoomMemberships::JOIN).await.unwrap();
        let player_members: Vec<RoomMember> = joined_members.into_iter().filter(|member| !member.is_account_user()).collect();
        player_members
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
                error!("Error executing AI chat: {}", e);

                if self.verbose {
                    self.room.send(RoomMessageEventContent::notice_plain("[execute_story_prompt] Failed to execute prompt.")).await.unwrap();
                }

                //Err("Failed to execute AI chat.".to_string())
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

                /* 
                //let value = 
                match serde_json::from_str::<T>(&owned_string) {
                    Ok(obj) => {
                        /* 
                        let json_diff = serde_json::to_string(&Diff::serializable(&new_game_state, &game_info)).unwrap();
                        info!("GAME_INFO DIFF: {}", json_diff.replace('\n', " "));
        
                        if self.verbose {
                            self.room.send(RoomMessageEventContent::notice_plain(json_diff)).await.unwrap();
                        }*/

                        Ok(obj)
                    },
                    Err(err) => {
                        error!("Error parsing json: {err}");
                        self.room.send(RoomMessageEventContent::notice_plain("[execute_json_prompt] Failed to parse the map info.")).await.unwrap();
                        Err("Failed to execute AI chat.".to_string())
                    }
                }*/

                //Err("NO valid JSON response.".to_string())
            },
            Err(e) => {
                error!("Error executing AI chat: {}", e);

                if self.verbose {
                    self.room.send(RoomMessageEventContent::notice_plain("[execute_story_prompt] Failed to execute prompt.")).await.unwrap();
                }

                //Err("Failed to execute AI chat.".to_string())
                Err(())
            }
        }
    }
}