//! Running the "read back" portion of the test.

use crate::{garbage::GarbageGenerator, PROGRESS_STYLE};
use anyhow::Context;
use std::{
    fs::OpenOptions,
    io::{self, BufReader, Seek},
    path::Path,
};
use tracing::{info_span, warn};
use tracing_indicatif::span_ext::IndicatifSpanExt;

type FailedReads = usize;

#[tracing::instrument(skip(generator))]
pub(crate) fn read_back(
    dev_path: &Path,
    generator: Box<dyn GarbageGenerator>,
) -> anyhow::Result<Result<(), FailedReads>> {
    let mut blockdev = OpenOptions::new()
        .read(true)
        .open(dev_path)
        .with_context(|| format!("Opening the device {dev_path:?} for reading"))?;
    let capacity = blockdev.seek(io::SeekFrom::End(0))?;
    blockdev.seek(io::SeekFrom::Start(0))?;

    let bar_span = info_span!("reading back");
    bar_span.pb_set_style(&PROGRESS_STYLE);
    bar_span.pb_set_length(capacity);
    let _bar_span_handle = bar_span.enter();

    let generator = BufReader::new(generator);
    let mut compare = CompareWriter::new(generator);
    io::copy(&mut blockdev, &mut compare)?;
    if compare.mismatched > 0 {
        return Ok(Err(compare.mismatched));
    }
    Ok(Ok(()))
}

/// A struct that pretends to be [io::Write] by doing block-by-block comparisons against another reader.
#[derive(Debug)]
struct CompareWriter<R: io::Read> {
    compare: R,
    mismatched: usize,
    current_offset: usize,
}

impl<R: io::Read> CompareWriter<R> {
    fn new(compare: R) -> Self {
        Self {
            compare,
            mismatched: 0,
            current_offset: 0,
        }
    }
}

impl<R: io::Read> io::Write for CompareWriter<R> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut read = vec![0; buf.len()];
        self.compare.read_exact(&mut read)?;
        self.current_offset += buf.len();
        if read != buf {
            warn!(
                offset = self.current_offset,
                "Did not read back the exact bytes written"
            );
            self.mismatched += 1;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::CompareWriter;
    use std::io;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn detects_issues() {
        let input: Vec<u8> = vec![1; 1024 * 1024];
        let mut read_back: Vec<u8> = vec![1; 1024 * 1024];
        read_back[1024 * 512] = 255; // corrupt our read-back data
        let mut read_back = io::Cursor::new(read_back);

        let mut compare = CompareWriter::new(io::Cursor::new(input));
        io::copy(&mut read_back, &mut compare).expect("No io errors");
        assert_eq!(compare.mismatched, 1);
    }

    #[traced_test]
    #[test]
    fn succeeds() {
        let input: Vec<u8> = vec![1; 1024 * 1024];
        let read_back: Vec<u8> = vec![1; 1024 * 1024];
        let mut read_back = io::Cursor::new(read_back);
        let mut compare = CompareWriter::new(io::Cursor::new(input));
        io::copy(&mut read_back, &mut compare).expect("No io errors");
        assert_eq!(compare.mismatched, 0);
    }
}
