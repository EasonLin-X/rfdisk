use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::PathBuf,
};

use crate::{model::DiskGuard, util::path::root_disk_name};

struct MountInfo {
    device_name: String,
    mount_point: String,
}

fn normalize_dev_name(dev_spec: &str) -> Option<String> {
    let dev_name = dev_spec.strip_prefix("/dev/")?;
    if dev_name.starts_with("mapper/") {
        fs::canonicalize(dev_spec)
            .ok()
            .and_then(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .or_else(|| Some(dev_name.to_string()))
    } else {
        Some(dev_name.to_string())
    }
}

fn mounted_block_devices() -> Vec<MountInfo> {
    let mut mounted = Vec::new();

    let Ok(content) = fs::read_to_string("/proc/mounts") else {
        return mounted;
    };

    for line in content.lines() {
        let mut parts = line.split_whitespace();
        let Some(dev_spec) = parts.next() else {
            continue;
        };
        let Some(mount_point) = parts.next() else {
            continue;
        };

        if let Some(device_name) = normalize_dev_name(dev_spec) {
            mounted.push(MountInfo {
                device_name,
                mount_point: mount_point.to_string(),
            });
        }
    }

    mounted
}

fn swap_block_devices() -> HashSet<String> {
    let mut swaps = HashSet::new();

    let Ok(content) = fs::read_to_string("/proc/swaps") else {
        return swaps;
    };

    for line in content.lines().skip(1) {
        let Some(dev_spec) = line.split_whitespace().next() else {
            continue;
        };
        if let Some(device_name) = normalize_dev_name(dev_spec) {
            swaps.insert(device_name);
        }
    }

    swaps
}

fn is_system_mount(mount_point: &str) -> bool {
    matches!(mount_point, "/" | "/boot" | "/boot/efi" | "/usr" | "/var")
}

fn underlying_root_disks(block_device: &str) -> HashSet<String> {
    let mut roots = HashSet::new();
    let mut queue = VecDeque::from([block_device.to_string()]);
    let mut seen = HashSet::new();

    while let Some(name) = queue.pop_front() {
        if !seen.insert(name.clone()) {
            continue;
        }

        let sys_path = PathBuf::from("/sys/class/block").join(&name);
        let slaves_path = sys_path.join("slaves");
        let mut found_slave = false;

        if let Ok(slaves) = fs::read_dir(slaves_path) {
            for slave in slaves.flatten() {
                found_slave = true;
                queue.push_back(slave.file_name().to_string_lossy().into_owned());
            }
        }

        if !found_slave {
            let root = root_disk_name(&name);
            if !root.is_empty() {
                roots.insert(root);
            }
        }
    }

    roots
}

fn update_guard(guards: &mut HashMap<String, DiskGuard>, disk: String, guard: DiskGuard) {
    let current = guards.get(&disk).copied().unwrap_or(DiskGuard::None);
    if guard.priority() > current.priority() {
        guards.insert(disk, guard);
    }
}

pub(crate) fn get_disk_guards() -> HashMap<String, DiskGuard> {
    let mut guards = HashMap::new();

    for mounted in mounted_block_devices() {
        let guard = if is_system_mount(&mounted.mount_point) {
            DiskGuard::System
        } else {
            DiskGuard::Mounted
        };

        for root in underlying_root_disks(&mounted.device_name) {
            update_guard(&mut guards, root, guard);
        }
    }

    for swap in swap_block_devices() {
        for root in underlying_root_disks(&swap) {
            update_guard(&mut guards, root, DiskGuard::Swap);
        }
    }

    guards
}

pub(crate) fn get_protected_disks() -> HashSet<String> {
    get_disk_guards()
        .into_iter()
        .filter_map(|(disk, guard)| guard.is_guarded().then_some(disk))
        .collect()
}
