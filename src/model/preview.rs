// Pure write-preview analysis for partition-table drafts.
// This module does not execute commands or read disks; it only classifies
// pending draft changes and assigns conservative risk levels.

use std::collections::HashMap;

use crate::{
    model::{DiskDevice, DraftConfig, DraftPartition, PendingState},
    safety::busy::{BusyImpact, DiskBusyReport},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WritePreview {
    pub(crate) disks: Vec<DiskWritePreview>,
    pub(crate) risk: RiskLevel,
}

impl WritePreview {
    pub(crate) fn from_drafts(disks: &[DiskDevice], drafts: &HashMap<String, DraftConfig>) -> Self {
        let mut previews = Vec::new();

        for (disk_id, draft) in drafts {
            if !draft.has_changes() {
                continue;
            }

            if let Some(disk) = disks.iter().find(|disk| disk.stable_id() == *disk_id) {
                previews.push(DiskWritePreview::from_disk(disk, draft));
            } else {
                previews.push(DiskWritePreview::missing_disk(disk_id));
            }
        }

        previews.sort_by(|a, b| a.disk_path.cmp(&b.disk_path));
        let risk = previews
            .iter()
            .fold(RiskLevel::Low, |risk, disk| risk.max(disk.risk));

        Self {
            disks: previews,
            risk,
        }
    }

    pub(crate) fn is_blocked(&self) -> bool {
        self.risk == RiskLevel::Blocked
    }

    pub(crate) fn status_summary(&self) -> String {
        if self.disks.is_empty() {
            return "No committed drafts to write.".to_string();
        }

        if let Some(blocked) = self
            .disks
            .iter()
            .find(|disk| disk.risk == RiskLevel::Blocked)
        {
            return blocked.status_summary();
        }

        if let Some(high) = self.disks.iter().find(|disk| disk.risk == RiskLevel::High) {
            return high.status_summary();
        }

        if self.disks.len() == 1 {
            return self.disks[0].status_summary();
        }

        let counts = self.total_counts();
        format!(
            "Write {} disk drafts: keep {}, create {}, delete {}, high {}. Type yes to write.",
            self.disks.len(),
            counts.keep,
            counts.create,
            counts.delete,
            counts.high
        )
    }

    fn total_counts(&self) -> ChangeCounts {
        self.disks
            .iter()
            .fold(ChangeCounts::default(), |mut counts, disk| {
                counts.add(disk.counts());
                counts
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DiskWritePreview {
    pub(crate) disk_path: String,
    pub(crate) risk: RiskLevel,
    pub(crate) changes: Vec<PartitionChange>,
    pub(crate) blocked_reason: Option<String>,
}

impl DiskWritePreview {
    pub(crate) fn from_disk(disk: &DiskDevice, draft: &DraftConfig) -> Self {
        let mut changes = Vec::new();

        if draft.table_type != draft.original_table_type {
            changes.push(PartitionChange {
                target: disk.dev_path.clone(),
                kind: ChangeKind::TableTypeChange,
                risk: RiskLevel::High,
                detail: format!(
                    "table type {} -> {}",
                    draft.original_table_type.as_str(),
                    draft.table_type.as_str()
                ),
            });
        }

        let has_existing = draft
            .partitions
            .iter()
            .any(|partition| partition.pending != PendingState::New);

        for partition in &draft.partitions {
            let change = match partition.pending {
                PendingState::Existing => {
                    if partition.key_fields_changed() {
                        recreate_change(partition)
                    } else {
                        keep_change(partition)
                    }
                }
                PendingState::New => create_change(partition, has_existing),
                PendingState::Deleted => delete_change(partition),
                PendingState::Modified => recreate_change(partition),
            };
            changes.push(change);
        }

        let risk = changes
            .iter()
            .fold(RiskLevel::Low, |risk, change| risk.max(change.risk));

        let mut preview = Self {
            disk_path: disk.dev_path.clone(),
            risk,
            changes,
            blocked_reason: None,
        };
        let busy_impact = DiskBusyReport::from_disk_and_partitions(disk, &draft.partitions)
            .impact_for_changes(&preview.changes);
        preview.apply_busy_impact(busy_impact);
        preview
    }

    pub(crate) fn missing_disk(disk_id: &str) -> Self {
        Self {
            disk_path: disk_id.to_string(),
            risk: RiskLevel::Blocked,
            changes: Vec::new(),
            blocked_reason: Some(format!("Blocked: {disk_id} disappeared before write.")),
        }
    }

    pub(crate) fn status_summary(&self) -> String {
        if let Some(reason) = &self.blocked_reason {
            return reason.clone();
        }

        if self.risk == RiskLevel::High {
            if let Some(change) = self
                .changes
                .iter()
                .find(|change| change.risk == RiskLevel::High)
            {
                return format!(
                    "High risk: {} {}. Type yes to write.",
                    self.disk_path, change.detail
                );
            }
        }

        let counts = self.counts();
        format!(
            "Write {}: keep {}, create {}, delete {}, high {}. Type yes to write.",
            self.disk_path, counts.keep, counts.create, counts.delete, counts.high
        )
    }

    pub(crate) fn log_summary(&self) -> String {
        let counts = self.counts();
        let blocked = self.blocked_reason.as_deref().unwrap_or("-");
        format!(
            "preview: risk={:?}, keep={}, create={}, delete={}, recreate={}, table_type_change={}, blocked={}",
            self.risk,
            counts.keep,
            counts.create,
            counts.delete,
            counts.recreate,
            counts.table_type_change,
            blocked
        )
    }

    fn counts(&self) -> ChangeCounts {
        let mut counts = ChangeCounts::default();
        for change in &self.changes {
            match change.kind {
                ChangeKind::Keep => counts.keep += 1,
                ChangeKind::Create => counts.create += 1,
                ChangeKind::Delete => counts.delete += 1,
                ChangeKind::Recreate => counts.recreate += 1,
                ChangeKind::TableTypeChange => counts.table_type_change += 1,
            }

            if change.risk == RiskLevel::High {
                counts.high += 1;
            }
        }
        counts
    }

    fn apply_busy_impact(&mut self, impact: BusyImpact) {
        if let Some(reason) = impact.blocked_reason {
            self.risk = RiskLevel::Blocked;
            self.blocked_reason = Some(reason);
            return;
        }

        self.risk = self.risk.max(impact.risk);
        for caution in impact.cautions {
            if let Some(change) = self
                .changes
                .iter_mut()
                .find(|change| change.target == caution.target)
            {
                change.risk = change.risk.max(RiskLevel::Caution);
                change.detail = format!("{}; {}", change.detail, caution.detail_suffix);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PartitionChange {
    pub(crate) target: String,
    pub(crate) kind: ChangeKind,
    pub(crate) risk: RiskLevel,
    pub(crate) detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChangeKind {
    Keep,
    Create,
    Delete,
    Recreate,
    TableTypeChange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RiskLevel {
    Low,
    Caution,
    High,
    Blocked,
}

impl RiskLevel {
    pub(crate) fn priority(&self) -> u8 {
        match self {
            Self::Low => 0,
            Self::Caution => 1,
            Self::High => 2,
            Self::Blocked => 3,
        }
    }

    pub(crate) fn max(self, other: Self) -> Self {
        if other.priority() > self.priority() {
            other
        } else {
            self
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ChangeCounts {
    keep: usize,
    create: usize,
    delete: usize,
    recreate: usize,
    table_type_change: usize,
    high: usize,
}

impl ChangeCounts {
    fn add(&mut self, other: Self) {
        self.keep += other.keep;
        self.create += other.create;
        self.delete += other.delete;
        self.recreate += other.recreate;
        self.table_type_change += other.table_type_change;
        self.high += other.high;
    }
}

fn keep_change(partition: &DraftPartition) -> PartitionChange {
    PartitionChange {
        target: partition.display_name.clone(),
        kind: ChangeKind::Keep,
        risk: RiskLevel::Low,
        detail: format!("keep {}", partition.display_name),
    }
}

fn create_change(partition: &DraftPartition, has_existing: bool) -> PartitionChange {
    PartitionChange {
        target: partition.display_name.clone(),
        kind: ChangeKind::Create,
        risk: if has_existing {
            RiskLevel::Caution
        } else {
            RiskLevel::Low
        },
        detail: format!("create {}", partition.display_name),
    }
}

fn delete_change(partition: &DraftPartition) -> PartitionChange {
    PartitionChange {
        target: partition.display_name.clone(),
        kind: ChangeKind::Delete,
        risk: RiskLevel::High,
        detail: format!("delete {}", partition.display_name),
    }
}

fn recreate_change(partition: &DraftPartition) -> PartitionChange {
    PartitionChange {
        target: partition.display_name.clone(),
        kind: ChangeKind::Recreate,
        risk: RiskLevel::High,
        detail: format!("recreate {}", partition.display_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DetectedTableLabel, DiskGuard, DiskKind, PartitionTableType, ScanStatus};

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

    fn existing_partition() -> DraftPartition {
        DraftPartition {
            display_name: "/dev/sdb1".to_string(),
            dev_path: Some("/dev/sdb1".to_string()),
            start_sector: 2048,
            size_bytes: 1024 * 1024,
            partition_type: "Linux filesystem".to_string(),
            partition_type_raw: "0FC63DAF-8483-4772-8E79-3D69D8477DE4".to_string(),
            fs_type: "ext4".to_string(),
            uuid: String::new(),
            part_uuid: String::new(),
            fs_label: String::new(),
            part_name: "root".to_string(),
            mount_points: Vec::new(),
            is_swap: false,
            pending: PendingState::Existing,
            original_start_sector: Some(2048),
            original_size_bytes: Some(1024 * 1024),
            original_partition_type_raw: Some("0FC63DAF-8483-4772-8E79-3D69D8477DE4".to_string()),
            original_part_name: Some("root".to_string()),
        }
    }

    fn new_partition() -> DraftPartition {
        DraftPartition {
            display_name: "[new partition 1]".to_string(),
            dev_path: None,
            start_sector: 2048,
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

    #[test]
    fn empty_disk_create_is_low_risk() {
        let draft = DraftConfig {
            original_table_type: PartitionTableType::Gpt,
            table_type: PartitionTableType::Gpt,
            partitions: vec![new_partition()],
        };

        let preview = DiskWritePreview::from_disk(&disk(DiskGuard::None), &draft);

        assert_eq!(preview.risk, RiskLevel::Low);
        assert_eq!(preview.changes[0].kind, ChangeKind::Create);
    }

    #[test]
    fn existing_partition_without_change_is_keep() {
        let draft = DraftConfig {
            original_table_type: PartitionTableType::Gpt,
            table_type: PartitionTableType::Gpt,
            partitions: vec![existing_partition()],
        };

        let preview = DiskWritePreview::from_disk(&disk(DiskGuard::None), &draft);

        assert_eq!(preview.risk, RiskLevel::Low);
        assert_eq!(preview.changes[0].kind, ChangeKind::Keep);
    }

    #[test]
    fn deleted_existing_partition_is_high_risk_delete() {
        let mut partition = existing_partition();
        partition.pending = PendingState::Deleted;
        let draft = DraftConfig {
            original_table_type: PartitionTableType::Gpt,
            table_type: PartitionTableType::Gpt,
            partitions: vec![partition],
        };

        let preview = DiskWritePreview::from_disk(&disk(DiskGuard::None), &draft);

        assert_eq!(preview.risk, RiskLevel::High);
        assert_eq!(preview.changes[0].kind, ChangeKind::Delete);
    }

    #[test]
    fn table_type_change_is_high_risk() {
        let draft = DraftConfig {
            original_table_type: PartitionTableType::Gpt,
            table_type: PartitionTableType::Mbr,
            partitions: vec![existing_partition()],
        };

        let preview = DiskWritePreview::from_disk(&disk(DiskGuard::None), &draft);

        assert_eq!(preview.risk, RiskLevel::High);
        assert!(preview
            .changes
            .iter()
            .any(|change| change.kind == ChangeKind::TableTypeChange));
    }

    #[test]
    fn changed_existing_key_fields_are_recreate() {
        let mut partition = existing_partition();
        partition.size_bytes += 512;
        let draft = DraftConfig {
            original_table_type: PartitionTableType::Gpt,
            table_type: PartitionTableType::Gpt,
            partitions: vec![partition],
        };

        let preview = DiskWritePreview::from_disk(&disk(DiskGuard::None), &draft);

        assert_eq!(preview.risk, RiskLevel::High);
        assert_eq!(preview.changes[0].kind, ChangeKind::Recreate);
    }

    #[test]
    fn protected_disk_with_changes_is_blocked() {
        let draft = DraftConfig {
            original_table_type: PartitionTableType::Gpt,
            table_type: PartitionTableType::Gpt,
            partitions: vec![new_partition()],
        };

        let preview = DiskWritePreview::from_disk(&disk(DiskGuard::System), &draft);

        assert_eq!(preview.risk, RiskLevel::Blocked);
        assert!(preview.blocked_reason.is_some());
    }
}
