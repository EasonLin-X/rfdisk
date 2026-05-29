use crate::{
    app::state::{
        App, AppLayer, Focus, InputMode, PartitionRow, SizeAdjustment, TypePickerColumn,
        WriteConfirmKind,
    },
    model::{
        DraftPartition, PartitionTableType, PendingState, WritePreview, COMMON_PARTITION_TYPES,
    },
    util::{
        sector::bytes_to_sectors,
        size::{format_size, parse_size},
    },
};

const SIZE_AUTO_SHRINK_MAX_EXTRA_BYTES: u64 = 1024 * 1024;
const SIZE_AUTO_SHRINK_MAX_EXTRA_RATIO: f64 = 0.001;

pub(crate) fn move_index(index: usize, len: usize, direction: isize) -> usize {
    if len == 0 {
        return 0;
    }

    if direction < 0 {
        index.saturating_sub(direction.unsigned_abs()).min(len - 1)
    } else {
        index.saturating_add(direction as usize).min(len - 1)
    }
}

impl App {
    pub(crate) fn close_type_picker(&mut self) {
        self.input_mode = InputMode::Normal;
        self.layer = AppLayer::Edit;
        self.focus = Focus::Partitions;
        self.selected_menu = 2;
        self.status = "Type picker closed. Commit saves draft changes.".to_string();
    }

    pub(crate) fn enter_type_picker(&mut self) {
        if self.selected_disk().is_none() {
            self.status = "No disk selected.".to_string();
            return;
        }

        self.layer = AppLayer::Edit;
        self.focus = Focus::Partitions;
        self.selected_menu = 2;
        self.input_mode = InputMode::TypePicker;
        self.type_picker_column = TypePickerColumn::TableType;
        self.type_picker_table_idx = match self.active_table_type() {
            PartitionTableType::Gpt => 0,
            PartitionTableType::Mbr => 1,
        };
        self.type_picker_part_idx = self
            .type_picker_part_idx
            .min(COMMON_PARTITION_TYPES.len().saturating_sub(1));
        self.status = "Type picker: Part Type changes the selected partition draft; Table Type changes draft GPT/MBR.".to_string();
    }

    pub(crate) fn toggle_type_picker_column(&mut self) {
        self.type_picker_column = match self.type_picker_column {
            TypePickerColumn::PartType => TypePickerColumn::TableType,
            TypePickerColumn::TableType => TypePickerColumn::PartType,
        };
    }

    pub(crate) fn move_type_picker_selection(&mut self, direction: isize) {
        match self.type_picker_column {
            TypePickerColumn::PartType => {
                self.type_picker_part_idx = move_index(
                    self.type_picker_part_idx,
                    COMMON_PARTITION_TYPES.len(),
                    direction,
                );
            }
            TypePickerColumn::TableType => {
                self.type_picker_table_idx = move_index(
                    self.type_picker_table_idx,
                    PartitionTableType::ALL.len(),
                    direction,
                );
            }
        }
    }

    pub(crate) fn apply_type_picker_selection(&mut self) {
        match self.type_picker_column {
            TypePickerColumn::PartType => {
                self.apply_partition_type_from_picker();
            }
            TypePickerColumn::TableType => {
                let table_type = PartitionTableType::ALL[self.type_picker_table_idx];
                self.set_table_type_from_picker(table_type);
            }
        }
    }

    pub(crate) fn apply_partition_type_from_picker(&mut self) {
        let Some(disk) = self.selected_disk() else {
            self.status = "No disk selected.".to_string();
            return;
        };

        if disk.is_protected() {
            self.status = disk.guard_message().to_string();
            return;
        }

        let rows = self.partition_rows();
        let Some(row) = rows.get(self.current_partition_idx) else {
            self.status = "No partition selected.".to_string();
            return;
        };
        let target_index = match row {
            PartitionRow::Partition { draft_index, .. } => *draft_index,
            PartitionRow::FreeSpace(_) => {
                self.status = "Create a partition first, then set its type.".to_string();
                return;
            }
        };

        let Some(choice) = COMMON_PARTITION_TYPES
            .get(self.type_picker_part_idx)
            .copied()
        else {
            self.status = "Unknown partition type selection.".to_string();
            return;
        };
        let table_type = self.active_table_type();
        let Some(raw_type) = choice.raw_for(table_type) else {
            self.status = format!(
                "{} is not supported for {} in this version.",
                choice.name,
                table_type.as_str()
            );
            return;
        };

        let Some(draft) = self.ensure_draft() else {
            self.status = "No disk selected.".to_string();
            return;
        };
        let Some(partition) = draft.partitions.get_mut(target_index) else {
            self.status = "Selected partition disappeared from draft.".to_string();
            return;
        };

        partition.partition_type = choice.name.to_string();
        partition.partition_type_raw = raw_type.to_string();
        self.input_mode = InputMode::Normal;
        self.layer = AppLayer::Edit;
        self.focus = Focus::Partitions;
        self.selected_menu = 2;
        self.status = format!(
            "Part Type set to {} in draft. This does not format the filesystem; Commit and Write apply the partition-table type.",
            choice.name
        );
    }

    pub(crate) fn set_table_type_from_picker(&mut self, new_type: PartitionTableType) {
        let Some(disk) = self.selected_disk() else {
            self.status = "No disk selected.".to_string();
            return;
        };

        if disk.is_protected() {
            self.status = disk.guard_message().to_string();
            return;
        }

        let visible_partition_count = self.active_partitions().len();
        if let Some(draft) = self.ensure_draft() {
            draft.table_type = new_type;
        }
        self.layer = AppLayer::Edit;
        self.selected_menu = 2;
        self.input_mode = InputMode::Normal;
        self.status = if visible_partition_count > 0 {
            format!(
                "High risk: Type changed to {}. Write will recreate the partition table from the draft; existing filesystems are only likely to survive if start/size/type are preserved exactly.",
                new_type.as_str()
            )
        } else {
            format!(
                "Type set to {} for an empty disk. Low risk: this only chooses the new partition table type until Write.",
                new_type.as_str()
            )
        };
    }

    pub(crate) fn start_new_partition(&mut self) {
        let Some(disk) = self.selected_disk() else {
            self.status = "No disk selected.".to_string();
            return;
        };

        if disk.is_protected() {
            self.status = disk.guard_message().to_string();
            return;
        }

        if self.active_table_type() == PartitionTableType::Mbr
            && self.active_partitions().len() >= 4
        {
            self.status = "MBR supports up to 4 primary partitions in this version.".to_string();
            return;
        }

        let Some(free_space) = self.selected_free_space() else {
            if self.free_space_segments().is_empty() {
                self.status = "No aligned free space is available for a new partition.".to_string();
            } else {
                self.status =
                    "Select a [free space] row before creating a new partition.".to_string();
            }
            return;
        };

        let available_bytes = free_space.size_bytes();

        self.ensure_draft();
        self.layer = AppLayer::Edit;
        self.selected_menu = 0;
        self.input = format!("{available_bytes}B");
        self.input_is_default = true;
        self.pending_size_adjustment = None;
        self.pending_new_start_sector = Some(free_space.start_sector);
        self.pending_new_available_bytes = available_bytes;
        self.input_mode = InputMode::Size;
        self.status = format!(
            "Enter new partition size, or press Enter to use all remaining space ({}).",
            format_size(available_bytes)
        );
    }

    pub(crate) fn add_new_partition_from_input(&mut self) {
        if let Some(adjustment) = self.pending_size_adjustment {
            self.create_new_partition(adjustment.adjusted_bytes);
            return;
        }

        let Some(size_bytes) = parse_size(&self.input) else {
            self.status = "Invalid size. Use values like 512M, 20G, or 1T.".to_string();
            return;
        };

        if self.selected_disk().is_none() {
            self.status = "No disk selected.".to_string();
            return;
        }

        let available_bytes = self.pending_new_available_bytes;

        if size_bytes == 0 {
            self.status = "New partition size must be greater than 0.".to_string();
            return;
        }

        if size_bytes > available_bytes {
            let extra_bytes = size_bytes.saturating_sub(available_bytes);
            let extra_ratio = extra_bytes as f64 / available_bytes.max(1) as f64;
            if extra_bytes > SIZE_AUTO_SHRINK_MAX_EXTRA_BYTES
                && extra_ratio > SIZE_AUTO_SHRINK_MAX_EXTRA_RATIO
            {
                self.status = format!(
                    "Requested {} exceeds selected free space {} by {}. Choose a smaller size.",
                    format_size(size_bytes),
                    format_size(available_bytes),
                    format_size(extra_bytes)
                );
                return;
            }

            self.pending_size_adjustment = Some(SizeAdjustment {
                requested_bytes: size_bytes,
                adjusted_bytes: available_bytes,
            });
            self.input = format!("{available_bytes}B");
            self.input_is_default = false;
            self.status = format!(
                "Requested {} slightly exceeds free space. Adjusted to {} sectors ({}). Press Enter again to confirm.",
                format_size(size_bytes),
                bytes_to_sectors(available_bytes),
                format_size(available_bytes)
            );
            return;
        }

        self.create_new_partition(size_bytes);
    }

    fn create_new_partition(&mut self, size_bytes: u64) {
        let Some(start_sector) = self.pending_new_start_sector else {
            self.status = "No selected free space. Start New from a [free space] row.".to_string();
            return;
        };

        let table_type = self.active_table_type();
        {
            let Some(draft) = self.ensure_draft() else {
                self.status = "No disk selected.".to_string();
                return;
            };

            let index = draft.partitions.len() + 1;
            draft.partitions.push(DraftPartition {
                display_name: format!("[new partition {index}]"),
                dev_path: None,
                start_sector,
                size_bytes,
                partition_type: "Linux filesystem".to_string(),
                partition_type_raw: table_type.default_partition_type_raw().to_string(),
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
            });
        }

        let preferred_free = self.partition_rows().iter().position(|row| {
            matches!(row, PartitionRow::FreeSpace(segment) if segment.start_sector > start_sector)
        });
        let new_partition = self.partition_rows().iter().position(|row| {
            matches!(
                row,
                PartitionRow::Partition { partition, .. }
                    if partition.start_sector == start_sector
            )
        });
        self.current_partition_idx = preferred_free.or(new_partition).unwrap_or(0);
        self.partition_table_state
            .select(Some(self.current_partition_idx));
        self.input.clear();
        self.input_is_default = false;
        self.pending_size_adjustment = None;
        self.pending_new_start_sector = None;
        self.pending_new_available_bytes = 0;
        self.input_mode = InputMode::Normal;
        self.layer = AppLayer::Edit;
        self.selected_menu = 0;
        self.status =
            "New partition added to draft. Commit saves it; Cancel discards it.".to_string();
    }

    pub(crate) fn delete_selected_partition(&mut self) {
        let Some(disk) = self.selected_disk() else {
            self.status = "No disk selected.".to_string();
            return;
        };

        if disk.is_protected() {
            self.status = disk.guard_message().to_string();
            return;
        }

        let selected = self.current_partition_idx;
        let rows = self.partition_rows();
        let target_index = match rows.get(selected) {
            Some(PartitionRow::Partition { draft_index, .. }) => *draft_index,
            Some(PartitionRow::FreeSpace(_)) => {
                self.status = "Free space cannot be deleted. Use New to allocate it.".to_string();
                return;
            }
            None => {
                self.status = "No partition selected to delete.".to_string();
                return;
            }
        };

        let Some(draft) = self.ensure_draft() else {
            self.status = "No disk selected.".to_string();
            return;
        };

        if draft.partitions[target_index].pending == PendingState::New {
            draft.partitions.remove(target_index);
        } else {
            draft.partitions[target_index].pending = PendingState::Deleted;
        }

        let row_count_after = self.partition_row_count();
        self.current_partition_idx = if row_count_after == 0 {
            0
        } else {
            self.current_partition_idx.min(row_count_after - 1)
        };
        self.partition_table_state
            .select((row_count_after > 0).then_some(self.current_partition_idx));
        self.layer = AppLayer::Edit;
        self.selected_menu = 1;
        self.status =
            "Partition deleted from draft. Commit saves it; Cancel discards it.".to_string();
    }

    pub(crate) fn commit_active_draft(&mut self) {
        if !self.has_active_draft_changes() {
            self.status = "No draft changes to commit.".to_string();
            return;
        }

        self.layer = AppLayer::Main;
        self.focus = Focus::Disks;
        self.selected_menu = 0;
        self.status =
            "Draft committed. Select another disk or choose Write to apply all drafts.".to_string();
    }

    pub(crate) fn start_write_all(&mut self) {
        if self.has_committed_drafts() {
            let preview = self.write_preview();
            if preview.disks.is_empty() {
                self.status = "No committed drafts to write.".to_string();
                return;
            }
            if preview.is_blocked() {
                self.status = preview.status_summary();
                return;
            }

            self.input.clear();
            self.input_is_default = false;
            self.pending_size_adjustment = None;
            self.input_mode = InputMode::WriteConfirm;
            self.write_confirm_kind = WriteConfirmKind::Partition;
            self.layer = AppLayer::Main;
            self.focus = Focus::Disks;
            self.status = preview.status_summary();
            return;
        }

        self.status = "No partition drafts to write.".to_string();
    }

    pub(crate) fn finish_write_confirmation(&mut self) {
        match self.write_confirm_kind {
            WriteConfirmKind::Partition => self.finish_partition_write_confirmation(),
        }
    }

    fn finish_partition_write_confirmation(&mut self) {
        match self.input.trim() {
            "yes" => {
                let preview = self.write_preview();
                if preview.is_blocked() {
                    self.status = preview.status_summary();
                    self.reset_write_confirmation();
                    return;
                }

                self.status = self.write_all_committed_drafts().unwrap_or_else(|err| err);
                self.reset_write_confirmation();
            }
            "no" => {
                self.status = "Write confirmation canceled. Drafts are still pending.".to_string();
                self.reset_write_confirmation();
            }
            _ => self.status = "Please type exactly yes or no.".to_string(),
        }
    }

    fn reset_write_confirmation(&mut self) {
        self.input.clear();
        self.input_mode = InputMode::Normal;
        self.layer = AppLayer::Main;
        self.focus = Focus::Disks;
        self.selected_menu = 0;
    }

    pub(crate) fn write_preview(&self) -> WritePreview {
        WritePreview::from_drafts(&self.disks, &self.drafts)
    }
}
