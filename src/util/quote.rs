use poise::serenity_prelude::{ActivityData, OnlineStatus};
use rand::Rng;

use crate::config::Config;

pub fn get_random_quote(config: &Config) -> &str {
    let mut rng = rand::rng();
    let index = rng.random_range(0..config.quotes.quotes.len());
    &config.quotes.quotes[index]
}

pub fn get_random_status() -> OnlineStatus {
    let mut rng = rand::rng();
    let statuses = [
        OnlineStatus::Online,
        OnlineStatus::Idle,
        OnlineStatus::DoNotDisturb,
    ];

    let index = rng.random_range(0..statuses.len());
    statuses[index]
}

pub fn get_random_activity(config: &Config) -> ActivityData {
    let mut rng = rand::rng();
    let quote = get_random_quote(config);

    let activity_types = [
        ActivityData::playing(quote),
        ActivityData::listening(quote),
        ActivityData::watching(quote),
        ActivityData::competing(quote),
    ];

    let index = rng.random_range(0..activity_types.len());
    activity_types[index].clone()
}

pub fn get_random_interval_minutes(config: &Config) -> u64 {
    let mut rng = rand::rng();
    let (min, max) = config.quotes.update_interval_minutes;
    rng.random_range(min..=max)
}
