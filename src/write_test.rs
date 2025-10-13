//! Running the "write" portion of the test.

use crate::{garbage::GarbageGenerator, OPEN_FLAGS, PROGRESS_STYLE};
use anyhow::Context;
use std::os::unix::fs::OpenOptionsExt as _;
use std::{
    fs::OpenOptions,
    io::{self, BufReader},
    path::Path,
};
use tracing::info_span;
use tracing_indicatif::span_ext::IndicatifSpanExt;

#[tracing::instrument(skip(generator))]
pub(crate) fn write(dev_path: &Path, generator: Box<dyn GarbageGenerator>) -> anyhow::Result<()> {
    let capacity = crate::determine_size(dev_path)?;
    let mut out = OpenOptions::new()
        .write(true)
        .custom_flags(OPEN_FLAGS)
        .open(dev_path)
        .with_context(|| format!("Opening the device {dev_path:?} for writing"))?;

    let bar_span = info_span!("writing");
    bar_span.pb_set_style(&PROGRESS_STYLE);
    bar_span.pb_set_length(capacity);
    let _bar_span_handle = bar_span.enter();

    let mut generator = BufReader::new(generator);
    match io::copy(&mut generator, &mut out) {
        Ok(_) => Ok(()),
        Err(e) if e.raw_os_error() == Some(28) => {
            // "disk full", meaning we're done:
            Ok(())
        }
        Err(e) if e.kind() == io::ErrorKind::WriteZero => {
            // "disk full" on macOS, meaning we're done:
            Ok(())
        }
        Err(e) => anyhow::bail!("io Error {:?}: kind {:?}", e, e.kind()),
    }
}
