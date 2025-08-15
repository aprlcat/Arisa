use chrono::{DateTime, Local, TimeZone, Utc};

use crate::{
    Context, Error,
    util::command::{check_cooldown, create_error_response, create_success_response},
};

#[poise::command(
    slash_command,
    description_localized("en-US", "Convert Unix timestamps to human-readable dates")
)]
pub async fn timestamp(
    ctx: Context<'_>,
    #[description = "Unix timestamp (leave empty to get current timestamp)"] timestamp: Option<i64>,
    #[description = "Date string to convert to timestamp (format: YYYY-MM-DD HH:MM:SS)"]
    date: Option<String>,
    #[description = "Show in local timezone instead of UTC"] local: Option<bool>,
) -> Result<(), Error> {
    check_cooldown(
        &ctx,
        "timestamp",
        ctx.data().config.cooldowns.per_user_cooldown,
    )
    .await?;

    let _use_local = local.unwrap_or(false);

    let (title, content) = if let Some(ts) = timestamp {
        match Utc.timestamp_opt(ts, 0) {
            chrono::LocalResult::Single(dt) => {
                let utc_str = dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();
                let local_dt = dt.with_timezone(&Local);
                let local_str = local_dt.format("%Y-%m-%d %H:%M:%S %Z").to_string();
                let relative = format_relative_time(ts);

                let content = format!(
                    "**Unix Timestamp:** `{}`\n**UTC:** {}\n**Local:** {}\n**Relative:** \
                     {}\n**ISO 8601:** {}\n**RFC 2822:** {}",
                    ts,
                    utc_str,
                    local_str,
                    relative,
                    dt.to_rfc3339(),
                    dt.to_rfc2822()
                );

                ("Timestamp Conversion", content)
            }
            _ => {
                let embed = create_error_response(
                    "Invalid timestamp",
                    "The provided timestamp is out of range or invalid.",
                );
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
                return Ok(());
            }
        }
    } else if let Some(date_str) = date {
        match parse_date_string(&date_str) {
            Ok(dt) => {
                let timestamp = dt.timestamp();
                let utc_str = dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();
                let relative = format_relative_time(timestamp);

                let content = format!(
                    "**Date:** {}\n**Unix Timestamp:** `{}`\n**Relative:** {}\n**ISO 8601:** \
                     {}\n**RFC 2822:** {}",
                    utc_str,
                    timestamp,
                    relative,
                    dt.to_rfc3339(),
                    dt.to_rfc2822()
                );

                ("Date Conversion", content)
            }
            Err(e) => {
                let embed = create_error_response(
                    "Invalid date format",
                    &format!(
                        "Could not parse date: {}\n\nExpected format: YYYY-MM-DD HH:MM:SS",
                        e
                    ),
                );
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
                return Ok(());
            }
        }
    } else {
        let now = Utc::now();
        let timestamp = now.timestamp();
        let utc_str = now.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        let local_now = now.with_timezone(&Local);
        let local_str = local_now.format("%Y-%m-%d %H:%M:%S %Z").to_string();

        let content = format!(
            "**Unix Timestamp:** `{}`\n**UTC:** {}\n**Local:** {}\n**ISO 8601:** {}\n**RFC \
             2822:** {}",
            timestamp,
            utc_str,
            local_str,
            now.to_rfc3339(),
            now.to_rfc2822()
        );

        ("Current Timestamp", content)
    };

    let embed = create_success_response(title, &content, false, &ctx.data().config);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

fn parse_date_string(date_str: &str) -> Result<DateTime<Utc>, String> {
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d",
        "%d/%m/%Y %H:%M:%S",
        "%d/%m/%Y %H:%M",
        "%d/%m/%Y",
        "%m/%d/%Y %H:%M:%S",
        "%m/%d/%Y %H:%M",
        "%m/%d/%Y",
    ];

    for format in &formats {
        if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
            return Ok(Utc.from_utc_datetime(&naive_dt));
        }
        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, format) {
            return Ok(Utc.from_utc_datetime(&naive_date.and_hms_opt(0, 0, 0).unwrap()));
        }
    }

    Err("Invalid date format".to_string())
}

fn format_relative_time(timestamp: i64) -> String {
    let now = Utc::now().timestamp();
    let diff = now - timestamp;

    if diff == 0 {
        return "now".to_string();
    }

    let (abs_diff, suffix) = if diff > 0 {
        (diff, "ago")
    } else {
        (-diff, "from now")
    };

    match abs_diff {
        0..=59 => format!(
            "{} second{} {}",
            abs_diff,
            if abs_diff == 1 { "" } else { "s" },
            suffix
        ),
        60..=3599 => {
            let minutes = abs_diff / 60;
            format!(
                "{} minute{} {}",
                minutes,
                if minutes == 1 { "" } else { "s" },
                suffix
            )
        }
        3600..=86399 => {
            let hours = abs_diff / 3600;
            format!(
                "{} hour{} {}",
                hours,
                if hours == 1 { "" } else { "s" },
                suffix
            )
        }
        86400..=2591999 => {
            let days = abs_diff / 86400;
            format!(
                "{} day{} {}",
                days,
                if days == 1 { "" } else { "s" },
                suffix
            )
        }
        2592000..=31535999 => {
            let months = abs_diff / 2592000;
            format!(
                "{} month{} {}",
                months,
                if months == 1 { "" } else { "s" },
                suffix
            )
        }
        _ => {
            let years = abs_diff / 31536000;
            format!(
                "{} year{} {}",
                years,
                if years == 1 { "" } else { "s" },
                suffix
            )
        }
    }
}
