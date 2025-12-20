use crate::services::command_context::CommandContext;
use tracing::info;

pub async fn join(context: CommandContext) -> Result<(), ()> {
    let arg = context.clean_text();

    // Parse comma-separated list of display names
    let mut names: Vec<String> = arg.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // If no names provided, add the sender themselves
    if names.is_empty() {
        let sender_player_member = context.sender_room_member().await?;
        let sender_display_name = sender_player_member.display_name().unwrap().to_owned();
        names.push(sender_display_name);
    }

    // Resolve display names to room member user IDs
    let room_members = context.all_player_room_members().await;

    let mut to_add: Vec<(String, String)> = Vec::new(); // (user_id, display_name)
    let mut not_found_display_names: Vec<String> = Vec::new();
    let mut added_display_names: Vec<String> = Vec::new();
    let mut existing_display_names: Vec<String> = Vec::new();

    for name in names {
        let found = room_members.iter().find(|m| {
            if let Some(dn) = m.display_name() { dn.to_lowercase() == name.to_lowercase() } else { false }
        });

        if let Some(member) = found {
            let display_name = member.display_name().unwrap().to_string();
            to_add.push((member.user_id().to_string(), display_name));
        } else {
            not_found_display_names.push(name);
        }
    }

    // Update room state once, adding all resolved user IDs
    let added_user_ids: Vec<String> = context.update_room_state(|state| {
        let mut added = Vec::new();
        for (user_id, display_name) in &to_add {
            if !state.lobby.contains(user_id) {
                state.lobby.push(user_id.clone());
                added.push(user_id.clone());
                added_display_names.push(display_name.to_string());
            } else {
                existing_display_names.push(display_name.to_string());
            }
        }
        Ok(added)
    }).await?;

    // Inform about results
    let mut msg = String::new();
    if !added_display_names.is_empty() {
        let names_str = added_display_names.join(", ");
        msg = format!("{} have joined the party.", names_str);   
    }

    if !existing_display_names.is_empty() {
        let names_str = existing_display_names.join(", ");
        msg = format!("{}\n{} is/are already in the party.", msg, names_str);
    }

    if !not_found_display_names.is_empty() {
        let names_str = not_found_display_names.join(", ");
        msg = format!("{}\n{} could not be found in the room.", msg, names_str);
    }

    context.room_send(msg.as_str()).await.unwrap();
    Ok(())
}
