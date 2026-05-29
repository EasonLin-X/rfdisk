// First-pass busy/safety analysis for mounted, swap, and protected disks.
// The module reports whether busy partitions are kept or touched; preview
// owns the final risk merge and user-facing write decision.

use crate::model::{ChangeKind, DiskDevice, DiskGuard, DraftPartition, PartitionChange, RiskLevel};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BusyReason {
    ProtectedSystemDisk,
    Mounted(Vec<String>),
    Swap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BusyReport {
    pub(crate) dev_path: String,
    pub(crate) reasons: Vec<BusyReason>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DiskBusyReport {
    pub(crate) disk_dev_path: String,
    pub(crate) partition_reports: Vec<BusyReport>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BusyImpact {
    pub(crate) risk: RiskLevel,
    pub(crate) blocked_reason: Option<String>,
    pub(crate) cautions: Vec<BusyCaution>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BusyCaution {
    pub(crate) target: String,
    pub(crate) detail_suffix: String,
}

impl DiskBusyReport {
    pub(crate) fn from_disk_and_partitions(
        disk: &DiskDevice,
        partitions: &[DraftPartition],
    ) -> Self {
        let mut partition_reports = Vec::new();

        if matches!(disk.guard, DiskGuard::System | DiskGuard::Used) {
            partition_reports.push(BusyReport {
                dev_path: disk.dev_path.clone(),
                reasons: vec![BusyReason::ProtectedSystemDisk],
            });
        }

        for partition in partitions {
            let mut reasons = Vec::new();
            if !partition.mount_points.is_empty() {
                reasons.push(BusyReason::Mounted(partition.mount_points.clone()));
            }
            if partition.is_swap {
                reasons.push(BusyReason::Swap);
            }

            if !reasons.is_empty() {
                partition_reports.push(BusyReport {
                    dev_path: partition.display_name.clone(),
                    reasons,
                });
            }
        }

        Self {
            disk_dev_path: disk.dev_path.clone(),
            partition_reports,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.partition_reports.is_empty()
    }

    pub(crate) fn impact_for_changes(&self, changes: &[PartitionChange]) -> BusyImpact {
        let mut impact = BusyImpact {
            risk: RiskLevel::Low,
            blocked_reason: None,
            cautions: Vec::new(),
        };

        if self.is_empty() {
            return impact;
        }

        if self.has_protected_system_disk() {
            impact.risk = RiskLevel::Blocked;
            impact.blocked_reason = Some(format!(
                "Blocked: {} is a protected system disk.",
                self.disk_dev_path
            ));
            return impact;
        }

        if self.table_type_change_touches_busy(changes) {
            impact.risk = RiskLevel::Blocked;
            impact.blocked_reason = Some(format!(
                "Blocked: {} table type change would touch mounted/swap partitions.",
                self.disk_dev_path
            ));
            return impact;
        }

        for report in &self.partition_reports {
            let Some(change) = changes
                .iter()
                .find(|change| change.target == report.dev_path)
            else {
                continue;
            };

            match change.kind {
                ChangeKind::Keep => {
                    impact.risk = impact.risk.max(RiskLevel::Caution);
                    impact.cautions.push(BusyCaution {
                        target: report.dev_path.clone(),
                        detail_suffix: format!(
                            "busy but kept ({})",
                            format_reasons(&report.reasons)
                        ),
                    });
                }
                ChangeKind::Delete | ChangeKind::Recreate => {
                    impact.risk = RiskLevel::Blocked;
                    impact.blocked_reason = Some(format!(
                        "Blocked: {} is {} and would be {}.",
                        report.dev_path,
                        format_reasons(&report.reasons),
                        change.kind.as_action()
                    ));
                    return impact;
                }
                ChangeKind::Create | ChangeKind::TableTypeChange => {}
            }
        }

        impact
    }

    fn has_protected_system_disk(&self) -> bool {
        self.partition_reports.iter().any(|report| {
            report
                .reasons
                .iter()
                .any(|reason| matches!(reason, BusyReason::ProtectedSystemDisk))
        })
    }

    fn table_type_change_touches_busy(&self, changes: &[PartitionChange]) -> bool {
        let has_busy_partition = self.partition_reports.iter().any(|report| {
            report
                .reasons
                .iter()
                .any(|reason| matches!(reason, BusyReason::Mounted(_) | BusyReason::Swap))
        });
        has_busy_partition
            && changes
                .iter()
                .any(|change| change.kind == ChangeKind::TableTypeChange)
    }
}

impl ChangeKind {
    fn as_action(&self) -> &'static str {
        match self {
            Self::Keep => "kept",
            Self::Create => "created",
            Self::Delete => "deleted",
            Self::Recreate => "recreated",
            Self::TableTypeChange => "table type changed",
        }
    }
}

fn format_reasons(reasons: &[BusyReason]) -> String {
    reasons
        .iter()
        .map(|reason| match reason {
            BusyReason::ProtectedSystemDisk => "protected system disk".to_string(),
            BusyReason::Mounted(points) => format!("mounted at {}", points.join(",")),
            BusyReason::Swap => "swap".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" and ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        DetectedTableLabel, DiskKind, DiskWritePreview, DraftConfig, PartitionTableType,
        PendingState, ScanStatus,
    };

    fn disk(guard: DiskGuard) -> DiskDevice {
        DiskDevice {
            name: "sdb".to_string(),
            dev_path: "/dev/sdb".to_string(),
            size_bytes: 20 * 1024 * 1024 * 1024,
            kind: DiskKind::Hdd,
            serial: "test-disk".to_string(),
            guard,
            model: "test".to_string(),
            table_label: DetectedTableLabel::Gpt,
            scan_status: ScanStatus::sfdisk(),
        }
    }

    fn existing_partition(name: &str) -> DraftPartition {
        DraftPartition {
            display_name: name.to_string(),
            dev_path: Some(name.to_string()),
            start_sector: 2048,
            size_bytes: 1024 * 1024,
            partition_type: "Linux filesystem".to_string(),
            partition_type_raw: "0FC63DAF-8483-4772-8E79-3D69D8477DE4".to_string(),
            fs_type: "ext4".to_string(),
            uuid: String::new(),
            part_uuid: String::new(),
            fs_label: String::new(),
            part_name: String::new(),
            mount_points: Vec::new(),
            is_swap: false,
            pending: PendingState::Existing,
            original_start_sector: Some(2048),
            original_size_bytes: Some(1024 * 1024),
            original_partition_type_raw: Some("0FC63DAF-8483-4772-8E79-3D69D8477DE4".to_string()),
            original_part_name: Some(String::new()),
        }
    }

    fn new_partition() -> DraftPartition {
        DraftPartition {
            display_name: "[new partition 2]".to_string(),
            dev_path: None,
            start_sector: 4096,
            size_bytes: 1024 * 1024,
            partition_type: "Linux filesystem".to_string(),
            partition_type_raw: "0FC63DAF-8483-4772-8E79-3D69D8477DE4".to_string(),
            fs_type: String::new(),
            uuid: String::new(),
            part_uuid: String::new(),
            fs_label: String::new(),
            part_name: String::new(),
            mount_points: Vec::new(),
            is_swap: false,
            pending: PendingState::New,
            original_start_sector: None,
            original_size_bytes: None,
            original_partition_type_raw: None,
            original_part_name: None,
        }
    }

    fn preview_for(partitions: Vec<DraftPartition>) -> DiskWritePreview {
        let draft = DraftConfig {
            original_table_type: PartitionTableType::Gpt,
            table_type: PartitionTableType::Gpt,
            partitions,
        };
        DiskWritePreview::from_disk(&disk(DiskGuard::None), &draft)
    }

    #[test]
    fn mounted_keep_plus_create_is_caution() {
        let mut mounted = existing_partition("/dev/sdb1");
        mounted.mount_points = vec!["/mnt".to_string()];
        let partitions = vec![mounted.clone(), new_partition()];
        let report =
            DiskBusyReport::from_disk_and_partitions(&disk(DiskGuard::Mounted), &partitions);
        let preview = preview_for(partitions);
        let impact = report.impact_for_changes(&preview.changes);

        assert_eq!(impact.risk, RiskLevel::Caution);
        assert!(impact.blocked_reason.is_none());
    }

    #[test]
    fn mounted_delete_is_blocked() {
        let mut mounted = existing_partition("/dev/sdb1");
        mounted.mount_points = vec!["/mnt".to_string()];
        mounted.pending = PendingState::Deleted;
        let partitions = vec![mounted.clone()];
        let report =
            DiskBusyReport::from_disk_and_partitions(&disk(DiskGuard::Mounted), &partitions);
        let preview = preview_for(partitions);
        let impact = report.impact_for_changes(&preview.changes);

        assert_eq!(impact.risk, RiskLevel::Blocked);
        assert!(impact.blocked_reason.is_some());
    }

    #[test]
    fn swap_keep_plus_create_is_caution() {
        let mut swap = existing_partition("/dev/sdb1");
        swap.is_swap = true;
        let partitions = vec![swap.clone(), new_partition()];
        let report = DiskBusyReport::from_disk_and_partitions(&disk(DiskGuard::Swap), &partitions);
        let preview = preview_for(partitions);
        let impact = report.impact_for_changes(&preview.changes);

        assert_eq!(impact.risk, RiskLevel::Caution);
        assert!(impact.blocked_reason.is_none());
    }

    #[test]
    fn swap_delete_is_blocked() {
        let mut swap = existing_partition("/dev/sdb1");
        swap.is_swap = true;
        swap.pending = PendingState::Deleted;
        let partitions = vec![swap.clone()];
        let report = DiskBusyReport::from_disk_and_partitions(&disk(DiskGuard::Swap), &partitions);
        let preview = preview_for(partitions);
        let impact = report.impact_for_changes(&preview.changes);

        assert_eq!(impact.risk, RiskLevel::Blocked);
    }

    #[test]
    fn protected_system_disk_is_blocked() {
        let partitions = vec![new_partition()];
        let report =
            DiskBusyReport::from_disk_and_partitions(&disk(DiskGuard::System), &partitions);
        let preview = preview_for(partitions);
        let impact = report.impact_for_changes(&preview.changes);

        assert_eq!(impact.risk, RiskLevel::Blocked);
    }

    #[test]
    fn table_type_change_with_mounted_partition_is_blocked() {
        let mut mounted = existing_partition("/dev/sdb1");
        mounted.mount_points = vec!["/mnt".to_string()];
        let partitions = vec![mounted.clone()];
        let draft = DraftConfig {
            original_table_type: PartitionTableType::Gpt,
            table_type: PartitionTableType::Mbr,
            partitions: partitions.clone(),
        };
        let report =
            DiskBusyReport::from_disk_and_partitions(&disk(DiskGuard::Mounted), &partitions);
        let preview = DiskWritePreview::from_disk(&disk(DiskGuard::None), &draft);
        let impact = report.impact_for_changes(&preview.changes);

        assert_eq!(impact.risk, RiskLevel::Blocked);
    }
}
