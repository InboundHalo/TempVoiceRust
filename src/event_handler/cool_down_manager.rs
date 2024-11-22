use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serenity::model::prelude::UserId;

#[derive(Clone)]
pub struct CooldownManager {
    user_cooldowns: Arc<Mutex<HashMap<UserId, HashMap<UserId, Instant>>>>,
}

impl CooldownManager {
    pub fn new() -> Self {
        CooldownManager {
            user_cooldowns:  Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn can_user_ping_user(&self, user_id: &UserId, user_id_to_ping: &UserId) -> bool {
        const DELAY: Duration = Duration::from_secs(20);

        let now = Instant::now();
        let mut user_lock = self.user_cooldowns.lock().unwrap();
        let mut user_cooldowns = user_lock.entry(user_id.clone()).or_insert_with(HashMap::new);

        if let Some(last_ping_time) = user_cooldowns.get(user_id_to_ping) {
            if now.duration_since(*last_ping_time) < DELAY {
                return false;
            }
        }

        user_cooldowns.insert(*user_id_to_ping, now);

        return true;
    }
}