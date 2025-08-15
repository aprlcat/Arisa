use crate::{
    Context, Error,
    util::{
        command::check_cooldown,
        embed::{CatppuccinColors, create_info_embed},
    },
};

#[poise::command(
    slash_command,
    description_localized("en-US", "Hawk tuah! Spit on that thang!")
)]
pub async fn hawktuah(ctx: Context<'_>) -> Result<(), Error> {
    check_cooldown(
        &ctx,
        "hawktuah",
        ctx.data().config.cooldowns.per_user_cooldown,
    )
    .await?;

    let embed = create_info_embed("Hawk Tuah!", &ctx.data().config)
        .description("Spit on that thang! 🦅💦")
        .image("https://media.tenor.com/5BBbfyeW7soAAAAC/hawk-hawk-tuah.gif")
        .color(CatppuccinColors::MAUVE);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
