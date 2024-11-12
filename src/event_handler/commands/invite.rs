use serenity::all::{
    ChannelId, CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType,
    Context, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, Mentionable,
    User, UserId, VoiceState,
};
use serenity::builder::{CreateCommand, CreateCommandOption};

pub fn register() -> CreateCommand {
    CreateCommand::new("invite")
        .description("Invites a user to the voice channel")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "User to invite")
                .required(true),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> CreateInteractionResponse {
    let invited_user = match get_invited_user(&command) {
        None => {
            return CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content("You must mention a user to invite."),
            );
        }
        Some(user_id) => user_id,
    };

    let guild_id = match command.guild_id {
        None => {
            return CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content("This command can only be used in a server."),
            );
        }
        Some(guild_id) => guild_id,
    };

    let inviter = &command.user;
    let guild = guild_id.to_guild_cached(&ctx).unwrap();

    let voice_channel_id = match get_voice_channel_id(guild.voice_states.get(&inviter.id)) {
        None => {
            return CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content("You must be in a voice channel to use this command."),
            )
        }
        Some(channel_id) => channel_id,
    };

    let dm_result = dm_user(
        ctx,
        invited_user,
        inviter,
        get_channel_link(guild_id, voice_channel_id),
    )
    .await;

    match dm_result {
        Err(_) => CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .ephemeral(true)
                .content("Failed to send the invitation. The user might have DMs disabled."),
        ),
        Ok(_) => CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(format!("Invitation sent to {}.", invited_user.mention())),
        ),
    }
}

async fn dm_user(
    ctx: &Context,
    invited_user: &UserId,
    inviter: &User,
    channel_invite: String,
) -> Result<(), serenity::Error> {
    invited_user
        .direct_message(&ctx.http, |m| {
            m.content(format!(
                "Hey, {} invited you to join the voice channel: {}",
                inviter.mention(),
                channel_invite
            ))
        })
        .await
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
