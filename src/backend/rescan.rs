use std::{fs, io, time::Duration};

use crate::backend::guards::get_protected_disks;

pub(crate) fn trigger_scsi_scan() -> io::Result<()> {
    let protected_disks = get_protected_disks();

    if let Ok(entries) = fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with("sd") || protected_disks.contains(&name) {
                continue;
            }

            let delete_path = entry.path().join("device/delete");
            if delete_path.exists() {
                fs::write(delete_path, "1")?;
            }
        }
    }

    std::thread::sleep(Duration::from_millis(100));

    let scan_paths = glob::glob("/sys/class/scsi_host/host*/scan").map_err(io::Error::other)?;

    for path in scan_paths.flatten() {
        fs::write(path, "- - -")?;
    }

    std::thread::sleep(Duration::from_millis(200));
    Ok(())
}
