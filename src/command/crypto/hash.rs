use crate::{
    Context, Error,
    util::{
        command::{check_cooldown, create_error_response, create_success_response, validate_input_size},
        crypto::HashAlgorithm,
    },
};

#[derive(poise::ChoiceParameter)]
pub enum HashChoice {
    #[name = "MD5"]
    MD5,
    #[name = "SHA1"]
    SHA1,
    #[name = "SHA224"]
    SHA224,
    #[name = "SHA256"]
    SHA256,
    #[name = "SHA384"]
    SHA384,
    #[name = "SHA512"]
    SHA512,
}

impl HashChoice {
    fn to_algorithm(&self) -> HashAlgorithm {
        match self {
            HashChoice::MD5 => HashAlgorithm::Md5,
            HashChoice::SHA1 => HashAlgorithm::Sha1,
            HashChoice::SHA224 => HashAlgorithm::Sha224,
            HashChoice::SHA256 => HashAlgorithm::Sha256,
            HashChoice::SHA384 => HashAlgorithm::Sha384,
            HashChoice::SHA512 => HashAlgorithm::Sha512,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            HashChoice::MD5 => "MD5",
            HashChoice::SHA1 => "SHA1",
            HashChoice::SHA224 => "SHA224",
            HashChoice::SHA256 => "SHA256",
            HashChoice::SHA384 => "SHA384",
            HashChoice::SHA512 => "SHA512",
        }
    }
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Generate cryptographic hashes of data")
)]
pub async fn hash(
    ctx: Context<'_>,
    #[description = "Hash algorithm to use"] algorithm: HashChoice,
    #[description = "The data to hash"] data: String,
) -> Result<(), Error> {
    check_cooldown(&ctx, "hash", ctx.data().config.cooldowns.hash_cooldown).await?;

    if let Err(e) = validate_input_size(&data, &ctx.data().config) {
        let embed = create_error_response("Hash Error", &e.to_string());
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let hash_algo = algorithm.to_algorithm();
    let hash_result = hash_algo.hash(data.as_bytes());
    let title = format!("{} Hash", algorithm.name());
    let embed = create_success_response(&title, &hash_result, true, &ctx.data().config);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}