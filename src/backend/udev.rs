use std::{collections::HashMap, time::Duration};

use crate::backend::cmd::run_command;

#[derive(Clone, Debug, Default)]
pub struct UdevInfo {
    pub values: HashMap<String, String>,
}

impl UdevInfo {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
}

pub fn read_udev_info(device: &str) -> UdevInfo {
    let Ok(output) = run_command(
        "udevadm",
        &["info", "--query=property", "--name", device],
        Duration::from_secs(2),
    ) else {
        return UdevInfo::default();
    };
    if output.status != Some(0) {
        return UdevInfo::default();
    }

    let values = output
        .stdout
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            Some((key.to_string(), value.to_string()))
        })
        .collect();

    UdevInfo { values }
}
