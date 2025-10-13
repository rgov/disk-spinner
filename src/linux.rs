#[cfg(target_os = "linux")]
mod platform_specific;
use std::path::PathBuf;

#[cfg(target_os = "linux")]
pub(crate) use platform_specific::*;

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn child_partitions(
    device_name: &str,
    block_partitions: impl Iterator<Item = PathBuf>,
) -> Vec<PathBuf> {
    block_partitions
        .filter(|part_path| {
            part_path
                .file_name()
                .and_then(|name| {
                    name.to_string_lossy()
                        .strip_prefix(device_name)
                        .unwrap_or_default()
                        .parse::<usize>()
                        .ok()
                        .map(|_| true)
                })
                .unwrap_or(false)
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case("sda", &["/dev/sdb1", "/dev/sda1"], &["/dev/sda1"]; "a normal partition of the given device")]
    #[test_case("sda", &["/dev/sdb1"], &[]; "no-partitions")]
    #[test_case("sda", &["/dev/sda", "/dev/sdb", "/dev/sdai"], &[]; "block devices above 26 are present")]
    fn detects_child_partitions(dev: &str, existing: &[&str], should: &[&str]) {
        let detected = child_partitions(dev, existing.iter().map(PathBuf::from));
        let should: Vec<PathBuf> = should.iter().map(PathBuf::from).collect();
        assert_eq!(detected, should);
    }
}
