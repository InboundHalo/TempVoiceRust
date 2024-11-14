use serenity::all::{
    ChannelId, CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType,
    Context, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, Mentionable,
    Message, User, UserId, VoiceState,
};
use serenity::builder::{CreateCommand, CreateCommandOption, CreateMessage};
use std::collections::HashMap;

use crate::event_handler::cool_down_manager::CooldownManager;

pub fn register() -> CreateCommand {
    CreateCommand::new("invite")
        .description("Invites a user to the voice channel")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "User to invite")
                .required(true),
        )
}

pub async fn run(
    ctx: &Context,
    command: &CommandInteraction,
    cooldown_manager: &CooldownManager,
) -> CreateInteractionResponse {
    let guild_id = match command.guild_id {
        None => return ephemeral_response("This command can only be used in a server."),
        Some(guild_id) => guild_id,
    };

    let invited_user = match get_invited_user(&command) {
        None => return ephemeral_response("You must mention a user to invite."),
        Some(user_id) => user_id,
    };

    // Try to check cooldown before making a request to discord's servers
    let inviter = &command.user;
    let is_command_on_cooldown = !cooldown_manager.can_user_ping_user(&inviter.id, &invited_user);
    if is_command_on_cooldown {
        return ephemeral_response("Please wait as you have already pinged this person!");
    }

    let is_invited_user_bot = invited_user.to_user(&ctx).await.unwrap().bot;
    if is_invited_user_bot {
        return ephemeral_response("You can not invite a bot!");
    }

    let voice_states = &guild_id.to_guild_cached(&ctx).unwrap().voice_states;
    let voice_channel_id = match get_voice_channel_id(voice_states.get(&inviter.id)) {
        None => return ephemeral_response("You must be in a voice channel to use this command."),
        Some(channel_id) => channel_id,
    };

    if !is_invited_user_in_same_voice_channel(&voice_states, &voice_channel_id, &invited_user) {
        return ephemeral_response("You cannot invite someone who is already in the voice channel.");
    }
    
    

    let dm_result = dm_user(
        ctx,
        invited_user,
        inviter,
        get_channel_link(guild_id, voice_channel_id),
    )
    .await;
    match dm_result {
        Err(_) => {
            ephemeral_response("Failed to send the invitation. The user might have DMs disabled.")
        }
        Ok(_) => CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(format!("Invitation sent to {}.", invited_user.mention())),
        ),
    }
}

fn is_invited_user_in_same_voice_channel(voice_states: &&HashMap<UserId, VoiceState>, voice_channel_id: &ChannelId, invited_user: &&UserId) -> bool {
    voice_states.iter().any(|(user_id, voice_state)| {
        let is_in_same_voice_channel = match voice_state.channel_id {
            None => return false,
            Some(channel_id) => channel_id.get() == voice_channel_id.get(),
        };

        if !is_in_same_voice_channel {
            return false;
        };

        return invited_user.get() == user_id.get();
    })
}

async fn dm_user(
    ctx: &Context,
    invited_user: &UserId,
    inviter: &User,
    channel_invite: String,
) -> serenity::Result<Message> {
    invited_user
        .direct_message(
            &ctx,
            CreateMessage::new().content(format!(
                "Hey, {} invited you to join the voice channel: {}",
                inviter.mention(),
                channel_invite
            )),
        )
        .await
}

fn ephemeral_response(string: &str) -> CreateInteractionResponse {
    CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .ephemeral(true)
            .content(string),
    )
}

fn get_invited_user(command: &CommandInteraction) -> Option<&UserId> {
    let command_data_option = command.data.options.first()?;

    match command_data_option {
        CommandDataOption {
            value: CommandDataOptionValue::User(user),
            ..
        } => Some(user),
        _ => None,
    }
}

fn get_voice_channel_id(optional_voice_state: Option<&VoiceState>) -> Option<ChannelId> {
    optional_voice_state?.channel_id
}

fn get_channel_link(guild_id: GuildId, channel_id: ChannelId) -> String {
    format!("https://discord.com/channels/{}/{}", guild_id, channel_id)
}
