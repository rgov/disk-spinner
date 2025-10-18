use std::io;

use rand::{RngCore as _, SeedableRng as _};
use rand_chacha::ChaCha8Rng;

use super::GarbageGenerator;

pub struct Blake3Generator {
    buf: Vec<u8>,
    hasher: blake3::Hasher,
    lba: usize,
}

impl GarbageGenerator for Blake3Generator {}

impl Blake3Generator {
    /// Generate a new Blake3 garbage generator for a block size from a random seed.
    pub(super) fn new(block_size: usize, seed: u64) -> Self {
        let buf = vec![0; block_size];

        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut key = [0; 32];
        rng.fill_bytes(&mut key);
        let hasher = blake3::Hasher::new_keyed(&key);

        Self {
            buf,
            hasher,
            lba: 0,
        }
    }
}

/// GarbageGenerator implements Read in order to supply the write test
/// with random data that can be copied to disk.
impl io::Read for Blake3Generator {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut done = 0;
        for chunk in buf.chunks_exact_mut(self.buf.len()) {
            let length = chunk.len();
            let mut chunk = io::Cursor::new(chunk);
            self.hasher.update(&self.lba.to_le_bytes());
            let reader = self.hasher.finalize_xof();
            io::copy(&mut reader.take(length.try_into().unwrap()), &mut chunk)?;
            self.hasher.reset();
            done += length;
            self.lba += 1;
        }
        Ok(done)
    }
}
