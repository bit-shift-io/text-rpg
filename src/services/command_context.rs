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

use crate::{llm_client::LlmClient};
use crate::globals::*;

use super::game_info::{GameInfo, PlayerCharacterInfo};

pub struct CommandContext {
    pub command: String,
    pub sender: OwnedUserId,
    pub text: String,
    pub room: MatrixRoom,
    pub verbose: bool,
}

impl CommandContext {
    pub fn new(command: String, sender: OwnedUserId, text: String, room: MatrixRoom) -> Self {
        let verbose = text.contains("verbose");
        Self {
            command,
            sender,
            text,
            room,
            verbose
        }
    }

    /// Text with any command + special arguments removed
    pub fn clean_text(&self) -> String {
        self.text
            .replace(&self.command, "")
            .replace("verbose", "")
            .trim()
            .to_string()
    }

    // Are we allowed to respond to this command?
    pub async fn handles_room(&self) -> bool {
        let room_name = self.room.name().unwrap_or_default();
        let config = GLOBAL_CONFIG.lock().await.clone().unwrap();

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

    pub async fn bot_display_name(&self) -> Result<String, ()> {
        let joined_members = match self.room.members(RoomMemberships::JOIN).await {
            Ok(members) => members,
            Err(e) => {
                error!("Error fetching joined members: {}", e);
                return Err(());
            }
        };
        // The bot is the account user
        let bot_member = joined_members.iter().find(|member| member.is_account_user()).ok_or(())?;
        Ok(bot_member.display_name().unwrap_or("Dungeon Master").to_string())
    }

    pub async fn find_room_member_player_character_info(&self, room_member: RoomMember) -> Option<PlayerCharacterInfo> {
        let room_member_display_name = room_member.display_name().unwrap();//.to_string();

        let r = self.with_game_info(|game_info| {
            let f = game_info.player_characters.iter().find(|&player_character| player_character.name.eq(&room_member_display_name));
            Ok(f.unwrap().clone())
        }).await.unwrap();
        Some(r)
    }

    pub async fn all_player_room_members(&self) -> Vec<RoomMember> {
        let joined_members = self.room.members(RoomMemberships::JOIN).await.unwrap();
        let player_members: Vec<RoomMember> = joined_members.into_iter().filter(|member| !member.is_account_user()).collect();
        player_members
    }

    pub async fn room_send(&self, msg: &str) -> Result<String, ()> {
        match self.room.send(RoomMessageEventContent::notice_markdown(msg)).await {
            Ok(response) => Ok("".to_string()),
            Err(e) => {
                error!("Error sending message: {}", e);
                Err(())
            }
        }
    }

    pub async fn with_room_state<T, F>(&self, callback: F) -> Result<T, ()>
    where
        F: FnOnce(&RoomState) -> Result<T, ()>,
    {
        let room_id = self.room.room_id().to_string();
        let states = GLOBAL_ROOM_STATES.lock().await;
        let state = states.get(&room_id).ok_or(())?;
        callback(state)
    }

    pub async fn update_room_state<T, F>(&self, callback: F) -> Result<T, ()>
    where
        F: FnOnce(&mut RoomState) -> Result<T, ()>,
    {
        let room_id = self.room.room_id().to_string();
        let mut states = GLOBAL_ROOM_STATES.lock().await;
        let state = states.entry(room_id.clone()).or_insert_with(|| RoomState {
            room_id: room_id.clone(),
            ..Default::default()
        });
        
        let result = callback(state);
        if result.is_ok() {
            save_room_state(&room_id, state).await;
        }
        result
    }

    pub async fn with_game_info<T>(&self, callback: impl FnOnce(&GameInfo) -> Result<T, ()>) -> Result<T, ()> {
        let room_id = self.room.room_id().to_string();
        let states = GLOBAL_ROOM_STATES.lock().await;
        
        let state = match states.get(&room_id) {
            Some(s) => s,
            None => {
                let _ = self.room_send("No game in progress (lobby is empty).").await;
                return Err(());
            }
        };

        match state.game_info.as_ref() {
            Some(game_info) => callback(game_info),
            None => {
                let _ = self.room_send("No game in progress in this room.").await;
                Err(())
            }
        }
    }

    pub async fn game_info_as_json(&self) -> Result<String, ()> {
        self.with_game_info(|game_info| {
            game_info.to_json_string()
        }).await
    }

    pub async fn execute_prompt(&self, prompt: String) -> Result<String, ()> {
        self.notify_typing().await;
        info!("[execute_prompt] prompt: {}", prompt);

        //let config = GLOBAL_CONFIG.lock().await.clone().unwrap();
        let mut client_mut = GLOBAL_LLM_CLIENT.lock().await; //.as_ref();//.unwrap();
        let client = client_mut.as_mut().unwrap();
        
        let r2 = client.chat(&prompt).await;
        match r2 {
            Ok(result) => {
                info!("[execute_prompt] result: {}", result);
                Ok(result)
            },
            Err(e) => {
                error!("[execute_prompt] Failed to execute prompt: {}", e);
                self.room.send(RoomMessageEventContent::notice_plain(format!("[execute_prompt] Failed to execute prompt: {}", e))).await.unwrap();
                Err(())
            }
        }
    }
}