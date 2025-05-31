use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use crate::error::{BotError, Result};

#[derive(Clone)]
pub struct CooldownManager {
    cooldowns: Arc<RwLock<HashMap<String, HashMap<u64, Instant>>>>,
}

impl CooldownManager {
    pub fn new() -> Self {
        Self {
            cooldowns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_cooldown(
        &self,
        command: &str,
        user_id: u64,
        cooldown_seconds: u64,
    ) -> Result<()> {
        let mut cooldowns = self.cooldowns.write().await;
        let command_cooldowns = cooldowns.entry(command.to_string()).or_insert_with(HashMap::new);

        if let Some(&last_used) = command_cooldowns.get(&user_id) {
            let elapsed = last_used.elapsed();
            let cooldown_duration = Duration::from_secs(cooldown_seconds);

            if elapsed < cooldown_duration {
                let remaining = cooldown_duration - elapsed;
                return Err(BotError::Cooldown(remaining.as_secs()));
            }
        }

        command_cooldowns.insert(user_id, Instant::now());
        Ok(())
    }

    pub async fn cleanup_expired(&self, max_age_seconds: u64) {
        let mut cooldowns = self.cooldowns.write().await;
        let cutoff = Instant::now() - Duration::from_secs(max_age_seconds);

        for command_cooldowns in cooldowns.values_mut() {
            command_cooldowns.retain(|_, &mut last_used| last_used > cutoff);
        }

        cooldowns.retain(|_, command_cooldowns| !command_cooldowns.is_empty());
    }
}

impl Default for CooldownManager {
    fn default() -> Self {
        Self::new()
    }
}