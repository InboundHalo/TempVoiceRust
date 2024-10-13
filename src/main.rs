use std::env;
use std::sync::Arc;

use serenity::all::{EventHandler, GatewayIntents};
use serenity::Client;
use serenity::prelude::TypeMapKey;

use crate::storage::{SQLiteStorage, Storage};

mod storage;
mod event_handler;
mod creator_channel;
mod temporary_channel;

pub(crate) struct StorageKey;

impl TypeMapKey for StorageKey {
    type Value = Arc<dyn Storage + Send + Sync>;
}

#[tokio::main]
async fn main() {
    println!("Starting up");

    let storage: Arc<dyn Storage + Send + Sync> = Arc::new(
        SQLiteStorage::new("my_test_database.db").expect("Failed to initialize storage"),
    );

    let mut client: Client = setup_discord_bot().await;

    let mut data = client.data.write().await;
    data.insert::<StorageKey>(Arc::clone(&storage));
    drop(data);

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