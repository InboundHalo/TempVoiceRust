use async_trait::async_trait;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::all::{ChannelId, GuildId, UserId};
use serenity::prelude::*;
use tokio::task;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatorChannelConfig {
    pub(crate) guild_id: GuildId,
    pub(crate) creator_id: ChannelId,
    pub(crate) category_id: ChannelId,
    pub(crate) naming_standard: String,
    pub(crate) channel_numbers: Vec<u8>,
    pub(crate) user_limit:u32
}

impl CreatorChannelConfig {
    pub(crate) fn get_next_number(&self) -> u8 {
        get_next_number(self, 1)
    }


}

fn get_next_number(creator_channel_config: &CreatorChannelConfig, number: u8) -> u8 {
    if creator_channel_config.channel_numbers.contains(&number) {
        get_next_number(creator_channel_config, number+1)
    } else {
        number
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemporaryVoiceChannel {
    channel_id: ChannelId,
    creator_id: ChannelId,
    owner_id: UserId,
    name: String,
    template_name: String,
    number: u8,
}

#[async_trait]
pub trait StorageType {
    async fn get_creator_voice_config(&self, channel_id: ChannelId) -> Option<CreatorChannelConfig>;
    async fn set_creator_voice_config(&self, channel_id: ChannelId, creator_config: CreatorChannelConfig);

    async fn get_temporary_voice_channel(&self, channel_id: ChannelId) -> Option<TemporaryVoiceChannel>;
    async fn set_temporary_voice_channel(&self, channel_id: ChannelId, temporary_channel: TemporaryVoiceChannel);
}



pub struct SQLiteStorage {
    database_path: String,
}

impl SQLiteStorage {
    pub(crate) fn new(database_path: &str) -> Result<Self> {
        let storage = SQLiteStorage {
            database_path: database_path.to_string(),
        };
        storage.initialize_database()?;
        Ok(storage)
    }

    fn initialize_database(&self) -> Result<()> {
        let conn = Connection::open(&self.database_path)?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS creator_channel_config (
                channel_id INTEGER PRIMARY KEY,
                config_data TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS temporary_voice_channel (
                channel_id INTEGER PRIMARY KEY,
                config_data TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }
}

#[async_trait]
impl StorageType for SQLiteStorage {
    async fn get_creator_voice_config(&self, channel_id: ChannelId) -> Option<CreatorChannelConfig> {
        let db_path = self.database_path.clone();
        let channel_id_u64 = channel_id.get();
        task::spawn_blocking(move || {
            let conn = Connection::open(db_path).ok()?;
            let mut stmt = conn
                .prepare("SELECT config_data FROM creator_channel_config WHERE channel_id = ?1")
                .ok()?;
            let mut rows = stmt.query(params![channel_id_u64]).ok()?;
            if let Some(row) = rows.next().ok()? {
                let config_data: String = row.get(0).ok()?;
                let config: CreatorChannelConfig = serde_json::from_str(&config_data).ok()?;
                Some(config)
            } else {
                None
            }
        })
            .await
            .unwrap_or(None)
    }

    async fn set_creator_voice_config(&self, channel_id: ChannelId, creator_config: CreatorChannelConfig) {
        let db_path = self.database_path.clone();
        let channel_id_u64 = channel_id.get();
        let config_data = serde_json::to_string(&creator_config).unwrap_or_default();
        task::spawn_blocking(move || {
            let conn = Connection::open(db_path).ok()?;
            conn.execute(
                "
                INSERT INTO creator_channel_config (channel_id, config_data) VALUES (?1, ?2)
                ON CONFLICT(channel_id) DO UPDATE SET config_data=excluded.config_data
                ",
                params![channel_id_u64, config_data],
            )
                .ok()
        })
            .await.expect("TODO: panic message");
    }

    async fn get_temporary_voice_channel(&self, channel_id: ChannelId) -> Option<TemporaryVoiceChannel> {
        let db_path = self.database_path.clone();
        let channel_id_u64 = channel_id.get();
        task::spawn_blocking(move || {
            let conn = Connection::open(db_path).ok()?;
            let mut stmt = conn
                .prepare("SELECT config_data FROM temporary_voice_channel WHERE channel_id = ?1")
                .ok()?;
            let mut rows = stmt.query(params![channel_id_u64]).ok()?;
            if let Some(row) = rows.next().ok()? {
                let config_data: String = row.get(0).ok()?;
                let temp_channel: TemporaryVoiceChannel = serde_json::from_str(&config_data).ok()?;
                Some(temp_channel)
            } else {
                None
            }
        })
            .await
            .unwrap_or(None)
    }

    async fn set_temporary_voice_channel(&self, channel_id: ChannelId, temporary_channel: TemporaryVoiceChannel) {
        let db_path = self.database_path.clone();
        let channel_id_u64 = channel_id.get();
        let config_data = serde_json::to_string(&temporary_channel).unwrap_or_default();
        task::spawn_blocking(move || {
            let conn = Connection::open(db_path).ok()?;
            conn.execute(
                "
                INSERT INTO temporary_voice_channel (channel_id, config_data) VALUES (?1, ?2)
                ON CONFLICT(channel_id) DO UPDATE SET config_data=excluded.config_data
                ",
                params![channel_id_u64, config_data],
            )
                .ok()
        })
            .await.expect("TODO: panic message");
    }
}
