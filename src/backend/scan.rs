use std::{collections::HashSet, fs, path::PathBuf};

use crate::{
    backend::{blkid, lsblk, sfdisk},
    model::{DiskDevice, PartitionInfo},
    util::{
        path::{read_u64, root_disk_name},
        sector::sector_count_to_bytes,
    },
};

pub(crate) fn scan_partitions(disk: &DiskDevice) -> Vec<PartitionInfo> {
    let mut partitions = sfdisk::read_layout(&disk.dev_path)
        .map(|layout| layout.partitions)
        .unwrap_or_else(|_| scan_partitions_from_sysfs(disk));

    let lsblk_map = lsblk::read_lsblk_map();
    let swaps = read_swap_devices();

    for partition in &mut partitions {
        if let Some(info) = lsblk_map.get(&partition.dev_path) {
            if partition.fs_type.is_empty() {
                partition.fs_type = info.fs_type.clone();
            }
            if partition.label.is_empty() {
                partition.label = info.label.clone();
            }
            if partition.uuid.is_empty() {
                partition.uuid = info.uuid.clone();
            }
            if partition.part_uuid.is_empty() {
                partition.part_uuid = info.part_uuid.clone();
            }
            partition.mount_points = info.mount_points.clone();
        }

        let blkid = blkid::read_blkid(&partition.dev_path);
        if partition.fs_type.is_empty() {
            partition.fs_type = blkid.fs_type;
        }
        if partition.uuid.is_empty() {
            partition.uuid = blkid.uuid;
        }
        if partition.part_uuid.is_empty() {
            partition.part_uuid = blkid.part_uuid;
        }
        if partition.label.is_empty() {
            partition.label = blkid.label;
        }
        partition.is_swap = swaps.contains(&partition.dev_path);
    }

    partitions.sort_by(|a, b| a.start_sector.cmp(&b.start_sector));
    partitions
}

fn scan_partitions_from_sysfs(disk: &DiskDevice) -> Vec<PartitionInfo> {
    let disk_path = PathBuf::from("/sys/block").join(&disk.name);
    let Ok(entries) = fs::read_dir(disk_path) else {
        return Vec::new();
    };

    let mut partitions = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if root_disk_name(&name) != disk.name || name == disk.name {
            continue;
        }

        let path = entry.path();
        let start_sector = read_u64(path.join("start")).unwrap_or(0);
        let size_bytes = read_u64(path.join("size"))
            .map(sector_count_to_bytes)
            .unwrap_or(0);

        partitions.push(PartitionInfo {
            dev_path: format!("/dev/{name}"),
            start_sector,
            size_bytes,
            fs_type: String::new(),
            partition_type: "partition".to_string(),
            partition_type_raw: String::new(),
            uuid: String::new(),
            part_uuid: String::new(),
            label: String::new(),
            part_name: String::new(),
            mount_points: Vec::new(),
            is_swap: false,
        });
    }

    partitions
}

fn read_swap_devices() -> HashSet<String> {
    let Ok(content) = fs::read_to_string("/proc/swaps") else {
        return HashSet::new();
    };

    content
        .lines()
        .skip(1)
        .filter_map(|line| line.split_whitespace().next().map(str::to_string))
        .collect()
}
