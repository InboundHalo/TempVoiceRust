use std::collections::HashSet;
use std::env;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, EventHandler, GatewayIntents, GuildId};
use serenity::Client;
use serenity::prelude::TypeMapKey;

use crate::storage::{CreatorChannelConfig, SQLiteStorage, StorageType};

mod storage;
mod event_handler;

pub(crate) struct SQLiteStorageKey;

impl TypeMapKey for SQLiteStorageKey {
    type Value = Arc<SQLiteStorage>;
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
        .event_handler(event_handler::Handler)
        .await
        .expect("Err creating client")
}