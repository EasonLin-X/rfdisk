use serde::Deserialize;
use std::{collections::HashMap, time::Duration};

use crate::backend::cmd::run_command;

#[derive(Clone, Debug, Default)]
pub struct LsblkInfo {
    pub fs_type: String,
    pub label: String,
    pub uuid: String,
    pub part_uuid: String,
    pub mount_points: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LsblkRoot {
    blockdevices: Vec<LsblkDevice>,
}

#[derive(Debug, Deserialize)]
struct LsblkDevice {
    path: Option<String>,
    fstype: Option<String>,
    label: Option<String>,
    uuid: Option<String>,
    partuuid: Option<String>,
    mountpoints: Option<Vec<Option<String>>>,
    children: Option<Vec<LsblkDevice>>,
}

pub fn read_lsblk_map() -> HashMap<String, LsblkInfo> {
    let Ok(output) = run_command(
        "lsblk",
        &[
            "--json",
            "-o",
            "PATH,FSTYPE,LABEL,UUID,PARTUUID,MOUNTPOINTS",
        ],
        Duration::from_secs(3),
    ) else {
        return HashMap::new();
    };
    if output.status != Some(0) {
        return HashMap::new();
    }

    let Ok(root) = serde_json::from_str::<LsblkRoot>(&output.stdout) else {
        return HashMap::new();
    };

    let mut map = HashMap::new();
    for device in root.blockdevices {
        collect_device(device, &mut map);
    }
    map
}

fn collect_device(device: LsblkDevice, map: &mut HashMap<String, LsblkInfo>) {
    if let Some(path) = device.path {
        map.insert(
            path,
            LsblkInfo {
                fs_type: device.fstype.unwrap_or_default(),
                label: device.label.unwrap_or_default(),
                uuid: device.uuid.unwrap_or_default(),
                part_uuid: device.partuuid.unwrap_or_default(),
                mount_points: device
                    .mountpoints
                    .unwrap_or_default()
                    .into_iter()
                    .flatten()
                    .collect(),
            },
        );
    }

    for child in device.children.unwrap_or_default() {
        collect_device(child, map);
    }
}
