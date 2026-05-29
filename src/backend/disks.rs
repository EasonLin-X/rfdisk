use std::{fs, path::Path};

use crate::{
    backend::{guards::get_disk_guards, sfdisk, udev},
    model::{DiskDevice, DiskGuard, DiskKind, ScanStatus},
    util::{
        path::{read_trimmed, read_u64},
        sector::sector_count_to_bytes,
    },
};

fn is_disk_device(name: &str) -> bool {
    name.starts_with("sd")
        || name.starts_with("nvme")
        || name.starts_with("vd")
        || name.starts_with("xvd")
        || name.starts_with("mmcblk")
}

fn detect_disk_kind(name: &str, sys_path: &Path) -> DiskKind {
    if name.starts_with("nvme") {
        return DiskKind::Nvme;
    }

    match read_trimmed(sys_path.join("queue/rotational")).as_deref() {
        Some("0") => DiskKind::Ssd,
        Some("1") => DiskKind::Hdd,
        _ => DiskKind::Unknown,
    }
}

pub(crate) fn scan_disks() -> Vec<DiskDevice> {
    let disk_guards = get_disk_guards();
    let mut disks = Vec::new();

    let Ok(entries) = fs::read_dir("/sys/block") else {
        return disks;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_disk_device(&name) {
            continue;
        }

        let sys_path = entry.path();
        let size_bytes = read_u64(sys_path.join("size"))
            .map(sector_count_to_bytes)
            .unwrap_or(0);
        let kind = detect_disk_kind(&name, &sys_path);
        let dev_path = format!("/dev/{name}");
        let udev = udev::read_udev_info(&dev_path);
        let serial = read_trimmed(sys_path.join("device/serial"))
            .or_else(|| read_trimmed(sys_path.join("serial")))
            .or_else(|| udev.get("ID_SERIAL_SHORT").map(str::to_string))
            .or_else(|| udev.get("ID_SERIAL").map(str::to_string))
            .unwrap_or_else(|| format!("NO_SERIAL_{name}"));
        let model = read_trimmed(sys_path.join("device/model"))
            .or_else(|| udev.get("ID_MODEL").map(str::to_string))
            .unwrap_or_else(|| "-".to_string());

        let (table_label, scan_status) = match sfdisk::read_layout(&dev_path) {
            Ok(layout) => (layout.label, ScanStatus::sfdisk()),
            Err(err) => (
                crate::model::DetectedTableLabel::from_sfdisk_label(None),
                ScanStatus::sysfs_fallback(err),
            ),
        };

        disks.push(DiskDevice {
            table_label,
            dev_path,
            guard: disk_guards.get(&name).copied().unwrap_or(DiskGuard::None),
            name,
            size_bytes,
            kind,
            serial,
            model,
            scan_status,
        });
    }

    disks.sort_by(|a, b| a.name.cmp(&b.name));
    disks
}
