use poise::serenity_prelude::CreateEmbed;

use crate::util::embed::{create_error_embed, create_info_embed, create_success_embed};

const MAX_INPUT_SIZE: usize = 4096;
const MAX_OUTPUT_SIZE: usize = 4000;

pub fn validate_input_size(input: &str) -> Result<(), String> {
    if input.len() > MAX_INPUT_SIZE {
        return Err(format!(
            "Input too large (max {} characters)",
            MAX_INPUT_SIZE
        ));
    }
    Ok(())
}

pub fn truncate_output(output: String) -> String {
    if output.len() > MAX_OUTPUT_SIZE {
        format!("{}...\n*Output truncated*", &output[..MAX_OUTPUT_SIZE - 20])
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

pub fn create_success_response(title: &str, content: &str, is_code: bool) -> CreateEmbed {
    let description = if is_code {
        format_code_block(content, None)
    } else {
        content.to_string()
    };

    create_success_embed(title).description(truncate_output(description))
}

#[allow(dead_code)]
pub fn create_info_response(title: &str, content: &str, is_code: bool) -> CreateEmbed {
    let description = if is_code {
        format_code_block(content, None)
    } else {
        content.to_string()
    };

    create_info_embed(title).description(truncate_output(description))
}
