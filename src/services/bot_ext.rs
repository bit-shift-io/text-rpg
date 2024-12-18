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

                let context = CommandContext::new(event.sender.clone(), body.to_string(), room);
                if !context.handles_room().await {
                    return;
                }

                context.notify_typing().await;

                if let Err(e) = callback(context).await {
                    error!("Error responding to: {}\nError: {:?}", body, e);
                }
            },
        );
    }
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