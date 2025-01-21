use crate::event_handler::cool_down_manager::CooldownManager;
use crate::StorageKey;
use serenity::all::{ChannelId, CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType, Context, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, Mentionable, Message, PermissionOverwrite, PermissionOverwriteType, Permissions, User, UserId, VoiceState};
use serenity::builder::{CreateCommand, CreateCommandOption, CreateMessage};
use serenity::http::Http;
use std::collections::HashMap;
use std::sync::Arc;

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

    let voice_states = {
        let guild = match guild_id.to_guild_cached(&ctx) {
            Some(guild) => guild.clone(),
            None => return ephemeral_response("Failed to retrieve guild data."),
        };
        guild.voice_states.clone()
    };

    let voice_channel_id = match get_voice_channel_id(voice_states.get(&inviter.id)) {
        None => return ephemeral_response("You must be in a voice channel to use this command."),
        Some(channel_id) => channel_id,
    };

    if is_invited_user_in_same_voice_channel(&voice_states, &voice_channel_id, &invited_user) {
        return ephemeral_response(
            "You cannot invite someone who is already in the voice channel.",
        );
    }

    // Check if user is owner of the voice channel if so give the invited user perms
    let storage = {
        let data_read = ctx.data.read().await;
        match data_read.get::<StorageKey>().cloned() {
            None => {
                println!("Storage is null!");
                panic!()
            }
            Some(storage) => storage,
        }
    };

    let temporary_voice_channel = storage.get_temporary_voice_channel(&voice_channel_id).await;

    let is_owner_of_voice_channel = match temporary_voice_channel {
        None => false,
        Some(temporary_voice_channel) => temporary_voice_channel.owner_id == inviter.id,
    };

    let guild_channel = match voice_channel_id.to_channel(ctx).await {
        Ok(channel) => channel.guild(),
        Err(_) => None,
    };

    if is_owner_of_voice_channel {
        let permissions = PermissionOverwrite {
            allow: Permissions::VIEW_CHANNEL
                | Permissions::MOVE_MEMBERS // This permission lets the invited user join even if the voice channel is full
                | Permissions::CONNECT
                | Permissions::SPEAK
                | Permissions::SEND_MESSAGES
                | Permissions::READ_MESSAGE_HISTORY,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Member(invited_user.clone()),
        };

        let _ = voice_channel_id.create_permission(ctx, permissions).await;
    }

    let can_connect = match guild_channel {
        None => false,
        Some(guild_channel) => {
            if let Some(guild) = guild_channel.guild(ctx) {
                if let Some(member) = guild.members.get(invited_user) {
                    let permissions = guild.user_permissions_in(&guild_channel, member);

                    permissions.administrator() || (permissions.connect() && permissions.view_channel())
                } else { false }
            } else { false }
        }
    };

    let dm_result = dm_user(
        ctx.http.clone(),
        invited_user,
        inviter,
        get_channel_link(guild_id, voice_channel_id),
    );

    match can_connect {
        true => match dm_result.await {
            Ok(_) => CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(format!("Invitation sent to {}.", invited_user.mention()))),
            Err(_) => CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(format!("Failed to send the invitation. The {} might have DMs disabled. They have been pinged and can join however.", invited_user.mention()))),
        }
        false => ephemeral_response("User can not connect to voice channel!"),
    }
}

fn is_invited_user_in_same_voice_channel(
    voice_states: &HashMap<UserId, VoiceState>,
    voice_channel_id: &ChannelId,
    invited_user: &&UserId,
) -> bool {
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
    ctx: Arc<Http>,
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
