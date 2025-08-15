use crate::{
    Context, Error,
    util::{
        command::{
            check_cooldown, create_error_response, create_success_response, validate_input_size,
        },
        crypto::{calculate_adler32, calculate_crc32},
    },
};

#[derive(poise::ChoiceParameter)]
pub enum ChecksumAlgorithm {
    #[name = "CRC32"]
    CRC32,
    #[name = "Adler32"]
    Adler32,
}

impl ChecksumAlgorithm {
    fn name(&self) -> &'static str {
        match self {
            ChecksumAlgorithm::CRC32 => "CRC32",
            ChecksumAlgorithm::Adler32 => "Adler32",
        }
    }
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Calculate checksums of data for integrity verification")
)]
pub async fn checksum(
    ctx: Context<'_>,
    #[description = "Checksum algorithm to use"] algorithm: ChecksumAlgorithm,
    #[description = "The data to calculate checksum for"] data: String,
) -> Result<(), Error> {
    check_cooldown(&ctx, "checksum", ctx.data().config.cooldowns.hash_cooldown).await?;

    if let Err(e) = validate_input_size(&data, &ctx.data().config) {
        let embed = create_error_response("Checksum Error", &e.to_string());
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let (title, result) = match algorithm {
        ChecksumAlgorithm::CRC32 => {
            let checksum = calculate_crc32(data.as_bytes());
            (
                format!("{} Checksum", algorithm.name()),
                format!("{:08x}", checksum),
            )
        }
        ChecksumAlgorithm::Adler32 => {
            let checksum = calculate_adler32(data.as_bytes());
            (
                format!("{} Checksum", algorithm.name()),
                format!("{:08x}", checksum),
            )
        }
    };

    let embed = create_success_response(&title, &result, true, &ctx.data().config);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
