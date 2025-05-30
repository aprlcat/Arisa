use poise::serenity_prelude::{Color, CreateEmbed};

use crate::util::quote;

pub struct CatppuccinColors;

impl CatppuccinColors {
    pub const LAVENDER: u32 = 0xbabbf1;
    pub const RED: u32 = 0xe78284;
    pub const GREEN: u32 = 0xa6d189;
    #[allow(dead_code)]
    pub const YELLOW: u32 = 0xe5c890;
    pub const BLUE: u32 = 0x8caaee;
    #[allow(dead_code)]
    pub const MAUVE: u32 = 0xca9ee6;
}

pub fn create_embed(title: &str, color: u32) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .color(Color::new(color))
        .footer(poise::serenity_prelude::CreateEmbedFooter::new(
            quote::get_random_quote(),
        ))
}

pub fn create_success_embed(title: &str) -> CreateEmbed {
    create_embed(title, CatppuccinColors::GREEN)
}

pub fn create_error_embed(title: &str) -> CreateEmbed {
    create_embed(title, CatppuccinColors::RED)
}

pub fn create_info_embed(title: &str) -> CreateEmbed {
    create_embed(title, CatppuccinColors::LAVENDER)
}

#[allow(dead_code)]
pub fn create_warning_embed(title: &str) -> CreateEmbed {
    create_embed(title, CatppuccinColors::YELLOW)
}
