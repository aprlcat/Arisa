use blake3::Hasher as Blake3Hasher;
use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};

#[derive(Debug, Clone)]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    #[allow(dead_code)]
    Blake3,
}

impl HashAlgorithm {
    pub fn hash(&self, input: &[u8]) -> String {
        match self {
            HashAlgorithm::Md5 => {
                let digest = md5::compute(input);
                format!("{:x}", digest)
            }
            HashAlgorithm::Sha1 => {
                let mut hasher = Sha1::new();
                hasher.update(input);
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::Sha224 => {
                let mut hasher = Sha224::new();
                hasher.update(input);
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(input);
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::Sha384 => {
                let mut hasher = Sha384::new();
                hasher.update(input);
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(input);
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::Blake3 => {
                let mut hasher = Blake3Hasher::new();
                hasher.update(input);
                format!("{}", hasher.finalize().to_hex())
            }
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            HashAlgorithm::Md5 => "MD5",
            HashAlgorithm::Sha1 => "SHA-1",
            HashAlgorithm::Sha224 => "SHA-224",
            HashAlgorithm::Sha256 => "SHA-256",
            HashAlgorithm::Sha384 => "SHA-384",
            HashAlgorithm::Sha512 => "SHA-512",
            HashAlgorithm::Blake3 => "BLAKE3",
        }
    }
}

pub fn calculate_crc32(input: &[u8]) -> u32 {
    crc32fast::hash(input)
}

#[allow(dead_code)]
pub fn calculate_checksum(input: &[u8]) -> u32 {
    input.iter().map(|&b| b as u32).sum()
}

pub fn calculate_adler32(input: &[u8]) -> u32 {
    adler::adler32_slice(input)
}
