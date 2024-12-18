
//use bevy_ecs::{entity::Entity, query::QueryBuilder, system::{Commands, Query, SystemState}};
use tracing::{error, info};
use matrix_sdk::{
    media::{MediaFileHandle, MediaFormat, MediaRequest},
    room::MessagesOptions,
    ruma::{
        events::room::message::{MessageType, RoomMessageEventContent},
        OwnedUserId,
    },
    Room as MatrixRoom, RoomMemberships,
};
use serde::{Deserialize, Serialize};
use regex::Regex;

use crate::{get_ai_chat, lib::command_context::CommandContext};
use crate::globals::*;

pub async fn dump(context: CommandContext) -> Result<(), ()> {
    let game_info = context.clone_game_info().await?;
    let game_info_json = serde_json::to_string_pretty(&game_info).unwrap();

    context.room.send(RoomMessageEventContent::notice_plain(game_info_json)).await.unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate test;
    use test::Bencher;


    #[test]
    fn test_dump_world() {

    }

}