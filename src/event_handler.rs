use std::sync::Arc;

use crate::storage::Storage;
use crate::temporary_channel::{get_name_from_template, get_user_presence, TemporaryVoiceChannel};
use crate::{StorageKey};
use async_trait::async_trait;
use serenity::all::{Channel, ChannelId, ChannelType, Context, CreateChannel, EditChannel, EventHandler, GuildChannel, Member, PermissionOverwrite, PermissionOverwriteType, Ready, VoiceState};
use serenity::model::Permissions;
use crate::creator_channel::CreatorChannelConfig;

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        
        println!("{} is ready!", ready.user.name);
    }

    async fn voice_state_update(&self, ctx: Context, old_voice_state: Option<VoiceState>, new_voice_state: VoiceState) {
        // Return early if the channels has not changed
        if old_voice_state.as_ref().and_then(|old_voice_state| old_voice_state.channel_id) == new_voice_state.channel_id {
            return;
        }

        let member = match &new_voice_state.member {
            Some(ref member) => member,
            None => return,
        };

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

        // Member joins a voice channel
        if new_voice_state.channel_id.is_some() {
            match on_voice_channel_join(&ctx, &storage, member, new_voice_state.channel_id.unwrap()).await {
                None => {} // This means they did not join a creator channel
                Some(result) => {
                    match result {
                    Ok(_channel) => {}
                    Err(why) => {
                        println!("Error joining channel: {:?}", why);
                        match member.disconnect_from_voice(&ctx).await {
                            Ok(_) => {
                                println!("Disconnected from voice channel");
                            }
                            Err(_) => {
                                println!("Failed to disconnect from voice channel");
                            }
                        };
                    }
                        }
                }
            };
        }

        // Member leaves a voice channel
        if old_voice_state.is_some() {
            on_voice_channel_leave(&ctx, &storage, old_voice_state.unwrap()).await;
        }
    }
}

async fn on_voice_channel_join(
    ctx: &Context,
    storage: &Arc<impl Storage + Send + Sync + ?Sized>,
    member: &Member,
    creator_channel_id: ChannelId,
) -> Option<Result<GuildChannel, &'static str>> {
    let mut config = storage.get_creator_voice_config(&creator_channel_id).await?;

    let voice_channel_owner = member.user.clone();
    let voice_channel_owner_id = voice_channel_owner.id;
    let voice_channel_owner_name = member.display_name();

    let naming_standard = config.naming_standard.clone();

    let guild_id = config.guild_id;

    let number = config.get_next_number();
    let user_presence = get_user_presence(ctx, &guild_id, &voice_channel_owner_id);
    
    let channel_name =
        get_name_from_template(&naming_standard, &number, user_presence, voice_channel_owner_name);

    let creator_channel = match guild_id.channels(ctx).await {
        Err(_) => return Some(Err("Could not get guild channels")),
        Ok(hash_map) => {
            match hash_map.get(&creator_channel_id) {
                None => return Some(Err("Could not get the creator channel")),
                Some(guild_channel) => guild_channel.clone(),
            }
        }
    };

    let mut permissions_overrides = creator_channel.permission_overwrites.clone();

    permissions_overrides.push(PermissionOverwrite {
        allow: Permissions::MOVE_MEMBERS | Permissions::MANAGE_CHANNELS,
        deny: Permissions::empty(),
        kind: PermissionOverwriteType::Member(member.user.id),
    });

    let bitrate = creator_channel.bitrate.unwrap_or_else(|| 64000);
    let nsfw = creator_channel.nsfw;

    let builder = CreateChannel::new(channel_name.clone())
        .kind(ChannelType::Voice)
        .user_limit(config.user_limit)
        .category(config.category_id)
        .position(number.get())
        .permissions(permissions_overrides)
        .audit_log_reason("Temp voice bot")
        .nsfw(nsfw)
        .bitrate(bitrate);

    // Create the channel
    let channel = match config.guild_id.create_channel(&ctx.http, builder).await {
        Ok(channel) => channel,
        Err(_) => return Some(Err("Could not create guild channel")),
    };

    let channel_id = channel.id;

    // Move the member to the new voice channel
    if let Err(_) = member.move_to_voice_channel(&ctx.http, channel_id).await {
        let _ = channel.delete(ctx).await;
        return Some(Err("Could not move voice channel to creator channel"));
    }

    if !config.add_number(number) {
        return Some(Err("Could not add number to config!"));
    }

    let temporary_voice_channel = TemporaryVoiceChannel::new(
        config.guild_id,
        channel_id,
        creator_channel_id,
        voice_channel_owner_id,
        channel_name,
        naming_standard,
        number,
    );

    storage
        .set_temporary_voice_channel(&temporary_voice_channel)
        .await;

    if let Some(highest_number) = config.get_highest_number() {
        storage.set_creator_voice_config(&config).await;

        if number == highest_number {
            if let Err(why) = creator_channel_id
                .edit(ctx, EditChannel::new().position(highest_number.get() + 1))
                .await
            {
                println!("Error editing channel positions: {:?}", why);
                // Do not return as this does not matter too much if it fails
            }
        }
    } else {
        panic!("Highest number not found");
    }

    Some(Ok(channel))
}

async fn on_voice_channel_leave(
    ctx: &Context,
    storage: &Arc<impl Storage + Send + Sync + ?Sized>,
    old_voice_state: VoiceState,
) {
    let old_channel_id = match old_voice_state.channel_id {
        None => return,
        Some(old_channel_id) => old_channel_id,
    };

    let temp_channel = match storage.get_temporary_voice_channel(&old_channel_id).await {
        None => return,
        Some(temp_channel) => temp_channel,
    };

    let channel = match old_channel_id.to_channel(ctx).await {
        Ok(Channel::Guild(channel)) => channel,
        Err(why) => {
            println!(
                "Failed to retrieve the channel or it is not a guild channel: {}",
                why
            );
            return;
        }
        _ => {
            println!("Failed to retrieve the channel or it is not a guild channel. No error");
            return;
        }
    };

    let guild_id = channel.guild_id;
    let voice_channel_id = channel.id;

    let member_count = {
        let guild = match guild_id.to_guild_cached(ctx) {
            Some(guild) => guild,
            None => {
                println!("Failed to retrieve the guild.");
                return;
            }
        };

        let count = guild
            .voice_states
            .values()
            .filter(|vs| vs.channel_id == Some(voice_channel_id))
            .count();

        count
    };

    println!(
        "There are {} members left in the channel {}.",
        member_count, temp_channel.number
    );

    if member_count == 0 {
        println!("No members left, deleting the channel");
        match channel.delete(&ctx.http).await {
            Ok(_) => {
                match storage
                    .get_creator_voice_config(&temp_channel.creator_id)
                    .await
                {
                    None => {
                        println!("Something went very wrong when deleting a channel!");
                        todo!()
                    }
                    Some(mut creator_channel_config) => {
                        creator_channel_config.remove_number(&temp_channel.number);

                        storage
                            .set_creator_voice_config(&creator_channel_config)
                            .await;
                        storage.delete_temporary_voice_channel(&channel.id).await;
                    }
                }
            }
            Err(_) => {
                println!("Something went very wrong when deleting a channel!");
                todo!()
            }
        };
    }
}
