mod aes;
use std::{fmt, io::Read, str::FromStr};

pub(crate) use aes::AesGenerator;

/// The method to use for generating deterministic "garbage" data
#[derive(Debug, Clone, Copy)]
pub enum GarbageGeneratorVariant {
    /// AES, CTR mode with 128-bit little-endian counter.
    Aes,
}

impl fmt::Display for GarbageGeneratorVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GarbageGeneratorVariant::Aes => write!(f, "AES"),
        }
    }
}

impl FromStr for GarbageGeneratorVariant {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "aes" => Ok(GarbageGeneratorVariant::Aes),
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
    ) -> impl GarbageGenerator {
        match self {
            GarbageGeneratorVariant::Aes => AesGenerator::new(block_size, seed, progress),
        }
    }
}

/// A type that allows garbage generation via its [`Read`] implementation.
pub trait GarbageGenerator: Read {
    /// Creates a new garbage generator for the given block size and seed.
    fn new(block_size: usize, seed: u64, progress: Box<dyn Fn(u64)>) -> Self;
}
