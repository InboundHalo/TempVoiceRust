mod storage;

use serenity::all::{ChannelType, Context, CreateChannel, EventHandler, GatewayIntents, Ready, VoiceState};
use crate::storage::{SQLiteStorage, StorageType};
use serde::{Deserialize, Serialize};
use serenity::prelude::TypeMapKey;
use serenity::{async_trait, Client};
use std::env;
use std::sync::Arc;

pub struct SQLiteStorageKey;

impl TypeMapKey for SQLiteStorageKey {
    type Value = Arc<SQLiteStorage>;
}

struct Handler;

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

        let member_name = member.user.name.clone();

        // Member joins a voice channel
        if new.channel_id.is_some() {
            let channel_id = new.channel_id.unwrap();
            if let Some(config) = storage.get_creator_voice_config(channel_id).await {
                println!(
                    "Member {} joined a creator channel: {:?}",
                    member_name, config
                );

                let channel_name = config.naming_standard.replace("%number%", config.get_next_number().to_string().as_str());

                let builder = CreateChannel::new(channel_name);
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



                let result = new.member.unwrap().move_to_voice_channel(&ctx.http, channel.id).await;

                if result.is_err() {
                    println!("Unable to move user to voice channel");
                    return;
                }

                // TODO: Add to DB

                println!("Created voice channel: {}", channel.name);
            } else {
                println!("Member {} joined a regular channel", member_name);
            }
            return;
        }

        // Member leaves a voice channel
        if old.is_some() {
            if let Some(old_channel_id) = old.as_ref().unwrap().channel_id {
                // TODO: check if there is a temporary vc and if so check if there are any more members in the vc and if not then delete it after interval

                if let Some(temp_channel) = storage.get_temporary_voice_channel(old_channel_id).await {
                    println!(
                        "Member {} left a temporary voice channel: {:?}",
                        member_name, temp_channel
                    );

                } else {
                    println!("Member {} left a regular channel", member_name);
                }
            }
            return;
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Starting up");
    
    let storage = Arc::new(
        SQLiteStorage::new("my_test_database.db").expect("Failed to initialize storage"),
    );

    let mut client: Client = setup_discord_bot().await;

    // Find out why this only works with {}
    {
        let mut data = client.data.write().await;
        data.insert::<SQLiteStorageKey>(Arc::clone(&storage));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

async fn setup_discord_bot() -> Client {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_PRESENCES;

    Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client")
}