//! Running the "write" portion of the test.

use crate::{garbage::GarbageGenerator, IOBuffer, OPEN_FLAGS, PROGRESS_STYLE};
use anyhow::Context;
use std::os::unix::fs::OpenOptionsExt as _;
use std::{fs::OpenOptions, io, path::Path};
use tracing::info_span;
use tracing_indicatif::span_ext::IndicatifSpanExt;

#[tracing::instrument(skip(generator))]
pub(crate) fn write(
    dev_path: &Path,
    generator: Box<dyn GarbageGenerator>,
    buffer_size: usize,
) -> anyhow::Result<usize> {
    let capacity = crate::determine_size(dev_path)?;
    let out = OpenOptions::new()
        .write(true)
        .custom_flags(OPEN_FLAGS)
        .open(dev_path)
        .with_context(|| format!("Opening the device {dev_path:?} for writing"))?;
    write_garbage(out, generator, capacity, buffer_size)
}

fn write_garbage(
    mut blockdev: impl io::Write,
    mut generator: impl io::Read,
    capacity: u64,
    buffer_size: usize,
) -> anyhow::Result<usize> {
    let bar_span = info_span!("writing");
    bar_span.pb_set_style(&PROGRESS_STYLE);
    bar_span.pb_set_length(capacity);
    let _bar_span_handle = bar_span.enter();

    let mut buf = IOBuffer::with_capacity(buffer_size);
    buf.resize(buffer_size, 0);
    let mut done = 0;
    loop {
        generator
            .read_exact(&mut buf)
            .context("Generating pseudorandom data")?;
        match blockdev.write_all(&buf) {
            Ok(_) => {}
            Err(e) if e.raw_os_error() == Some(28) => {
                // "disk full", meaning we're done:
                return Ok(done);
            }
            Err(e) if e.kind() == io::ErrorKind::WriteZero => {
                // "disk full" on macOS, meaning we're done:
                return Ok(done);
            }
            Err(e) => anyhow::bail!("io Error at offset={done:?} {:?}: kind {:?}", e, e.kind()),
        };
        done += buffer_size;
        bar_span.pb_inc(buffer_size as u64);
    }
}
