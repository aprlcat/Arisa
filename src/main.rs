mod command;
mod config;
mod error;
mod listener;
mod util;

use std::sync::Arc;

use command::{
    crypto::{checksum, hash, uuid},
    encoding::{base64, endian, rot, timestamp, url},
    java::{jep, opcode},
    misc::{color, github, hawktuah, help},
    security::cve,
};
use config::Config;
use error::BotError;
use poise::serenity_prelude as serenity;
use util::cooldown::CooldownManager;

type Error = BotError;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Clone)]
pub struct Data {
    pub config: Arc<Config>,
    pub cooldown_manager: CooldownManager,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            let error_msg = match &error {
                BotError::Cooldown(seconds) => {
                    format!("⏰ Command on cooldown for {} seconds", seconds)
                }
                BotError::InputTooLarge(size) => {
                    format!("📏 Input too large: {} characters", size)
                }
                _ => format!("❌ Error: {}", error),
            };

            let embed = util::command::create_error_response("Command Error", &error_msg);
            if let Err(e) = ctx
                .send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await
            {
                println!("Error sending error response: {:?}", e);
            }

            println!("Error in command `{}`: {:?}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

async fn on_ready(
    ctx: &serenity::Context,
    ready: &serenity::Ready,
    _framework: &poise::Framework<Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    println!("Logged in as {}", ready.user.name);
    let initial_activity = util::quote::get_random_activity(&data.config);
    let initial_status = util::quote::get_random_status();
    ctx.set_presence(Some(initial_activity), initial_status);
    util::status::start_status_updater(ctx.clone().into(), data.config.clone());

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    util::logger::init().unwrap();

    let config =
        Arc::new(Config::load_or_create("config.toml").expect("Failed to load configuration"));

    if config.discord.token.is_empty() {
        panic!("Discord token not set in config.toml");
    }

    let cooldown_manager = CooldownManager::new();

    let cleanup_manager = cooldown_manager.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            cleanup_manager.cleanup_expired(3600).await;
        }
    });

    let data = Data {
        config: config.clone(),
        cooldown_manager,
    };

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                help(),
                base64(),
                url(),
                rot(),
                endian(),
                timestamp(),
                hash(),
                checksum(),
                uuid(),
                github(),
                color(),
                hawktuah(),
                jep(),
                opcode(),
                cve(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: None,
                ..Default::default()
            },
            on_error: |error| Box::pin(on_error(error)),
            pre_command: |ctx| {
                Box::pin(async move {
                    println!("Executing command {}...", ctx.command().qualified_name);
                })
            },
            post_command: |ctx| {
                Box::pin(async move {
                    println!("Executed command {}!", ctx.command().qualified_name);
                })
            },
            command_check: Some(|ctx| {
                Box::pin(async move {
                    if ctx.author().bot {
                        return Ok(false);
                    }
                    Ok(true)
                })
            }),
            skip_checks_for_owners: false,
            event_handler: |ctx, event, _framework, _data| {
                Box::pin(async move {
                    match event {
                        poise::serenity_prelude::FullEvent::Message { new_message } => {
                            listener::handle_message(ctx, new_message).await;
                        }
                        _ => {}
                    }
                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                let commands =
                    poise::builtins::create_application_commands(&framework.options().commands);

                let user_installable_commands: Vec<_> = commands
                    .into_iter()
                    .map(|command| {
                        command
                            .integration_types(vec![
                                serenity::InstallationContext::Guild,
                                serenity::InstallationContext::User,
                            ])
                            .contexts(vec![
                                serenity::InteractionContext::Guild,
                                serenity::InteractionContext::BotDm,
                                serenity::InteractionContext::PrivateChannel,
                            ])
                    })
                    .collect();

                ctx.http
                    .create_global_commands(&user_installable_commands)
                    .await?;
                println!(
                    "Registered {} user-installable commands",
                    user_installable_commands.len()
                );

                on_ready(ctx, ready, framework, &data).await?;
                Ok(data)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(&config.discord.token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}
