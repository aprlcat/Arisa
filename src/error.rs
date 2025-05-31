use std::fmt;

#[derive(Debug)]
pub enum BotError {
    InputTooLarge(usize),
    Network(reqwest::Error),
    ImageGeneration(image::ImageError),
    Serialization(serde_json::Error),
    InvalidFormat(String),
    GitHub(String),
    Color(String),
    Cooldown(u64),
    Config(String),
    Serenity(poise::serenity_prelude::Error),
}

impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BotError::InputTooLarge(size) => write!(f, "Input too large: {} characters", size),
            BotError::Network(e) => write!(f, "Network error: {}", e),
            BotError::ImageGeneration(e) => write!(f, "Image generation error: {}", e),
            BotError::Serialization(e) => write!(f, "Serialization error: {}", e),
            BotError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            BotError::GitHub(msg) => write!(f, "GitHub error: {}", msg),
            BotError::Color(msg) => write!(f, "Color error: {}", msg),
            BotError::Cooldown(seconds) => write!(f, "Command on cooldown for {} seconds", seconds),
            BotError::Config(msg) => write!(f, "Configuration error: {}", msg),
            BotError::Serenity(e) => write!(f, "Discord error: {}", e),
        }
    }
}

impl std::error::Error for BotError {}

impl From<reqwest::Error> for BotError {
    fn from(error: reqwest::Error) -> Self {
        BotError::Network(error)
    }
}

impl From<image::ImageError> for BotError {
    fn from(error: image::ImageError) -> Self {
        BotError::ImageGeneration(error)
    }
}

impl From<serde_json::Error> for BotError {
    fn from(error: serde_json::Error) -> Self {
        BotError::Serialization(error)
    }
}

impl From<poise::serenity_prelude::Error> for BotError {
    fn from(error: poise::serenity_prelude::Error) -> Self {
        BotError::Serenity(error)
    }
}

pub type Result<T> = std::result::Result<T, BotError>;