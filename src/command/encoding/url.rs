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
    description_localized("en-US", "Encode or decode data using URL encoding")
)]
pub async fn url(
    ctx: Context<'_>,
    #[description = "Choose operation"] operation: Operation,
    #[description = "The data to encode or decode"] data: String,
) -> Result<(), Error> {
    if let Err(e) = validate_input_size(&data) {
        let embed = create_error_response("URL Encoding Error", &e);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let (title, result) = match operation {
        Operation::Encode => {
            let encoded = urlencoding::encode(&data);
            ("URL Encoded", encoded.to_string())
        }
        Operation::Decode => match urlencoding::decode(&data) {
            Ok(decoded) => ("URL Decoded", decoded.to_string()),
            Err(e) => {
                let embed = create_error_response(
                    "URL Encoding Error",
                    &format!("Invalid URL encoding: {}", e),
                );
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
                return Ok(());
            }
        },
    };

    let embed = create_success_response(&title, &result, true);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
