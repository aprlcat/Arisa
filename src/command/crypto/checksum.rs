use crate::{
    Context, Error,
    util::{
        command::{create_error_response, create_success_response, validate_input_size},
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
    if let Err(e) = validate_input_size(&data) {
        let embed = create_error_response("Checksum Error", &e);
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

    let embed = create_success_response(&title, &result, true);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
