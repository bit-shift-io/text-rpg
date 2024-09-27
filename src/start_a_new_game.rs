use matrix_sdk::{
    media::{MediaFileHandle, MediaFormat, MediaRequest},
    room::MessagesOptions,
    ruma::{
        events::room::message::{MessageType, RoomMessageEventContent},
        OwnedUserId,
    },
    Room, RoomMemberships,
};



pub async fn start_a_new_game(sender: OwnedUserId, text: String, room: Room) -> Result<(), ()> {
    room.send(RoomMessageEventContent::notice_plain("Starting a new game. Let me think about for a bit.")).await.unwrap();

    room.send(RoomMessageEventContent::notice_plain("Okay, a new game is ready. Let's begin.")).await.unwrap();
    Ok(())
}