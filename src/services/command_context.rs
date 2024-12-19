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

use crate::get_ai_chat;
use crate::globals::*;

use super::game_info::GameInfo;

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

    pub async fn all_player_room_members(&self) -> Vec<RoomMember> {
        let joined_members = self.room.members(RoomMemberships::JOIN).await.unwrap();
        let player_members: Vec<RoomMember> = joined_members.into_iter().filter(|member| !member.is_account_user()).collect();
        player_members
    }

    pub async fn room_send(&self, msg: &str) -> Result<String, ()> {
        match self.room.send(RoomMessageEventContent::notice_plain(msg)).await {
            Ok(response) => Ok("".to_string()),
            Err(e) => {
                error!("Error sending message: {}", e);
                Err(())
            }
        }
    }

    pub async fn with_game_info<T>(&self, callback: impl FnOnce(&GameInfo) -> Result<T, ()>) -> Result<T, ()> {
        let mutex_guard = GLOBAL_GAME_INFO.lock().await;
        let game_info_option = mutex_guard.as_ref();

        if game_info_option.is_none() {
            error!("No game in progress.");
            self.room_send("No game in progress.").await?;
        }

        match game_info_option {
            Some(game_info) => callback(game_info),
            None => Err(())
        }
    }

    pub async fn execute_prompt(&self, prompt: String) -> Result<String, ()> {
        self.notify_typing().await;
        info!("[execute_prompt] prompt: {}", prompt);
        let r = get_ai_chat().await.execute(&None, prompt, Vec::new());
        match r {
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