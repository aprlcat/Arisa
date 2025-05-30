use crate::{
    Context, Error,
    util::command::{create_error_response, create_success_response, validate_input_size},
};

#[derive(poise::ChoiceParameter)]
pub enum Operation {
    #[name = "Encode"]
    Encode,
    #[name = "Decode"]
    Decode,
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Encode or decode data using Base64")
)]
pub async fn base64(
    ctx: Context<'_>,
    #[description = "Choose operation"] operation: Operation,
    #[description = "The data to encode or decode"] data: String,
) -> Result<(), Error> {
    if let Err(e) = validate_input_size(&data) {
        let embed = create_error_response("Base64 Error", &e);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let (title, result, is_success) = match operation {
        Operation::Encode => {
            let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
            ("Base64 Encoded", encoded, true)
        }
        Operation::Decode => {
            match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &data) {
                Ok(decoded) => match String::from_utf8(decoded) {
                    Ok(utf8_string) => ("Base64 Decoded", utf8_string, true),
                    Err(_) => {
                        let embed = create_error_response(
                            "Base64 Error",
                            "Decoded data is not valid UTF-8",
                        );
                        ctx.send(poise::CreateReply::default().embed(embed)).await?;
                        return Ok(());
                    }
                },
                Err(e) => {
                    let embed =
                        create_error_response("Base64 Error", &format!("Invalid base64: {}", e));
                    ctx.send(poise::CreateReply::default().embed(embed)).await?;
                    return Ok(());
                }
            }
        }
    };

    let embed = if is_success {
        create_success_response(title, &result, true)
    } else {
        create_error_response(title, &result)
    };

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
