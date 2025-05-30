use poise::serenity_prelude::CreateEmbed;

use crate::{
    Context, Error,
    util::embed::{CatppuccinColors, create_info_embed},
};

#[poise::command(
    slash_command,
    description_localized("en-US", "Show help information about commands")
)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    if let Some(command_name) = command {
        let commands = &ctx.framework().options().commands;
        if let Some(cmd) = commands.iter().find(|c| c.name == command_name) {
            let embed = create_command_help_embed(cmd);
            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        } else {
            let embed = create_info_embed("Command Not Found")
                .description(format!("No command named `{}` was found.", command_name))
                .color(CatppuccinColors::RED);
            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
    } else {
        let embed = create_general_help_embed(ctx).await?;
        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
    }

    Ok(())
}

fn create_command_help_embed(command: &poise::Command<crate::Data, Error>) -> CreateEmbed {
    let mut embed =
        create_info_embed(&format!("Help: {}", command.name)).color(CatppuccinColors::BLUE);

    if let Some(description) = &command.description {
        embed = embed.description(description);
    }

    if let Some(help_text) = &command.help_text {
        embed = embed.field("Details", help_text, false);
    }

    if !command.parameters.is_empty() {
        let mut params_text = String::new();
        for param in &command.parameters {
            params_text.push_str(&format!(
                "• **{}**: {}\n",
                param.name,
                param.description.as_deref().unwrap_or("No description")
            ));
        }
        embed = embed.field("Parameters", params_text, false);
    }

    embed = embed.field(
        "Usage",
        format!("Use `/{}` in Discord", command.name),
        false,
    );

    embed
}

async fn create_general_help_embed(ctx: Context<'_>) -> Result<CreateEmbed, Error> {
    let commands = &ctx.framework().options().commands;

    let mut embed = create_info_embed("Arisa - Command Help")
        .description("I go by it/she, I'm a discord bot for nerds, by nerds :3")
        .color(CatppuccinColors::LAVENDER);

    let mut encoding_commands = Vec::new();
    let mut crypto_commands = Vec::new();
    let mut misc_commands = Vec::new();

    for command in commands {
        if command.name == "help" {
            misc_commands.push(command);
        } else if ["base64", "url", "rot", "endian"].contains(&command.name.as_str()) {
            encoding_commands.push(command);
        } else if ["hash", "checksum"].contains(&command.name.as_str()) {
            crypto_commands.push(command);
        }
    }

    if !encoding_commands.is_empty() {
        let mut field_text = String::new();
        for cmd in encoding_commands {
            field_text.push_str(&format!(
                "• **{}** - {}\n",
                cmd.name,
                cmd.description.as_deref().unwrap_or("No description")
            ));
        }
        embed = embed.field("🔤 Encoding & Text", field_text, false);
    }

    if !crypto_commands.is_empty() {
        let mut field_text = String::new();
        for cmd in crypto_commands {
            field_text.push_str(&format!(
                "• **{}** - {}\n",
                cmd.name,
                cmd.description.as_deref().unwrap_or("No description")
            ));
        }
        embed = embed.field("🔐 Cryptography", field_text, false);
    }

    if !misc_commands.is_empty() {
        let mut field_text = String::new();
        for cmd in misc_commands {
            field_text.push_str(&format!(
                "• **{}** - {}\n",
                cmd.name,
                cmd.description.as_deref().unwrap_or("No description")
            ));
        }
        embed = embed.field("ℹ️ Miscellaneous", field_text, false);
    }

    embed = embed.field(
        "Usage",
        "Use `/help <command>` for detailed information about a specific command!",
        false,
    );

    Ok(embed)
}
