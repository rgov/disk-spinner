mod aes;
mod blake3;
use std::{fmt, io::Read, str::FromStr};

/// The method to use for generating deterministic "garbage" data
#[derive(Debug, Clone, Copy)]
pub enum GarbageGeneratorVariant {
    /// AES, CTR mode with 128-bit little-endian counter.
    Aes,

    /// The BLAKE3 cryptographic hash function; slightly faster than AES on Apple Silicon hardware.
    Blake3,
}

impl fmt::Display for GarbageGeneratorVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GarbageGeneratorVariant::Aes => write!(f, "AES"),
            GarbageGeneratorVariant::Blake3 => write!(f, "BLAKE3"),
        }
    }
}

impl FromStr for GarbageGeneratorVariant {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "aes" => Ok(GarbageGeneratorVariant::Aes),
            "blake3" => Ok(GarbageGeneratorVariant::Blake3),
            _ => Err(anyhow::anyhow!("Unknown garbage generator variant {s}")),
        }
    }
}

impl GarbageGeneratorVariant {
    /// Create a new garbage generator for the specified type. You need one for each write step and each read step, per device.
    pub fn to_generator(
        self,
        block_size: usize,
        seed: u64,
        progress: Box<dyn Fn(u64)>,
    ) -> Box<dyn GarbageGenerator> {
        match self {
            GarbageGeneratorVariant::Aes => {
                Box::new(aes::AesGenerator::new(block_size, seed, progress))
            }
            GarbageGeneratorVariant::Blake3 => {
                Box::new(blake3::Blake3Generator::new(block_size, seed, progress))
            }
        }
    }
}

/// A type that allows garbage generation via its [`Read`] implementation.
pub trait GarbageGenerator: Read {}
