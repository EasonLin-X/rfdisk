use serde::Deserialize;
use std::time::Duration;

use crate::{
    backend::cmd::run_command,
    model::{DetectedTableLabel, PartitionInfo},
};

#[derive(Clone, Debug)]
pub(crate) struct DiskLayout {
    pub(crate) label: DetectedTableLabel,
    pub(crate) partitions: Vec<PartitionInfo>,
}

#[derive(Debug, Deserialize)]
struct SfdiskRoot {
    partitiontable: Option<SfdiskPartitionTable>,
}

#[derive(Debug, Deserialize)]
struct SfdiskPartitionTable {
    label: Option<String>,
    partitions: Option<Vec<SfdiskPartition>>,
}

#[derive(Debug, Deserialize)]
struct SfdiskPartition {
    node: Option<String>,
    start: Option<u64>,
    size: Option<u64>,
    #[serde(rename = "type")]
    part_type: Option<String>,
    uuid: Option<String>,
    name: Option<String>,
    attrs: Option<String>,
}

pub(crate) fn read_layout(device: &str) -> Result<DiskLayout, String> {
    let output = run_command("sfdisk", &["--json", device], Duration::from_secs(3))?;
    if output.status != Some(0) {
        return Err(format!(
            "sfdisk --json failed with {:?}: {}",
            output.status,
            output.stderr.trim()
        ));
    }

    parse_sfdisk_json(&output.stdout)
}

pub(crate) fn parse_sfdisk_json(json: &str) -> Result<DiskLayout, String> {
    let root: SfdiskRoot =
        serde_json::from_str(json).map_err(|err| format!("invalid sfdisk json: {err}"))?;
    let Some(table) = root.partitiontable else {
        return Ok(DiskLayout {
            label: DetectedTableLabel::from_sfdisk_label(None),
            partitions: Vec::new(),
        });
    };

    let label = DetectedTableLabel::from_sfdisk_label(table.label.as_deref());
    let partitions = table
        .partitions
        .unwrap_or_default()
        .into_iter()
        .filter_map(|partition| {
            let dev_path = partition.node?;
            let start_sector = partition.start.unwrap_or(0);
            let size_bytes = partition.size.unwrap_or(0).saturating_mul(512);
            let partition_type_raw = partition.part_type.unwrap_or_default();
            let partition_type = (!partition_type_raw.is_empty())
                .then(|| display_partition_type(&partition_type_raw).to_string())
                .unwrap_or_else(|| "partition".to_string());
            let part_uuid = partition.uuid.unwrap_or_default();
            let part_name = partition.name.unwrap_or_default();
            let attrs = partition.attrs.unwrap_or_default();

            Some(PartitionInfo {
                dev_path,
                start_sector,
                size_bytes,
                fs_type: String::new(),
                partition_type: if attrs.is_empty() {
                    partition_type
                } else {
                    format!("{partition_type} {attrs}")
                },
                partition_type_raw,
                uuid: String::new(),
                part_uuid,
                label: String::new(),
                part_name,
                mount_points: Vec::new(),
                is_swap: false,
            })
        })
        .collect();

    Ok(DiskLayout { label, partitions })
}

fn display_partition_type(partition_type: &str) -> &str {
    let normalized = partition_type
        .trim()
        .trim_matches('{')
        .trim_matches('}')
        .to_ascii_uppercase();

    match normalized.as_str() {
        // GPT partition type GUIDs.
        "0FC63DAF-8483-4772-8E79-3D69D8477DE4" => "Linux filesystem",
        "0657FD6D-A4AB-43C4-84E5-0933C84B4F4F" => "Linux swap",
        "E6D6D379-F507-44C2-A23C-238F2A3DF928" => "Linux LVM",
        "A19D880F-05FC-4D3B-A006-743F0F84911E" => "Linux RAID",
        "21686148-6449-6E6F-744E-656564454649" => "BIOS boot",
        "C12A7328-F81F-11D2-BA4B-00A0C93EC93B" => "EFI System",
        "EBD0A0A2-B9E5-4433-87C0-68B6B72699C7" => "Microsoft basic data",
        "DE94BBA4-06D1-4D40-A16A-BFD50179D6AC" => "Windows recovery",

        // MBR partition type codes as reported by sfdisk.
        "83" => "Linux filesystem",
        "82" => "Linux swap",
        "8E" => "Linux LVM",
        "FD" => "Linux RAID",
        "EF" => "EFI System",
        "07" => "Microsoft basic data",
        _ => partition_type,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gpt_layout() {
        let json = r#"{
          "partitiontable": {
            "label": "gpt",
            "partitions": [
              {"node": "/dev/sdb1", "start": 2048, "size": 4096, "type": "linux", "uuid": "partuuid", "name": "root"}
            ]
          }
        }"#;

        let layout = parse_sfdisk_json(json).unwrap();
        assert_eq!(layout.label, DetectedTableLabel::Gpt);
        assert_eq!(layout.partitions.len(), 1);
        assert_eq!(layout.partitions[0].dev_path, "/dev/sdb1");
        assert_eq!(layout.partitions[0].size_bytes, 4096 * 512);
        assert_eq!(layout.partitions[0].partition_type, "linux");
        assert_eq!(layout.partitions[0].partition_type_raw, "linux");
        assert_eq!(layout.partitions[0].part_uuid, "partuuid");
        assert_eq!(layout.partitions[0].part_name, "root");
    }

    #[test]
    fn maps_common_gpt_partition_type_guid() {
        let json = r#"{
          "partitiontable": {
            "label": "gpt",
            "partitions": [
              {"node": "/dev/sdb1", "start": 2048, "size": 4096, "type": "0FC63DAF-8483-4772-8E79-3D69D8477DE4"}
            ]
          }
        }"#;

        let layout = parse_sfdisk_json(json).unwrap();
        assert_eq!(layout.partitions[0].partition_type, "Linux filesystem");
        assert_eq!(
            layout.partitions[0].partition_type_raw,
            "0FC63DAF-8483-4772-8E79-3D69D8477DE4"
        );
    }

    #[test]
    fn maps_common_mbr_partition_type_code() {
        let json = r#"{
          "partitiontable": {
            "label": "dos",
            "partitions": [
              {"node": "/dev/sdb1", "start": 2048, "size": 4096, "type": "83"}
            ]
          }
        }"#;

        let layout = parse_sfdisk_json(json).unwrap();
        assert_eq!(layout.partitions[0].partition_type, "Linux filesystem");
    }

    #[test]
    fn parses_empty_disk() {
        let layout = parse_sfdisk_json(r#"{"partitiontable":{"label":"dos"}}"#).unwrap();
        assert_eq!(layout.label, DetectedTableLabel::Mbr);
        assert!(layout.partitions.is_empty());
    }

    #[test]
    fn parses_unsupported_table_label_without_editable_type() {
        let layout = parse_sfdisk_json(r#"{"partitiontable":{"label":"sgi"}}"#).unwrap();

        assert_eq!(layout.label, DetectedTableLabel::Sgi);
        assert_eq!(layout.label.editable_table_type(), None);
    }
}
