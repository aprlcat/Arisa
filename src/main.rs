mod command;
mod listener;
mod util;

use std::env;

use command::{
    crypto::{checksum, hash, uuid},
    encoding::{base64, endian, rot, timestamp, url},
    misc::{color, github, help},
};
use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
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
) -> Result<(), Error> {
    println!("Logged in as {}", ready.user.name);
    let initial_activity = util::quote::get_random_activity();
    let initial_status = util::quote::get_random_status();
    ctx.set_presence(Some(initial_activity), initial_status);
    util::status::start_status_updater(ctx.clone().into());

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    util::logger::init().unwrap();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

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

                on_ready(ctx, ready, framework).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}
