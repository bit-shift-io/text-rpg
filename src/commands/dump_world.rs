
use bevy_ecs::{entity::Entity, query::QueryBuilder, system::{Commands, Query, SystemState}};
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

use crate::get_ai_chat;
use crate::globals::*;
use crate::components::room::Room;

pub async fn dump_world(sender: OwnedUserId, text: String, room: MatrixRoom) -> Result<(), ()> {
    room.send(RoomMessageEventContent::notice_plain("Dumping world...")).await.unwrap();

    {
        // https://github.com/bevyengine/bevy/discussions/15486
        

        let mut entities = vec![];
        let mut strings = vec![];

        {
            let world_guard = GLOBAL_WORLD.lock().unwrap(); // Error cause by this line.
            let mut world = world_guard;

            // https://doc.qu1x.dev/bevy_trackball/bevy/ecs/system/struct.SystemState.html
            // https://github.com/bevyengine/bevy/issues/2687
            let mut state: SystemState<(
                Commands,
                Query<Entity>,
            )> = SystemState::new(&mut world);

            let (commands, query) = state.get_mut(&mut world);
            
            //let mut query = QueryBuilder::<Entity>::new(&mut world).build();

            for entity in &query { //query.iter(&world) {
                //info!("{:?}", entity);

                entities.push(entity);

                //info!("{:#?}", world.inspect_entity(entity));

                // https://github.com/bevyengine/bevy/discussions/3332
                // https://abadcafe.wordpress.com/2020/12/13/serializing-bevy-ecs-using-reflect-trait/
                /* 
                if let Some(room_component) = world.get_mut::<Room>(entity) {
                    info!("Room component: {:?}", room_component);
                }*/
            }
        }

        {
            let world_guard = GLOBAL_WORLD.lock().unwrap(); // Error cause by this line.
            let world = world_guard;

            for entity in entities {
                let str = format!("{:#?}", world.inspect_entity(entity));
                strings.push(str);
            }
        }

        {
            let str = strings.join("\n");
            info!("{:#?}", str);
            room.send(RoomMessageEventContent::notice_plain(str)).await.unwrap();
        }
        
    }

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