use std::sync::Arc;
use async_trait::async_trait;
use serenity::all::{Channel, ChannelId, ChannelType, Context, CreateChannel, EventHandler, Member, Ready, VoiceState};
use crate::SQLiteStorageKey;
use crate::storage::{CreatorChannelConfig, SQLiteStorage, StorageType, TemporaryVoiceChannel};

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, data_about_bot: Ready) {
        println!("{} is connected!", data_about_bot.user.name);
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        // Get storage
        let storage = {
            let data_read = ctx.data.read().await;
            data_read.get::<SQLiteStorageKey>().cloned()
        };

        if storage.is_none() {
            println!("Storage is null!");
            return;
        }

        let storage = storage.unwrap();


        // Make sure we have a member
        let member = match new.member {
            Some(ref member) => member,
            None => return,
        };

        // Member joins a voice channel
        if new.channel_id.is_some() {
            on_voice_channel_join(&ctx, &storage, member, new.channel_id.unwrap()).await;
        }

        // Member leaves a voice channel
        if old.is_some() {
            on_voice_channel_leave(&ctx, &storage, member, old.unwrap()).await;
        }
    }
}

async fn on_voice_channel_join(
    ctx: &Context,
    storage: &Arc<SQLiteStorage>,
    member: &Member,
    channel_id: ChannelId
) {
    if let Some(mut config) = storage.get_creator_voice_config(channel_id).await {
        println!(
            "Member {} joined a creator channel: {:?}",
            member.user.name, config
        );

        let number = config.get_next_number();

        let channel_name = config.naming_standard.replace("%number%", number.to_string().as_str());

        let builder = CreateChannel::new(channel_name.clone());
        let builder = builder.kind(ChannelType::Voice);
        let builder = builder.user_limit(config.user_limit);
        let builder = builder.category(config.category_id);
        let builder = builder.audit_log_reason("Temp voice bot");

        let channel = config.guild_id.create_channel(&ctx.http, builder).await;

        if channel.is_err() {
            println!("Unable to create voice channel");
            return;
        }

        let channel = channel.unwrap();

        let channel_id = channel.id;

        let result = member.move_to_voice_channel(&ctx.http, channel_id).await;

        if result.is_err() {
            println!("Unable to move user to voice channel");
            return;
        }

        match config.add_number(number) {
            true => {
                storage.set_temporary_voice_channel
                (
                    channel_id,
                    TemporaryVoiceChannel{
                        channel_id,
                        creator_id: config.creator_id,
                        owner_id: member.user.id,
                        name: channel_name,
                        template_name: config.clone().naming_standard,
                        number,
                    }
                ).await;

                storage.set_creator_voice_config(config.creator_id, config).await;

                println!("Created voice channel: {}", channel.name);
            }
            false => {
                println!("Something went wrong!");
                todo!()
            }
        };
    } else {
        println!("Member {} joined a regular channel", member.user.name);
    }
    return;
}

async fn on_voice_channel_leave(
    ctx: &Context,
    storage: &Arc<SQLiteStorage>,
    member: &Member,
    old_voice_state: VoiceState,
) {
    let old_channel_id = match old_voice_state.channel_id {
        None => {
            println!("User was not in a voice channel previously.");
            return;
        }
        Some(old_channel_id) => old_channel_id
    };

    let temp_channel = match storage.get_temporary_voice_channel(old_channel_id).await {
        None => {
            println!("Member {} left a regular channel", member.user.name);
            return;
        }
        Some(temp_channel) => temp_channel
    };

    println!(
        "Member {} left a temporary voice channel: {:?}",
        member.user.name, temp_channel
    );

    let channel = match old_channel_id.to_channel(ctx).await {
        Ok(Channel::Guild(channel)) => channel,
        _ => {
            println!("Failed to retrieve the channel or it is not a guild channel.");
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

    println!("There are {} members left in the channel.", member_count);

    if member_count == 0 {
        println!("No members left, deleting the channel");
        match channel.delete(&ctx.http).await {
            Ok(_) => {
                match storage.get_creator_voice_config(
                    temp_channel.creator_id
                ).await {
                    None => {
                        println!("Something went very wrong when deleting a channel!");
                        todo!()
                    }
                    Some(mut creator_channel_config) => {
                        creator_channel_config.remove_number(&temp_channel.number);

                        storage.set_creator_voice_config(creator_channel_config.creator_id, creator_channel_config).await
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