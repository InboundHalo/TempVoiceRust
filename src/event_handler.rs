use std::sync::Arc;

use crate::storage::Storage;
use crate::temporary_channel::{get_name_from_template, get_user_presence, TemporaryVoiceChannel};
use crate::StorageKey;
use async_trait::async_trait;
use serenity::all::{
    Channel, ChannelId, ChannelType, Context, CreateChannel, EditChannel, EventHandler, Member,
    PermissionOverwrite, PermissionOverwriteType, Ready, VoiceState,
};
use serenity::model::Permissions;

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, data_about_bot: Ready) {
        println!("{} is connected!", data_about_bot.user.name);
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let storage = match {
            let data_read = ctx.data.read().await;
            data_read.get::<StorageKey>().cloned()
        } {
            None => {
                println!("Storage is null!");
                panic!()
            }
            Some(storage) => storage,
        };

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
    storage: &Arc<impl Storage + Send + Sync + ?Sized>,
    member: &Member,
    channel_id: ChannelId,
) {
    if let Some(mut config) = storage.get_creator_voice_config(&channel_id).await {
        let user = member.user.clone();
        let owner_id = user.id;
        let owner_name = member.display_name();

        let creator_channel_id = channel_id;
        let number = config.get_next_number();

        let naming_standard = config.naming_standard.clone();

        let user_presence = get_user_presence(ctx, &config.guild_id, &owner_id);
        let channel_name =
            get_name_from_template(&naming_standard, &number, user_presence, owner_name);

        let builder = CreateChannel::new(channel_name.clone())
            .kind(ChannelType::Voice)
            .user_limit(config.user_limit)
            .category(config.category_id)
            .position(number)
            .permissions(vec![PermissionOverwrite {
                allow: Permissions::MOVE_MEMBERS | Permissions::MANAGE_CHANNELS,
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Member(member.user.id),
            }])
            .audit_log_reason("Temp voice bot");

        // Create the channel
        let channel = match config.guild_id.create_channel(&ctx.http, builder).await {
            Ok(channel) => channel,
            Err(_) => {
                println!("Something went wrong while creating a channel!");
                return;
            }
        };

        let channel_id = channel.id;

        // Move the member to the new voice channel
        if let Err(_) = member.move_to_voice_channel(&ctx.http, channel_id).await {
            println!("Unable to move user to voice channel");
            return;
        }

        if config.add_number(number) {
            let temporary_voice_channel = TemporaryVoiceChannel::new(
                config.guild_id,
                channel_id,
                creator_channel_id,
                owner_id,
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
                        .edit(ctx, EditChannel::new().position(highest_number + 1))
                        .await
                    {
                        println!("Error editing channel positions: {:?}", why);
                        // Do not return as this does not matter too much if it fails
                    }
                }
            } else {
                panic!("Highest number not found");
            }
        } else {
            println!("Something went wrong!");
            todo!();
        }
    } else {
        println!("Member {} joined a regular channel", member.user.name);
    }
}

async fn on_voice_channel_leave(
    ctx: &Context,
    storage: &Arc<impl Storage + Send + Sync + ?Sized>,
    _member: &Member,
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

    println!("There are {} members left in the channel.", member_count);

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
                            .await
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
