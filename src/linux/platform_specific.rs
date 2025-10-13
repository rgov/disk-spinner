extern crate block_utils;
use crate::Args;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use tracing::warn;

#[derive(Debug, Clone)]
pub(crate) struct ValidDevice {
    pub path: PathBuf,
    pub partition: Option<u64>,
    pub device: block_utils::Device,
}

impl FromStr for ValidDevice {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (partition, device) = block_utils::get_device_from_path(s)?;
        Ok(Self {
            path: PathBuf::from(s),
            partition,
            device: device.ok_or(anyhow::anyhow!(
                "The device under test must be a valid block device."
            ))?,
        })
    }
}

pub(crate) fn sanity_checks(
    args: &Args,
    partition: Option<u64>,
    device_path: &Path,
    device: &block_utils::Device,
) -> anyhow::Result<()> {
    // Sanity checks:
    if partition.is_some() {
        if !args.allow_any_block_device {
            anyhow::bail!("Device is not a whole disk but a partition - pass --allow-any-block-device to run tests anyway.");
        } else {
            warn!(
                ?partition,
                ?device_path,
                "Testing a partition but running tests anyway."
            );
        }
    }
    if device.media_type != block_utils::MediaType::Rotational {
        if !args.allow_any_media {
            anyhow::bail!("Device is not a rotational disk - this tool may be harmful to solid-state drives and others! Pass --allow-any-media to run anyway.");
        } else {
            warn!(?device.media_type, ?device_path, "Media type is not as expected but running tests anyway.");
        }
    }
    let child_partitions: Vec<PathBuf> = block_utils::get_block_partitions_iter()?
        .filter(|part_path| {
            part_path
                .file_name()
                .map(|name| name.to_string_lossy().starts_with(&device.name))
                .unwrap_or(false)
        })
        .collect();

    if !child_partitions.is_empty() {
        anyhow::bail!("Detected child partitions on the device - I won't help you destroy an in-use drive: Delete those partitions yourself. Partitions found: {child_partitions:?}", );
    }
    Ok(())
}
