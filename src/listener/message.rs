use poise::serenity_prelude::{Context, Message};

pub async fn handle_message(ctx: &Context, msg: &Message) {
    if msg.author.bot {
        return;
    }

    let content = msg.content.to_lowercase();

    if content.starts_with("how do i ") || content.contains("how do i ") {
        if let Err(e) = msg.reply(&ctx.http, "very carefully").await {
            println!("Error sending 'very carefully' reply: {:?}", e);
        }
    }
}
