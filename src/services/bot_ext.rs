use headjack::Bot;
use matrix_sdk::{ruma::{events::room::message::RoomMessageEventContent, OwnedUserId}, Room};
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::RoomState;
use matrix_sdk::ruma::events::room::message::MessageType;
use tracing::{error, info, warn};

use super::command_context::CommandContext;

pub trait BotExt {
    fn register_command<F, Fut>(self: &Self, command: &str, callbck: F) 
    where 
        F: FnOnce(CommandContext) -> Fut + Send + 'static + Clone + Sync,
        Fut: std::future::Future<Output = Result<(), ()>> + Send + 'static;

    //fn announce_on_join(&self);
}

impl BotExt for Bot {

    fn register_command<F, Fut>(&self, command: &str, callback: F) 
    where 
        F: FnOnce(CommandContext) -> Fut + Send + 'static + Clone + Sync,
        Fut: std::future::Future<Output = Result<(), ()>> + Send + 'static,
    {
        let client = self.client();
        let username = self.full_name();
        let command_prefix = self.command_prefix();
        let command = command.to_owned();
        client.add_event_handler(
            move |event: OriginalSyncRoomMessageEvent, room: Room| async move {
                // Ignore messages from rooms we're not in
                if room.state() != RoomState::Joined {
                    return;
                }
                let MessageType::Text(text_content) = &event.content.msgtype else {
                    return;
                };
                if !is_allowed( event.sender.as_str(), &username) {
                    // Sender is not on the allowlist
                    return;
                }
                
                let body = text_content.body.trim_start();
                if !body.starts_with(&command) {
                    return;
                }

                let context = CommandContext::new(command, event.sender.clone(), body.to_string(), room);
                if !context.handles_room().await {
                    return;
                }

                info!("[register_command] Execute command: {:?}, sender: {:?}, text: {:?}.", context.command, event.sender.as_str(), body.to_string());
                context.notify_typing().await;

                if let Err(e) = callback(context).await {
                    error!("Error responding to: {}\nError: {:?}", body, e);
                }
            },
        );
    }

    // fn announce_on_join(&self) {
    //     let client = self.client();
    //     let username = self.client().user_id().unwrap().to_owned();
        
    //     client.add_event_handler(
    //         move |event: matrix_sdk::ruma::events::room::member::OriginalSyncRoomMemberEvent, room: Room| async move {
    //             // Check if it's us joining
    //             if event.state_key != username {
    //                 return;
    //             }

    //             // Check if it is a join event
    //             use matrix_sdk::ruma::events::room::member::MembershipState;
    //             if event.content.membership != MembershipState::Join {
    //                 return;
    //             }

    //             // Check if the room is in our allowlist
    //             let allowed_rooms = {
    //                 let config_guard = crate::GLOBAL_CONFIG.lock().await;
    //                 config_guard.as_ref().and_then(|c| c.rooms.clone()).unwrap_or_default()
    //             };

    //             let room_name = room.name().unwrap_or_default();
    //             if !allowed_rooms.iter().any(|r| *r == room_name) {
    //                 return;
    //             }

    //             // We just joined! Announce ourselves.
    //             let announcement = "Hello! I am online and ready to facilitate your text-based RPG adventures.";
    //             let content = RoomMessageEventContent::text_plain(announcement);
                
    //             if let Err(e) = room.send(content).await {
    //                 error!("Failed to send announcement to room {}: {:?}", room.room_id(), e);
    //             } else {
    //                 info!("Announced entry in room {}", room.room_id());
    //             }
    //         }
    //     );
    // }
}

/// Verify if the sender is on the allow_list
fn is_allowed(sender: &str, username: &str) -> bool {
    // Check to see if it's from ourselves, in which case we should ignore it
    if sender == username {
        false
    } else {
        true
    }
}