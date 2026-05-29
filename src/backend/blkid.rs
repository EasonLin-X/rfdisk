use std::{collections::HashMap, time::Duration};

use crate::backend::cmd::run_command;

#[derive(Clone, Debug, Default)]
pub struct BlkidInfo {
    pub uuid: String,
    pub part_uuid: String,
    pub label: String,
    pub fs_type: String,
}

pub fn read_blkid(device: &str) -> BlkidInfo {
    let Ok(output) = run_command("blkid", &["-o", "export", device], Duration::from_secs(2)) else {
        return BlkidInfo::default();
    };
    if output.status != Some(0) {
        return BlkidInfo::default();
    }

    let values: HashMap<String, String> = output
        .stdout
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            Some((key.to_string(), value.to_string()))
        })
        .collect();

    BlkidInfo {
        uuid: values.get("UUID").cloned().unwrap_or_default(),
        part_uuid: values.get("PARTUUID").cloned().unwrap_or_default(),
        label: values.get("LABEL").cloned().unwrap_or_default(),
        fs_type: values.get("TYPE").cloned().unwrap_or_default(),
    }
}
