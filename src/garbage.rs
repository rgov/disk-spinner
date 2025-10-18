mod aes;
mod blake3;
mod shishua;
use std::{fmt, io::Read, str::FromStr};

/// The method to use for generating deterministic "garbage" data
#[derive(Debug, Clone, Copy, Default)]
pub enum GarbageGeneratorVariant {
    #[cfg_attr(not(feature = "shishua-cli"), default)]
    /// AES, CTR mode with 128-bit little-endian counter.
    Aes,

    /// The BLAKE3 cryptographic hash function; slightly faster than AES on Apple Silicon hardware.
    Blake3,

    #[cfg_attr(feature = "shishua-cli", default)]
    #[cfg(feature = "shishua-cli")]
    /// The `shishua` RNG, invoked via the cli tool of the same name.
    ShishuaCli,
}

impl fmt::Display for GarbageGeneratorVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GarbageGeneratorVariant::Aes => write!(f, "AES"),
            GarbageGeneratorVariant::Blake3 => write!(f, "BLAKE3"),
            #[cfg(feature = "shishua-cli")]
            GarbageGeneratorVariant::ShishuaCli => write!(f, "shishua"),
        }
    }
}

impl FromStr for GarbageGeneratorVariant {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "aes" => Ok(GarbageGeneratorVariant::Aes),
            "blake3" => Ok(GarbageGeneratorVariant::Blake3),

            #[cfg(feature = "shishua-cli")]
            "shishua" => Ok(GarbageGeneratorVariant::ShishuaCli),

            _ => Err(anyhow::anyhow!("Unknown garbage generator variant {s}")),
        }
    }
}

impl GarbageGeneratorVariant {
    /// Create a new garbage generator for the specified type. You need one for each write step and each read step, per device.
    pub fn to_generator(self, block_size: usize, seed: u64) -> Box<dyn GarbageGenerator> {
        match self {
            GarbageGeneratorVariant::Aes => Box::new(aes::AesGenerator::new(block_size, seed)),
            GarbageGeneratorVariant::Blake3 => {
                Box::new(blake3::Blake3Generator::new(block_size, seed))
            }
            #[cfg(feature = "shishua-cli")]
            GarbageGeneratorVariant::ShishuaCli => Box::new(
                shishua::ShishuaCliGenerator::new(seed).expect("shishua child process should work"),
            ),
        }
    }
}

/// A type that allows garbage generation via its [`Read`] implementation.
pub trait GarbageGenerator: Read {}
