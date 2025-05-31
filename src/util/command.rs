use poise::serenity_prelude::CreateEmbed;

use crate::config::Config;
use crate::error::{BotError, Result};
use crate::util::embed::{create_error_embed, create_info_embed, create_success_embed};

pub async fn check_cooldown(
    ctx: &crate::Context<'_>,
    command_name: &str,
    cooldown_seconds: u64,
) -> Result<()> {
    let user_id = ctx.author().id.get();
    ctx.data()
        .cooldown_manager
        .check_cooldown(command_name, user_id, cooldown_seconds)
        .await
}

pub fn validate_input_size(input: &str, config: &Config) -> Result<()> {
    if input.len() > config.limits.max_input_size {
        return Err(BotError::InputTooLarge(input.len()));
    }
    Ok(())
}

pub fn truncate_output(output: String, config: &Config) -> String {
    if output.len() > config.limits.max_output_size {
        format!("{}...\n*Output truncated*", &output[..config.limits.max_output_size - 20])
    } else {
        output
    }
}

pub fn format_code_block(content: &str, language: Option<&str>) -> String {
    let lang = language.unwrap_or("");
    format!("```{}\n{}\n```", lang, content)
}

pub fn create_error_response(title: &str, error: &str) -> CreateEmbed {
    create_error_embed(title).description(error)
}

pub fn create_success_response(title: &str, content: &str, is_code: bool, config: &Config) -> CreateEmbed {
    let description = if is_code {
        format_code_block(content, None)
    } else {
        content.to_string()
    };

    create_success_embed(title, config).description(truncate_output(description, config))
}

pub fn create_info_response(title: &str, content: &str, is_code: bool, config: &Config) -> CreateEmbed {
    let description = if is_code {
        format_code_block(content, None)
    } else {
        content.to_string()
    };

    create_info_embed(title, config).description(truncate_output(description, config))
}