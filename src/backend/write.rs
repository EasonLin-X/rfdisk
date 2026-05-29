use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

use crate::{
    model::{DiskDevice, DraftConfig, DraftPartition, PartitionTableType},
    util::sector::bytes_to_sectors,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WriteDiskResult {
    WrittenAndReloaded,
    WrittenButReloadFailed { message: String },
    WriteFailed { message: String },
}

impl WriteDiskResult {
    #[cfg(test)]
    fn clears_draft(&self) -> bool {
        matches!(self, Self::WrittenAndReloaded)
    }

    #[cfg(test)]
    fn status_prefix(&self) -> &'static str {
        match self {
            Self::WrittenAndReloaded => "written and reloaded",
            Self::WrittenButReloadFailed { .. } => "written but not reloaded",
            Self::WriteFailed { .. } => "write failed",
        }
    }
}

pub(crate) fn write_partition_table(
    disk: &DiskDevice,
    draft: &DraftConfig,
    log: &mut fs::File,
) -> WriteDiskResult {
    let script = build_sfdisk_script(draft);
    writeln!(log, "sfdisk script:\n{script}").ok();

    let mut child = match Command::new("sfdisk")
        .arg(&disk.dev_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| WriteDiskResult::WriteFailed {
            message: format!("write failed: failed to start sfdisk: {err}"),
        }) {
        Ok(child) => child,
        Err(result) => return result,
    };

    let stdin = child
        .stdin
        .as_mut()
        .ok_or_else(|| WriteDiskResult::WriteFailed {
            message: "write failed: failed to open sfdisk stdin".to_string(),
        })
        .and_then(|stdin| {
            stdin
                .write_all(script.as_bytes())
                .map_err(|err| WriteDiskResult::WriteFailed {
                    message: format!("write failed: failed to write sfdisk script: {err}"),
                })
        });
    if let Err(result) = stdin {
        return result;
    }

    let output = match child
        .wait_with_output()
        .map_err(|err| WriteDiskResult::WriteFailed {
            message: format!("write failed: failed to wait for sfdisk: {err}"),
        }) {
        Ok(output) => output,
        Err(result) => return result,
    };

    log_command_output(log, "sfdisk", &output);
    if !output.status.success() {
        return WriteDiskResult::WriteFailed {
            message: format!("write failed: sfdisk exited with {}", output.status),
        };
    }

    let partprobe = Command::new("partprobe").arg(&disk.dev_path).output();
    match partprobe {
        Ok(output) if output.status.success() => log_command_output(log, "partprobe", &output),
        Ok(output) => {
            log_command_output(log, "partprobe", &output);
            let reread = match Command::new("blockdev")
                .arg("--rereadpt")
                .arg(&disk.dev_path)
                .output()
                .map_err(|err| WriteDiskResult::WrittenButReloadFailed {
                    message: format!(
                        "partition table was written, but kernel did not reload it: partprobe failed and blockdev could not start: {err}"
                    ),
                }) {
                Ok(output) => output,
                Err(result) => return result,
            };
            log_command_output(log, "blockdev --rereadpt", &reread);
            if !reread.status.success() {
                return WriteDiskResult::WrittenButReloadFailed {
                    message: "partition table was written, but kernel did not reload it."
                        .to_string(),
                };
            }
        }
        Err(err) => {
            writeln!(log, "partprobe start failed: {err}").ok();
            let reread = match Command::new("blockdev")
                .arg("--rereadpt")
                .arg(&disk.dev_path)
                .output()
                .map_err(|err| WriteDiskResult::WrittenButReloadFailed {
                    message: format!(
                        "partition table was written, but kernel did not reload it: partprobe missing and blockdev could not start: {err}"
                    ),
                }) {
                Ok(output) => output,
                Err(result) => return result,
            };
            log_command_output(log, "blockdev --rereadpt", &reread);
            if !reread.status.success() {
                return WriteDiskResult::WrittenButReloadFailed {
                    message: "partition table was written, but kernel did not reload it."
                        .to_string(),
                };
            }
        }
    }

    match Command::new("udevadm").arg("settle").output() {
        Ok(output) => {
            log_command_output(log, "udevadm settle", &output);
            if !output.status.success() {
                return WriteDiskResult::WrittenButReloadFailed {
                    message: "partition table was written and reloaded, but udev settle failed."
                        .to_string(),
                };
            }
        }
        Err(err) => {
            writeln!(log, "udevadm settle start failed: {err}").ok();
        }
    }

    WriteDiskResult::WrittenAndReloaded
}

fn build_sfdisk_script(draft: &DraftConfig) -> String {
    let label = draft.table_type.sfdisk_label();
    let mut script = format!("label: {label}\nunit: sectors\n\n");

    let mut partitions: Vec<DraftPartition> = draft.visible_partitions().cloned().collect();
    partitions.sort_by_key(|partition| partition.start_sector);

    for part in partitions {
        let start = part.start_sector;
        let size = bytes_to_sectors(part.size_bytes);
        if size == 0 {
            continue;
        }

        let mut fields = vec![format!("start={start}"), format!("size={size}")];
        let partition_type = if part.partition_type_raw.is_empty() {
            draft.table_type.default_partition_type_raw()
        } else {
            part.partition_type_raw.as_str()
        };
        if !partition_type.is_empty() {
            fields.push(format!("type={partition_type}"));
        }
        if draft.table_type == PartitionTableType::Gpt && !part.part_name.is_empty() {
            fields.push(format!(
                "name=\"{}\"",
                escape_sfdisk_quoted(&part.part_name)
            ));
        }

        script.push_str(&fields.join(", "));
        script.push('\n');
    }

    script
}

fn escape_sfdisk_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn log_command_output(log: &mut fs::File, name: &str, output: &std::process::Output) {
    writeln!(log, "$ {name}").ok();
    writeln!(log, "status: {}", output.status).ok();
    writeln!(log, "stdout:\n{}", String::from_utf8_lossy(&output.stdout)).ok();
    writeln!(log, "stderr:\n{}", String::from_utf8_lossy(&output.stderr)).ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PendingState;

    fn draft_partition() -> DraftPartition {
        DraftPartition {
            display_name: "/dev/sdb1".to_string(),
            dev_path: Some("/dev/sdb1".to_string()),
            start_sector: 2048,
            size_bytes: 4096 * 512,
            partition_type: "Linux filesystem".to_string(),
            partition_type_raw: "0FC63DAF-8483-4772-8E79-3D69D8477DE4".to_string(),
            fs_type: "ext4".to_string(),
            uuid: "fs-uuid".to_string(),
            part_uuid: "part-uuid".to_string(),
            fs_label: "rootfs".to_string(),
            part_name: "root".to_string(),
            mount_points: vec!["/".to_string()],
            is_swap: false,
            pending: PendingState::Existing,
            original_start_sector: Some(2048),
            original_size_bytes: Some(4096 * 512),
            original_partition_type_raw: Some("0FC63DAF-8483-4772-8E79-3D69D8477DE4".to_string()),
            original_part_name: Some("root".to_string()),
        }
    }

    #[test]
    fn sfdisk_script_preserves_existing_gpt_type_and_name() {
        let draft = DraftConfig {
            original_table_type: PartitionTableType::Gpt,
            table_type: PartitionTableType::Gpt,
            partitions: vec![draft_partition()],
        };

        let script = build_sfdisk_script(&draft);
        assert!(script.contains("label: gpt"));
        assert!(script.contains("type=0FC63DAF-8483-4772-8E79-3D69D8477DE4"));
        assert!(script.contains("name=\"root\""));
        assert!(!script.contains("uuid=part-uuid"));
    }

    #[test]
    fn sfdisk_script_uses_mbr_linux_default_when_raw_type_missing() {
        let mut partition = draft_partition();
        partition.partition_type_raw.clear();
        partition.part_name = "ignored-on-mbr".to_string();
        let draft = DraftConfig {
            original_table_type: PartitionTableType::Mbr,
            table_type: PartitionTableType::Mbr,
            partitions: vec![partition],
        };

        let script = build_sfdisk_script(&draft);
        assert!(script.contains("label: dos"));
        assert!(script.contains("type=83"));
        assert!(!script.contains("name="));
    }

    #[test]
    fn sfdisk_script_uses_modified_gpt_partition_type_raw() {
        let mut partition = draft_partition();
        partition.partition_type = "Linux swap".to_string();
        partition.partition_type_raw = "0657FD6D-A4AB-43C4-84E5-0933C84B4F4F".to_string();
        let draft = DraftConfig {
            original_table_type: PartitionTableType::Gpt,
            table_type: PartitionTableType::Gpt,
            partitions: vec![partition],
        };

        let script = build_sfdisk_script(&draft);

        assert!(script.contains("type=0657FD6D-A4AB-43C4-84E5-0933C84B4F4F"));
        assert!(!script.contains("uuid=part-uuid"));
    }

    #[test]
    fn sfdisk_script_uses_modified_mbr_partition_type_raw() {
        let mut partition = draft_partition();
        partition.partition_type = "Linux LVM".to_string();
        partition.partition_type_raw = "8E".to_string();
        partition.part_name.clear();
        let draft = DraftConfig {
            original_table_type: PartitionTableType::Mbr,
            table_type: PartitionTableType::Mbr,
            partitions: vec![partition],
        };

        let script = build_sfdisk_script(&draft);

        assert!(script.contains("label: dos"));
        assert!(script.contains("type=8E"));
        assert!(!script.contains("uuid=part-uuid"));
    }

    #[test]
    fn written_and_reloaded_clears_draft() {
        let result = WriteDiskResult::WrittenAndReloaded;

        assert!(result.clears_draft());
        assert_eq!(result.status_prefix(), "written and reloaded");
    }

    #[test]
    fn written_but_reload_failed_keeps_draft() {
        let result = WriteDiskResult::WrittenButReloadFailed {
            message: "partition table was written, but kernel did not reload it.".to_string(),
        };

        assert!(!result.clears_draft());
        assert_eq!(result.status_prefix(), "written but not reloaded");
    }
}
