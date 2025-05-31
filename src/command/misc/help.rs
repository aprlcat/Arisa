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
            let embed = create_command_help_embed(cmd, &ctx.data().config);
            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        } else {
            let embed = create_info_embed("Command Not Found", &ctx.data().config)
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

fn create_command_help_embed(
    command: &poise::Command<crate::Data, Error>,
    config: &crate::Config,
) -> CreateEmbed {
    let mut embed =
        create_info_embed(&format!("Help: {}", command.name), config).color(CatppuccinColors::BLUE);

    let description = get_command_description(command);
    embed = embed.description(&description);

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

    let mut embed = create_info_embed("Arisa", &ctx.data().config)
        .description("I go by it/she, I'm a discord bot for nerds, by nerds :3")
        .color(CatppuccinColors::LAVENDER);

    let mut encoding_commands = Vec::new();
    let mut crypto_commands = Vec::new();
    let mut assembly_commands = Vec::new();
    let mut misc_commands = Vec::new();

    for command in commands {
        match command.name.as_str() {
            "help" | "github" | "color" => misc_commands.push(command),
            "base64" | "url" | "rot" | "endian" | "timestamp" => encoding_commands.push(command),
            "hash" | "checksum" | "uuid" => crypto_commands.push(command),
            "x86" => assembly_commands.push(command),
            _ => misc_commands.push(command),
        }
    }

    if !encoding_commands.is_empty() {
        let mut field_text = String::new();
        for cmd in encoding_commands {
            field_text.push_str(&format!(
                "• **{}** - {}\n",
                cmd.name,
                get_command_description(cmd)
            ));
        }
        embed = embed.field("Encoding & Text", field_text, false);
    }

    if !crypto_commands.is_empty() {
        let mut field_text = String::new();
        for cmd in crypto_commands {
            field_text.push_str(&format!(
                "• **{}** - {}\n",
                cmd.name,
                get_command_description(cmd)
            ));
        }
        embed = embed.field("Cryptography", field_text, false);
    }

    if !assembly_commands.is_empty() {
        let mut field_text = String::new();
        for cmd in assembly_commands {
            field_text.push_str(&format!(
                "• **{}** - {}\n",
                cmd.name,
                get_command_description(cmd)
            ));
        }
        embed = embed.field("Assembly", field_text, false);
    }

    if !misc_commands.is_empty() {
        let mut field_text = String::new();
        for cmd in misc_commands {
            field_text.push_str(&format!(
                "• **{}** - {}\n",
                cmd.name,
                get_command_description(cmd)
            ));
        }
        embed = embed.field("Miscellaneous", field_text, false);
    }

    embed = embed.field(
        "Usage",
        "Use `/help <command>` for detailed information about a specific command!",
        false,
    );

    embed = embed.field(
        "Tip",
        "Install me to your account to use these commands everywhere on Discord :3",
        false,
    );

    Ok(embed)
}

fn get_command_description(command: &poise::Command<crate::Data, Error>) -> String {
    match command.name.as_str() {
        "base64" => "Encode or decode data using Base64".to_string(),
        "url" => "Encode or decode data using URL encoding".to_string(),
        "rot" => "Apply ROT cipher to text with custom rotation value".to_string(),
        "endian" => "Swap the endianness of hexadecimal data".to_string(),
        "timestamp" => "Convert Unix timestamps to human-readable dates".to_string(),
        "hash" => "Generate cryptographic hashes of data".to_string(),
        "checksum" => "Calculate checksums of data for integrity verification".to_string(),
        "uuid" => "Generate UUIDs (Universally Unique Identifiers)".to_string(),
        "x86" => "Look up x86 assembly instructions by mnemonic or hex opcode".to_string(),
        "github" => "Get GitHub user or repository information".to_string(),
        "color" => "Convert and display colors in multiple formats".to_string(),
        "help" => "Show help information about commands".to_string(),
        _ => command
            .description
            .as_deref()
            .unwrap_or("No description")
            .to_string(),
    }
}