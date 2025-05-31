use crate::{Context, Error, util::command::{check_cooldown, create_success_response}};

#[derive(poise::ChoiceParameter)]
pub enum UuidVersion {
    #[name = "Version 1 (Timestamp + MAC)"]
    V1,
    #[name = "Version 4 (Random)"]
    V4,
    #[name = "Version 7 (Timestamp + Random)"]
    V7,
}

impl UuidVersion {
    fn generate(&self) -> uuid::Uuid {
        match self {
            UuidVersion::V1 => uuid::Uuid::now_v1(&[1, 2, 3, 4, 5, 6]),
            UuidVersion::V4 => uuid::Uuid::new_v4(),
            UuidVersion::V7 => uuid::Uuid::now_v7(),
        }
    }

    fn description(&self) -> &'static str {
        match self {
            UuidVersion::V1 => "**Version 1** - Timestamp + MAC address based",
            UuidVersion::V4 => "**Version 4** - Random/pseudo-random",
            UuidVersion::V7 => "**Version 7** - Unix timestamp + random",
        }
    }
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Generate UUIDs (Universally Unique Identifiers)")
)]
pub async fn uuid(
    ctx: Context<'_>,
    #[description = "UUID version to generate"] version: Option<UuidVersion>,
    #[description = "Number of UUIDs to generate (1-10)"]
    #[min = 1]
    #[max = 10]
    count: Option<u8>,
    #[description = "Show UUID breakdown and information"] analyze: Option<bool>,
) -> Result<(), Error> {
    check_cooldown(&ctx, "uuid", ctx.data().config.cooldowns.per_user_cooldown).await?;

    let version = version.unwrap_or(UuidVersion::V4);
    let count = count.unwrap_or(1);
    let analyze = analyze.unwrap_or(false);

    let mut uuids = Vec::new();
    let mut descriptions = Vec::new();

    for _ in 0..count {
        let uuid = version.generate();
        uuids.push(uuid);

        if analyze {
            descriptions.push(analyze_uuid(&uuid, &version));
        }
    }

    let uuid_list = if count == 1 {
        format!("`{}`", uuids[0])
    } else {
        uuids
            .iter()
            .enumerate()
            .map(|(i, uuid)| format!("{}. `{}`", i + 1, uuid))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let mut description = format!(
        "{}\n\n**Generated UUID{}:**\n{}",
        version.description(),
        if count > 1 { "s" } else { "" },
        uuid_list
    );

    if analyze && !descriptions.is_empty() {
        description.push_str("\n\n**Analysis:**");
        for (i, desc) in descriptions.iter().enumerate() {
            if count > 1 {
                description.push_str(&format!("\n\n**UUID {}:**", i + 1));
            }
            description.push_str(&format!("\n{}", desc));
        }
    }

    if count == 1 {
        let uuid = &uuids[0];
        description.push_str(&format!(
            "\n\n**Formats:**\n• **Standard:** `{}`\n• **Hyphenated:** `{}`\n• **URN:** \
             `urn:uuid:{}`\n• **Braced:** `{{{}}}`\n• **Hex:** `{}`",
            uuid.hyphenated(),
            uuid.hyphenated(),
            uuid.hyphenated(),
            uuid.hyphenated(),
            uuid.simple()
        ));
    }

    let title = format!(
        "UUID {}",
        match version {
            UuidVersion::V1 => "Version 1",
            UuidVersion::V4 => "Version 4",
            UuidVersion::V7 => "Version 7",
        }
    );

    let embed = create_success_response(&title, &description, false, &ctx.data().config);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

fn analyze_uuid(uuid: &uuid::Uuid, version: &UuidVersion) -> String {
    match version {
        UuidVersion::V1 => analyze_uuid_v1(uuid),
        UuidVersion::V4 => analyze_uuid_v4(),
        UuidVersion::V7 => analyze_uuid_v7(uuid),
    }
}

fn analyze_uuid_v1(uuid: &uuid::Uuid) -> String {
    let bytes = uuid.as_bytes();

    let time_low = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let time_mid = u16::from_be_bytes([bytes[4], bytes[5]]);
    let time_hi = u16::from_be_bytes([bytes[6], bytes[7]]) & 0x0FFF;

    let timestamp = ((time_hi as u64) << 48) | ((time_mid as u64) << 32) | (time_low as u64);

    let clock_seq = ((bytes[8] & 0x3F) as u16) << 8 | bytes[9] as u16;

    let node = format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    );

    format!(
        "• **Timestamp:** {} (100ns intervals since 1582)\n• **Clock Sequence:** {}\n• **Node \
         (MAC):** {}",
        timestamp, clock_seq, node
    )
}

fn analyze_uuid_v4() -> String {
    format!(
        "• **Random Bytes:** 122 bits of randomness\n• **Collision Probability:** ~2^-122 \
         (astronomically low)"
    )
}

fn analyze_uuid_v7(uuid: &uuid::Uuid) -> String {
    let bytes = uuid.as_bytes();

    let timestamp_ms = u64::from_be_bytes([
        0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
    ]);

    let datetime = chrono::DateTime::from_timestamp_millis(timestamp_ms as i64)
        .unwrap_or_else(|| chrono::Utc::now());

    format!(
        "• **Timestamp:** {} UTC\n• **Milliseconds:** {}\n• **Random Data:** 74 bits",
        datetime.format("%Y-%m-%d %H:%M:%S%.3f"),
        timestamp_ms
    )
}