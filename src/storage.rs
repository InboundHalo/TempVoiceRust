use crate::creator_channel::CreatorChannelConfig;
use crate::temporary_channel::TemporaryVoiceChannel;
use async_trait::async_trait;
use rusqlite::{params, Connection};
use serde_json;
use serenity::all::ChannelId;
use tokio::task;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn get_creator_voice_config(
        &self,
        channel_id: &ChannelId,
    ) -> Option<CreatorChannelConfig>;
    async fn set_creator_voice_config(&self, creator_config: &CreatorChannelConfig);

    async fn get_temporary_voice_channel(
        &self,
        channel_id: &ChannelId,
    ) -> Option<TemporaryVoiceChannel>;
    async fn set_temporary_voice_channel(&self, temporary_channel: &TemporaryVoiceChannel);
    async fn delete_temporary_voice_channel(&self, channel_id: &ChannelId);
    async fn get_all_temporary_voice_channels(&self) -> Option<Vec<TemporaryVoiceChannel>>;
}

pub struct SQLiteStorage {
    database_path: String,
}

impl SQLiteStorage {
    pub(crate) fn new(database_path: &str) -> rusqlite::Result<Self> {
        let storage = SQLiteStorage {
            database_path: database_path.to_string(),
        };
        storage.initialize_database()?;
        Ok(storage)
    }

    fn initialize_database(&self) -> rusqlite::Result<()> {
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
impl Storage for SQLiteStorage {
    async fn get_creator_voice_config(
        &self,
        channel_id: &ChannelId,
    ) -> Option<CreatorChannelConfig> {
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

    async fn set_creator_voice_config(&self, creator_config: &CreatorChannelConfig) {
        let db_path = self.database_path.clone();
        let channel_id_u64 = creator_config.creator_id.get();
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
        .await
        .expect("TODO: panic message");
    }

    async fn get_temporary_voice_channel(
        &self,
        channel_id: &ChannelId,
    ) -> Option<TemporaryVoiceChannel> {
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
                let temp_channel: TemporaryVoiceChannel =
                    serde_json::from_str(&config_data).ok()?;
                Some(temp_channel)
            } else {
                None
            }
        })
        .await
        .unwrap_or(None)
    }

    async fn set_temporary_voice_channel(&self, temporary_channel: &TemporaryVoiceChannel) {
        let db_path = self.database_path.clone();
        let channel_id_u64 = temporary_channel.channel_id.get();
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
        .await
        .expect("TODO: panic message");
    }

    async fn delete_temporary_voice_channel(&self, channel_id: &ChannelId) {
        let db_path = self.database_path.clone();
        let channel_id_u64 = channel_id.get();
        task::spawn_blocking(move || {
            let conn = Connection::open(db_path).ok()?;
            conn.execute(
                "DELETE FROM temporary_voice_channel WHERE channel_id = ?1",
                params![channel_id_u64],
            )
            .ok()
        })
        .await
        .expect("Failed to delete temporary voice channel");
    }

    async fn get_all_temporary_voice_channels(&self) -> Option<Vec<TemporaryVoiceChannel>> {
        let db_path = self.database_path.clone();
        task::spawn_blocking(move || {
            let conn = Connection::open(db_path).ok()?;

            let mut statement = conn
                .prepare("SELECT config_data FROM temporary_voice_channel")
                .ok()?;

            let rows = statement
                .query_map(params![], |row| {
                    let config_data: String = row.get(0)?;
                    let temp_channel: TemporaryVoiceChannel = serde_json::from_str(&config_data)
                        .map_err(|_| rusqlite::Error::InvalidQuery)?;
                    Ok(temp_channel)
                })
                .ok()?;

            let temp_channels: Vec<TemporaryVoiceChannel> =
                rows.filter_map(|result| result.ok()).collect();

            Some(temp_channels)
        })
        .await
        .unwrap_or(None)
    }
}
