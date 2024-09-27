use matrix_sdk::{
    media::{MediaFileHandle, MediaFormat, MediaRequest},
    room::MessagesOptions,
    ruma::{
        events::room::message::{MessageType, RoomMessageEventContent},
        OwnedUserId,
    },
    Room, RoomMemberships,
};
use tracing::info;

use crate::get_ai_chat;


pub async fn start_a_new_game(sender: OwnedUserId, text: String, room: Room) -> Result<(), ()> {
    room.send(RoomMessageEventContent::notice_plain("Starting a new game. Let me think about for a bit.")).await.unwrap();

    let prompt = r#"
    I am a dungeon master. I need to create a setup for the game.

    I need a list of each room. 
        In each room I need a list of monsters. 
        In each room I need a list of items.
    I need a list of room connections.
    I need a list of monsters with any special abilities they might have.

    Each room should be described with the following structure:

        Room number: The room number.
        Name: The name of the room.
        Description: The description of the room.
        Monsters: A comma separated list of monsters in the room.
        Items: A comma separated list of items in the room.

    Each room connection should be described with the following structure:

        Room Connections: A list of connected room numbers.

    Each monstor should be described with the following structure:

        Monstor name: The name of a monster.
        Description: A short description of the monster.
        Abilities: A comma separated list of special abilties the monster has.


    I need the response in JSON format. "rooms" should be an array with the following structure:
    {
        "roomNumber": {The room number},
        "name": {The name of the room},
        "description": {The description of the room},
        "monsters": [A comma separated list of monsters in the room],
        "items": [A comma separated list of items in the room]
    }

    "monsters" should be an array with the following structure:
    {
        "name": {The name of a monster},
        "description": {A short description of the monster},
        "abilities": [A comma separated list of special abilties the monster has]
    }

    "roomConnections" should be an arry with the following structure:
    {
        "roomNumber": 1,
        "connectedRooms": [A list of connected room numbers]
    }
    "#;

    if let Ok(result) = get_ai_chat().execute(&None, prompt.to_string(), Vec::new()) {
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

    room.send(RoomMessageEventContent::notice_plain("Okay, a new game is ready. Let's begin.")).await.unwrap();
    Ok(())
}