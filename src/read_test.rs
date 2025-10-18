//! Running the "read back" portion of the test.

use crate::{garbage::GarbageGenerator, IOBuffer, OPEN_FLAGS, PROGRESS_STYLE};
use anyhow::Context;
use std::{
    fs::OpenOptions,
    io::{BufReader, Read},
    os::unix::fs::OpenOptionsExt as _,
    path::Path,
};
use tracing::{info_span, warn};
use tracing_indicatif::span_ext::IndicatifSpanExt;

type FailedReads = usize;

#[tracing::instrument(skip(generator))]
pub(crate) fn read_back(
    dev_path: &Path,
    generator: Box<dyn GarbageGenerator>,
    buffer_size: usize,
    written: usize,
) -> anyhow::Result<Result<(), FailedReads>> {
    let blockdev = OpenOptions::new()
        .read(true)
        .custom_flags(OPEN_FLAGS)
        .open(dev_path)
        .with_context(|| format!("Opening the device {dev_path:?} for reading"))?;

    let generator = BufReader::new(generator);
    let mismatched = compare_persisted_bytes(blockdev, generator, buffer_size, written)?;
    if mismatched > 0 {
        return Ok(Err(mismatched));
    }
    Ok(Ok(()))
}

fn compare_persisted_bytes(
    mut blockdev: impl Read,
    mut generator: impl Read,
    buffer_size: usize,
    written: usize,
) -> anyhow::Result<usize> {
    let bar_span = info_span!("reading back");
    bar_span.pb_set_style(&PROGRESS_STYLE);
    bar_span.pb_set_length(written as u64);
    let _bar_span_handle = bar_span.enter();
    let mut mismatches = 0;
    let mut offset = 0;
    let mut should = IOBuffer::with_capacity(buffer_size);
    should.resize(buffer_size, 0);
    let mut have = IOBuffer::with_capacity(buffer_size);
    have.resize(buffer_size, 0);
    while offset < written {
        generator
            .read_exact(&mut should)
            .context("Generating pseudorandom data")?;
        match blockdev.read_exact(&mut have) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            error => error.map_err(|e| anyhow::anyhow!("Reading bytes on disk: {:?}", e))?,
        }
        if have.is_empty() {
            break;
        }
        if *have != *should {
            warn!(offset, "Did not read back the exact bytes written");
            mismatches += 1;
        }
        offset += buffer_size;
        bar_span.pb_inc(buffer_size as u64);
    }
    if offset != written {
        warn!(
            validated = offset,
            written, "Number of bytes validated and written is not the same."
        );
    }
    Ok(mismatches)
}

#[cfg(test)]
mod test {
    use super::compare_persisted_bytes;
    use std::io;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn detects_issues() {
        let input: Vec<u8> = vec![1; 1024 * 1024];
        let mut read_back: Vec<u8> = vec![1; 1024 * 1024];
        read_back[1024 * 512] = 255; // corrupt our read-back data
        let read_back = io::Cursor::new(read_back);

        let mismatched =
            compare_persisted_bytes(read_back, io::Cursor::new(input), 1024, 1024 * 1024).unwrap();
        assert_eq!(mismatched, 1);
    }

    #[traced_test]
    #[test]
    fn succeeds() {
        let input: Vec<u8> = vec![1; 1024 * 1024];
        let read_back: Vec<u8> = vec![1; 1024 * 1024];
        let read_back = io::Cursor::new(read_back);
        let mismatched =
            compare_persisted_bytes(read_back, io::Cursor::new(input), 1024, 1024 * 1024).unwrap();
        assert_eq!(mismatched, 0);
    }
}
