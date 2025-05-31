use crate::{
    Context, Error,
    util::command::{check_cooldown, create_error_response, create_success_response, validate_input_size},
};

fn rot_char(c: char, n: u8) -> char {
    match c {
        'a'..='z' => ((c as u8 - b'a' + n) % 26 + b'a') as char,
        'A'..='Z' => ((c as u8 - b'A' + n) % 26 + b'A') as char,
        _ => c,
    }
}

fn rot_string(s: &str, n: u8) -> String {
    s.chars().map(|c| rot_char(c, n)).collect()
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Apply ROT cipher to text with custom rotation value")
)]
pub async fn rot(
    ctx: Context<'_>,
    #[description = "Rotation value (0-25)"]
    #[min = 0]
    #[max = 25]
    n: u8,
    #[description = "The text to apply ROT cipher to"] text: String,
) -> Result<(), Error> {
    check_cooldown(&ctx, "rot", ctx.data().config.cooldowns.per_user_cooldown).await?;

    if let Err(e) = validate_input_size(&text, &ctx.data().config) {
        let embed = create_error_response("ROT Error", &e.to_string());
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let result = rot_string(&text, n);
    let title = format!("ROT{}", n);
    let embed = create_success_response(&title, &result, true, &ctx.data().config);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}