use poise::serenity_prelude::{Color, CreateEmbed};

use crate::{config::Config, util::quote};

pub struct CatppuccinColors;

impl CatppuccinColors {
    pub const LAVENDER: u32 = 0xbabbf1;
    pub const RED: u32 = 0xe78284;
    pub const GREEN: u32 = 0xa6d189;
    pub const YELLOW: u32 = 0xe5c890;
    pub const BLUE: u32 = 0x8caaee;
    pub const MAUVE: u32 = 0xca9ee6;
}

pub fn create_embed(title: &str, color: u32, config: &Config) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .color(Color::new(color))
        .footer(poise::serenity_prelude::CreateEmbedFooter::new(
            quote::get_random_quote(config),
        ))
}

pub fn create_success_embed(title: &str, config: &Config) -> CreateEmbed {
    create_embed(title, CatppuccinColors::GREEN, config)
}

pub fn create_error_embed(title: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .color(Color::new(CatppuccinColors::RED))
}

pub fn create_info_embed(title: &str, config: &Config) -> CreateEmbed {
    create_embed(title, CatppuccinColors::LAVENDER, config)
}

pub fn create_warning_embed(title: &str, config: &Config) -> CreateEmbed {
    create_embed(title, CatppuccinColors::YELLOW, config)
}
