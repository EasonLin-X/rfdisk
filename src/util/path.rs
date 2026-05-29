use std::{
    fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
    time::UNIX_EPOCH,
};

pub(crate) fn create_log_path() -> io::Result<PathBuf> {
    let log_dir = PathBuf::from("/var/log/rfdisk");
    fs::create_dir_all(&log_dir)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Ok(log_dir.join(format!("{stamp}.log")))
}

pub(crate) fn read_trimmed(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn read_u64(path: impl AsRef<Path>) -> Option<u64> {
    read_trimmed(path)?.parse().ok()
}

pub(crate) fn root_disk_name(device_name: &str) -> String {
    if device_name.starts_with("nvme") || device_name.starts_with("mmcblk") {
        device_name
            .find('p')
            .map(|pos| device_name[..pos].to_string())
            .unwrap_or_else(|| device_name.to_string())
    } else {
        device_name
            .trim_end_matches(|ch: char| ch.is_ascii_digit())
            .to_string()
    }
}
