use crate::{
    Context, Error,
    util::command::{create_error_response, create_success_response},
};

#[poise::command(
    slash_command,
    description_localized("en-US", "Convert and display colors in multiple formats")
)]
pub async fn color(
    ctx: Context<'_>,
    #[description = "Color in HEX (#FF0000), RGB (255,0,0), or name (red)"] input: String,
) -> Result<(), Error> {
    let input = input.trim();

    let color = match parse_color(input) {
        Ok(c) => c,
        Err(e) => {
            let error_msg = format!(
                "{}\n\nSupported formats:\n• HEX: #FF0000 or FF0000\n• RGB: rgb(255, 0, 0) or \
                 255,0,0\n• HSL: hsl(0, 100%, 50%)\n• Color names: red, blue, green, etc.",
                e
            );
            let embed = create_error_response("Invalid Color Format", &error_msg);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let hex = format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b);
    let rgb = format!("rgb({}, {}, {})", color.r, color.g, color.b);
    let hsl = rgb_to_hsl(color.r, color.g, color.b);
    let hsl_str = format!("hsl({}, {}%, {}%)", hsl.0, hsl.1, hsl.2);
    let hsv = rgb_to_hsv(color.r, color.g, color.b);
    let hsv_str = format!("hsv({}, {}%, {}%)", hsv.0, hsv.1, hsv.2);
    let cmyk = rgb_to_cmyk(color.r, color.g, color.b);
    let cmyk_str = format!("cmyk({}%, {}%, {}%, {}%)", cmyk.0, cmyk.1, cmyk.2, cmyk.3);

    let description = format!(
        "**Color Formats:**\n**HEX:** `{}`\n**RGB:** `{}`\n**HSL:** `{}`\n**HSV:** \
         `{}`\n**CMYK:** `{}`\n\n**Values:**\n**Decimal:** `{}`\n**CSS:** `{}`\n**Int:** `{}`",
        hex,
        rgb,
        hsl_str,
        hsv_str,
        cmyk_str,
        (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32,
        rgb,
        (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32
    );

    let title = format!("Color: {}", hex);
    let embed = create_success_response(&title, &description, false);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

fn parse_color(input: &str) -> Result<Color, String> {
    let input = input.to_lowercase();
    if let Some(color) = parse_named_color(&input) {
        return Ok(color);
    }

    if input.starts_with('#') || input.chars().all(|c| c.is_ascii_hexdigit()) {
        return parse_hex_color(&input);
    }

    if input.starts_with("rgb(") && input.ends_with(')') {
        return parse_rgb_color(&input);
    }

    if input.contains(',') {
        return parse_comma_rgb(&input);
    }

    if input.starts_with("hsl(") && input.ends_with(')') {
        return parse_hsl_color(&input);
    }

    Err("Unrecognized color format".to_string())
}

fn parse_hex_color(input: &str) -> Result<Color, String> {
    let hex = input.strip_prefix('#').unwrap_or(input);

    match hex.len() {
        3 => {
            let r =
                u8::from_str_radix(&hex[0..1].repeat(2), 16).map_err(|_| "Invalid hex digit")?;
            let g =
                u8::from_str_radix(&hex[1..2].repeat(2), 16).map_err(|_| "Invalid hex digit")?;
            let b =
                u8::from_str_radix(&hex[2..3].repeat(2), 16).map_err(|_| "Invalid hex digit")?;
            Ok(Color { r, g, b })
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex digit")?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex digit")?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex digit")?;
            Ok(Color { r, g, b })
        }
        _ => Err("Hex color must be 3 or 6 characters".to_string()),
    }
}

fn parse_rgb_color(input: &str) -> Result<Color, String> {
    let content = input
        .strip_prefix("rgb(")
        .unwrap()
        .strip_suffix(')')
        .unwrap();
    parse_comma_rgb(content)
}

fn parse_comma_rgb(input: &str) -> Result<Color, String> {
    let parts: Vec<&str> = input.split(',').map(|s| s.trim()).collect();

    if parts.len() != 3 {
        return Err("RGB format requires 3 values".to_string());
    }

    let r = parts[0].parse::<u8>().map_err(|_| "Invalid red value")?;
    let g = parts[1].parse::<u8>().map_err(|_| "Invalid green value")?;
    let b = parts[2].parse::<u8>().map_err(|_| "Invalid blue value")?;

    Ok(Color { r, g, b })
}

fn parse_hsl_color(input: &str) -> Result<Color, String> {
    let content = input
        .strip_prefix("hsl(")
        .unwrap()
        .strip_suffix(')')
        .unwrap();
    let parts: Vec<&str> = content
        .split(',')
        .map(|s| s.trim().trim_end_matches('%'))
        .collect();

    if parts.len() != 3 {
        return Err("HSL format requires 3 values".to_string());
    }

    let h = parts[0].parse::<f32>().map_err(|_| "Invalid hue value")?;
    let s = parts[1]
        .parse::<f32>()
        .map_err(|_| "Invalid saturation value")?
        / 100.0;
    let l = parts[2]
        .parse::<f32>()
        .map_err(|_| "Invalid lightness value")?
        / 100.0;

    let (r, g, b) = hsl_to_rgb(h, s, l);
    Ok(Color { r, g, b })
}

fn parse_named_color(name: &str) -> Option<Color> {
    match name {
        "red" => Some(Color { r: 255, g: 0, b: 0 }),
        "green" => Some(Color { r: 0, g: 128, b: 0 }),
        "blue" => Some(Color { r: 0, g: 0, b: 255 }),
        "white" => Some(Color {
            r: 255,
            g: 255,
            b: 255,
        }),
        "black" => Some(Color { r: 0, g: 0, b: 0 }),
        "yellow" => Some(Color {
            r: 255,
            g: 255,
            b: 0,
        }),
        "cyan" => Some(Color {
            r: 0,
            g: 255,
            b: 255,
        }),
        "magenta" => Some(Color {
            r: 255,
            g: 0,
            b: 255,
        }),
        "orange" => Some(Color {
            r: 255,
            g: 165,
            b: 0,
        }),
        "purple" => Some(Color {
            r: 128,
            g: 0,
            b: 128,
        }),
        "pink" => Some(Color {
            r: 255,
            g: 192,
            b: 203,
        }),
        "brown" => Some(Color {
            r: 165,
            g: 42,
            b: 42,
        }),
        "gray" | "grey" => Some(Color {
            r: 128,
            g: 128,
            b: 128,
        }),
        "lime" => Some(Color { r: 0, g: 255, b: 0 }),
        "navy" => Some(Color { r: 0, g: 0, b: 128 }),
        "maroon" => Some(Color { r: 128, g: 0, b: 0 }),
        "olive" => Some(Color {
            r: 128,
            g: 128,
            b: 0,
        }),
        "teal" => Some(Color {
            r: 0,
            g: 128,
            b: 128,
        }),
        "silver" => Some(Color {
            r: 192,
            g: 192,
            b: 192,
        }),
        _ => None,
    }
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (u16, u8, u8) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let l = (max + min) / 2.0;

    if delta == 0.0 {
        return (0, 0, (l * 100.0) as u8);
    }

    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let h = if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    (h as u16, (s * 100.0) as u8, (l * 100.0) as u8)
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (u16, u8, u8) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let v = max;
    let s = if max == 0.0 { 0.0 } else { delta / max };

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    (h as u16, (s * 100.0) as u8, (v * 100.0) as u8)
}

fn rgb_to_cmyk(r: u8, g: u8, b: u8) -> (u8, u8, u8, u8) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let k = 1.0 - r.max(g.max(b));

    if k == 1.0 {
        return (0, 0, 0, 100);
    }

    let c = (1.0 - r - k) / (1.0 - k);
    let m = (1.0 - g - k) / (1.0 - k);
    let y = (1.0 - b - k) / (1.0 - k);

    (
        (c * 100.0) as u8,
        (m * 100.0) as u8,
        (y * 100.0) as u8,
        (k * 100.0) as u8,
    )
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r_prime, g_prime, b_prime) = match h as u16 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        300..=359 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };

    (
        ((r_prime + m) * 255.0) as u8,
        ((g_prime + m) * 255.0) as u8,
        ((b_prime + m) * 255.0) as u8,
    )
}
