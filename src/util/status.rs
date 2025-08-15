use std::sync::Arc;

use poise::serenity_prelude::Context as SerenityContext;
use tokio::time::{Duration, sleep};

use crate::{
    config::Config,
    util::quote::{get_random_activity, get_random_interval_minutes, get_random_status},
};

pub fn start_status_updater(ctx: Arc<SerenityContext>, config: Arc<Config>) {
    tokio::spawn(async move {
        loop {
            let wait_minutes = get_random_interval_minutes(&config);
            sleep(Duration::from_secs(wait_minutes * 60)).await;

            let activity = get_random_activity(&config);
            let status = get_random_status();

            ctx.set_presence(Some(activity), status);
            println!("Updated status to: {:?}", status);
        }
    });
}
