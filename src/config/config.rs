use serde::{Deserialize, Serialize};
use std::fs;
use crate::error::{BotError, Result};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub discord: DiscordConfig,
    pub limits: LimitsConfig,
    pub cooldowns: CooldownConfig,
    pub quotes: QuotesConfig,
    pub github: GitHubConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscordConfig {
    pub token: String,
    pub application_id: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitsConfig {
    pub max_input_size: usize,
    pub max_output_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CooldownConfig {
    pub global_cooldown: u64,
    pub per_user_cooldown: u64,
    pub hash_cooldown: u64,
    pub github_cooldown: u64,
    pub color_cooldown: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QuotesConfig {
    pub quotes: Vec<String>,
    pub update_interval_minutes: (u64, u64),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubConfig {
    pub user_agent: String,
    pub token: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            discord: DiscordConfig {
                token: String::new(),
                application_id: None,
            },
            limits: LimitsConfig {
                max_input_size: 4096,
                max_output_size: 4000,
            },
            cooldowns: CooldownConfig {
                global_cooldown: 1,
                per_user_cooldown: 3,
                hash_cooldown: 5,
                github_cooldown: 10,
                color_cooldown: 2,
            },
            quotes: QuotesConfig {
                quotes: vec![
                    "I LOVE TRANSGENDER WOEMN".to_string(),
                    "im sexy and i know it".to_string(),
                    "thinking github came before git like thinking pornhub came before porn".to_string(),
                    "translation lookaside buff a cock up my ass".to_string(),
                    "mov eax 0x80000000 mov ebx [eax] int 0x80".to_string(),
                    "love in the air! WRONG cannibalism".to_string(),
                    "void fastcall memset mm_loadh_ps double v3 254 0LL 42681LL".to_string(),
                    "ghidra backdoored by the NSA".to_string(),
                    "a monad just a monoid in the category of endofunctors".to_string(),
                    "nix was created by homosexuals for homosexuals".to_string(),
                    "schizophrenic pond dweller the Frog coming".to_string(),
                    "GITPULLO COMMITO MERGE CONFLICTO".to_string(),
                    "i wannq fuck my computer".to_string(),
                    "looks like the guys doing the testing got their CFLAGS wrong I reckon they forgot omit frame pointer".to_string(),
                    "g fsanitize undefined address fno omit frame pointer".to_string(),
                    "segfault yourself".to_string(),
                    "cat dev random".to_string(),
                ],
                update_interval_minutes: (3, 5),
            },
            github: GitHubConfig {
                user_agent: "Arisa-Bot/1.0".to_string(),
                token: None,
            },
        }
    }
}

impl Config {
    pub fn load_or_create(path: &str) -> Result<Self> {
        if std::path::Path::new(path).exists() {
            let content = fs::read_to_string(path)
                .map_err(|e| BotError::Config(format!("Failed to read config file: {}", e)))?;
            let config: Config = toml::from_str(&content)
                .map_err(|e| BotError::Config(format!("Failed to parse config: {}", e)))?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save(path)?;
            println!("Created default config at {}. Please edit it with your Discord token.", path);
            Ok(config)
        }
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| BotError::Config(format!("Failed to serialize config: {}", e)))?;
        fs::write(path, content)
            .map_err(|e| BotError::Config(format!("Failed to write config file: {}", e)))?;
        Ok(())
    }
}