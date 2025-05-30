use crate::{
    Context, Error,
    util::command::{create_error_response, create_success_response},
};

#[poise::command(
    slash_command,
    description_localized("en-US", "Swap the endianness of hexadecimal data")
)]
pub async fn endian(
    ctx: Context<'_>,
    #[description = "Hexadecimal data to swap (e.g., 'DEADBEEF' or '0xDEADBEEF')"] hex_data: String,
) -> Result<(), Error> {
    let clean_hex = hex_data.replace(" ", "").replace("0x", "");

    if clean_hex.len() % 2 != 0 {
        let embed = create_error_response("Endian Swap Error", "Hex string must have even length");
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    match hex::decode(&clean_hex) {
        Ok(bytes) => {
            let swapped: Vec<u8> = bytes.into_iter().rev().collect();
            let result = hex::encode(swapped).to_uppercase();
            let embed = create_success_response("Endianness Swapped", &result, true);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            let embed = create_error_response("Endian Swap Error", &format!("Invalid hex: {}", e));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }

    Ok(())
}
