use crate::{
    algo,
    app::state::{App, PartitionRow},
    model::{DraftConfig, DraftPartition, FreeSpaceSegment, PartitionInfo, PendingState},
};

impl App {
    pub(crate) fn active_disk_id(&self) -> Option<String> {
        self.selected_disk()
            .map(crate::model::DiskDevice::stable_id)
    }

    pub(crate) fn active_draft(&self) -> Option<&DraftConfig> {
        let disk_id = self.active_disk_id()?;
        self.drafts.get(&disk_id)
    }

    pub(crate) fn cached_partitions_for(
        &self,
        disk: &crate::model::DiskDevice,
    ) -> Vec<PartitionInfo> {
        self.partition_cache
            .get(&disk.stable_id())
            .cloned()
            .unwrap_or_default()
    }

    pub(crate) fn free_space_segments(&self) -> Vec<FreeSpaceSegment> {
        let Some(disk) = self.selected_disk() else {
            return Vec::new();
        };
        algo::free_space::calculate_free_space(
            disk,
            self.active_table_type(),
            &self.active_partitions(),
            2048,
        )
    }

    pub(crate) fn partition_row_count(&self) -> usize {
        self.partition_rows().len()
    }

    pub(crate) fn selected_free_space(&self) -> Option<FreeSpaceSegment> {
        match self.partition_rows().get(self.current_partition_idx) {
            Some(PartitionRow::FreeSpace(segment)) => Some(*segment),
            _ => None,
        }
    }

    pub(crate) fn active_partitions(&self) -> Vec<DraftPartition> {
        if let Some(draft) = self.active_draft() {
            return draft.visible_partitions().cloned().collect();
        }

        let Some(disk) = self.selected_disk() else {
            return Vec::new();
        };

        DraftConfig::from_partitions(self.cached_partitions_for(disk), self.active_table_type())
            .partitions
    }

    pub(crate) fn partition_rows(&self) -> Vec<PartitionRow> {
        let Some(disk) = self.selected_disk() else {
            return Vec::new();
        };

        let partition_entries: Vec<(usize, DraftPartition)> = if let Some(draft) =
            self.active_draft()
        {
            draft
                .partitions
                .iter()
                .enumerate()
                .filter(|(_, partition)| partition.pending != PendingState::Deleted)
                .map(|(index, partition)| (index, partition.clone()))
                .collect()
        } else {
            DraftConfig::from_partitions(self.cached_partitions_for(disk), self.active_table_type())
                .partitions
                .into_iter()
                .enumerate()
                .collect()
        };

        let visible_partitions: Vec<DraftPartition> = partition_entries
            .iter()
            .map(|(_, partition)| partition.clone())
            .collect();
        let free_spaces = algo::free_space::calculate_free_space(
            disk,
            self.active_table_type(),
            &visible_partitions,
            2048,
        );

        let mut rows = Vec::with_capacity(partition_entries.len() + free_spaces.len());
        rows.extend(
            partition_entries
                .into_iter()
                .map(|(draft_index, partition)| PartitionRow::Partition {
                    draft_index,
                    partition,
                }),
        );
        rows.extend(free_spaces.into_iter().map(PartitionRow::FreeSpace));

        rows.sort_by_key(|row| match row {
            PartitionRow::Partition { partition, .. } => (partition.start_sector, 0_u8),
            PartitionRow::FreeSpace(segment) => (segment.start_sector, 1_u8),
        });
        rows
    }
}
