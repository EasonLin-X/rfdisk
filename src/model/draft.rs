use crate::model::{enums::PartitionTableType, partition::PartitionInfo};

#[derive(Clone, Debug)]
pub(crate) struct DraftPartition {
    pub(crate) display_name: String,
    #[allow(dead_code)]
    pub(crate) dev_path: Option<String>,
    pub(crate) start_sector: u64,
    pub(crate) size_bytes: u64,
    pub(crate) partition_type: String,
    pub(crate) partition_type_raw: String,
    pub(crate) fs_type: String,
    #[allow(dead_code)]
    pub(crate) uuid: String,
    #[allow(dead_code)]
    pub(crate) part_uuid: String,
    #[allow(dead_code)]
    pub(crate) fs_label: String,
    pub(crate) part_name: String,
    pub(crate) mount_points: Vec<String>,
    pub(crate) is_swap: bool,
    pub(crate) pending: PendingState,
    pub(crate) original_start_sector: Option<u64>,
    pub(crate) original_size_bytes: Option<u64>,
    pub(crate) original_partition_type_raw: Option<String>,
    pub(crate) original_part_name: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PendingState {
    Existing,
    New,
    Deleted,
    #[allow(dead_code)]
    Modified,
}

#[derive(Clone, Debug)]
pub(crate) struct DraftConfig {
    pub(crate) original_table_type: PartitionTableType,
    pub(crate) table_type: PartitionTableType,
    pub(crate) partitions: Vec<DraftPartition>,
}

impl DraftConfig {
    pub(crate) fn from_partitions(
        partitions: Vec<PartitionInfo>,
        table_type: PartitionTableType,
    ) -> Self {
        let partitions = partitions
            .into_iter()
            .map(|partition| DraftPartition {
                display_name: partition.dev_path.clone(),
                dev_path: Some(partition.dev_path),
                start_sector: partition.start_sector,
                size_bytes: partition.size_bytes,
                partition_type: partition.partition_type,
                partition_type_raw: partition.partition_type_raw.clone(),
                fs_type: partition.fs_type,
                uuid: partition.uuid,
                part_uuid: partition.part_uuid,
                fs_label: partition.label,
                part_name: partition.part_name.clone(),
                mount_points: partition.mount_points,
                is_swap: partition.is_swap,
                pending: PendingState::Existing,
                original_start_sector: Some(partition.start_sector),
                original_size_bytes: Some(partition.size_bytes),
                original_partition_type_raw: Some(partition.partition_type_raw),
                original_part_name: Some(partition.part_name),
            })
            .collect();

        Self {
            original_table_type: table_type,
            table_type,
            partitions,
        }
    }

    pub(crate) fn has_changes(&self) -> bool {
        self.table_type != self.original_table_type
            || self
                .partitions
                .iter()
                .any(|part| part.pending != PendingState::Existing || part.key_fields_changed())
    }

    pub(crate) fn visible_partitions(&self) -> impl Iterator<Item = &DraftPartition> {
        self.partitions
            .iter()
            .filter(|part| part.pending != PendingState::Deleted)
    }
}

impl DraftPartition {
    pub(crate) fn key_fields_changed(&self) -> bool {
        self.original_start_sector != Some(self.start_sector)
            || self.original_size_bytes != Some(self.size_bytes)
            || self.original_partition_type_raw.as_deref() != Some(self.partition_type_raw.as_str())
            || self.original_part_name.as_deref() != Some(self.part_name.as_str())
    }
}
