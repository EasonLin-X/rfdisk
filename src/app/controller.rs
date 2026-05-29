use crate::{
    app::state::{App, AppLayer, Focus, InputMode},
    backend::{disks::scan_disks, rescan::trigger_scsi_scan, scan::scan_partitions},
    model::{DiskDevice, DraftConfig, PartitionTableType},
    util::i18n::{found_disks, refresh_complete, refresh_failed, tr, Lang, Msg},
};

pub(crate) const MAIN_MENU_LEN: usize = 5;
pub(crate) const EDIT_MENU_LEN: usize = 5;

impl App {
    pub(crate) fn new_with_lang(lang: Lang) -> Self {
        let mut app = Self {
            lang,
            ..Self::default()
        };
        app.refresh(false);
        app
    }

    pub(crate) fn refresh(&mut self, rescan_scsi: bool) {
        let saved_id = self
            .disks
            .get(self.current_disk_idx)
            .map(|disk| disk.stable_id());

        let rescan_result = if rescan_scsi {
            trigger_scsi_scan()
        } else {
            Ok(())
        };

        self.disks = scan_disks();
        self.partition_cache = self
            .disks
            .iter()
            .map(|disk| (disk.stable_id(), scan_partitions(disk)))
            .collect();

        self.current_disk_idx = saved_id
            .and_then(|id| self.disks.iter().position(|disk| disk.stable_id() == id))
            .unwrap_or(0);
        self.clamp_selection();

        self.status = match rescan_result {
            Ok(()) if rescan_scsi => refresh_complete(self.lang, self.disks.len()),
            Ok(()) => found_disks(self.lang, self.disks.len()),
            Err(err) => refresh_failed(self.lang, &err.to_string()),
        };
    }

    pub(crate) fn clamp_selection(&mut self) {
        if self.disks.is_empty() {
            self.current_disk_idx = 0;
            self.current_partition_idx = 0;
            self.disk_table_state.select(None);
            self.partition_table_state.select(None);
        } else {
            self.current_disk_idx = self.current_disk_idx.min(self.disks.len() - 1);
            self.clamp_partition_selection();
            self.disk_table_state.select(Some(self.current_disk_idx));
        }
    }

    pub(crate) fn clamp_partition_selection(&mut self) {
        let row_count = self.partition_row_count();
        if row_count == 0 {
            self.current_partition_idx = 0;
            self.partition_table_state.select(None);
        } else {
            self.current_partition_idx = self.current_partition_idx.min(row_count - 1);
            self.partition_table_state
                .select(Some(self.current_partition_idx));
        }
    }

    pub(crate) fn selected_disk(&self) -> Option<&DiskDevice> {
        self.disks.get(self.current_disk_idx)
    }

    pub(crate) fn active_table_type(&self) -> PartitionTableType {
        self.active_draft()
            .map(|draft| draft.table_type)
            .or_else(|| {
                self.selected_disk()
                    .and_then(DiskDevice::editable_table_type)
            })
            .unwrap_or(self.default_table_type)
    }

    pub(crate) fn has_active_draft_changes(&self) -> bool {
        self.active_draft().is_some_and(DraftConfig::has_changes)
    }

    pub(crate) fn has_committed_drafts(&self) -> bool {
        self.drafts.values().any(DraftConfig::has_changes)
    }

    pub(crate) fn menu_len(&self) -> usize {
        match self.layer {
            AppLayer::Main => MAIN_MENU_LEN,
            AppLayer::Edit => EDIT_MENU_LEN,
        }
    }

    pub(crate) fn ensure_draft(&mut self) -> Option<&mut DraftConfig> {
        let disk = self.selected_disk()?.clone();
        let table_type = self.active_table_type();
        let disk_id = disk.stable_id();
        let cached_partitions = self.cached_partitions_for(&disk);

        self.drafts
            .entry(disk_id)
            .or_insert_with(|| DraftConfig::from_partitions(cached_partitions, table_type));

        let disk_id = disk.stable_id();
        self.drafts.get_mut(&disk_id)
    }

    pub(crate) fn cancel_draft(&mut self) {
        if let Some(disk_id) = self.active_disk_id() {
            self.drafts.remove(&disk_id);
        }
        self.input.clear();
        self.input_is_default = false;
        self.pending_size_adjustment = None;
        self.pending_new_start_sector = None;
        self.pending_new_available_bytes = 0;
        self.input_mode = InputMode::Normal;
        self.layer = AppLayer::Main;
        self.focus = Focus::Disks;
        self.selected_menu = 0;
        self.current_partition_idx = 0;
        self.partition_table_state.select(Some(0));
        self.status = match self.lang {
            Lang::En => "Draft canceled. Returned to main layer.".to_string(),
            Lang::ZhCn => "草稿已取消。已返回主菜单。".to_string(),
        };
    }

    pub(crate) fn enter_edit_layer(&mut self) {
        if self.selected_disk().is_none() {
            self.status = tr(self.lang, Msg::NoSelectedDisk).to_string();
            return;
        }

        self.layer = AppLayer::Edit;
        self.focus = Focus::Partitions;
        self.selected_menu = 0;
        self.current_partition_idx = 0;
        self.partition_table_state.select(Some(0));
        self.status = match self.lang {
            Lang::En => "Edit layer. Commit saves this disk draft; Cancel discards it.".to_string(),
            Lang::ZhCn => "编辑层。提交会保存当前磁盘草稿；取消会丢弃它。".to_string(),
        };
    }

    pub(crate) fn leave_edit_layer(&mut self) {
        self.layer = AppLayer::Main;
        self.focus = Focus::Disks;
        self.selected_menu = 0;
        self.current_partition_idx = 0;
        self.partition_table_state.select(Some(0));
        self.status = match self.lang {
            Lang::En => "Main layer. Up/Down selects disks.".to_string(),
            Lang::ZhCn => "主菜单。使用 Up/Down 选择磁盘。".to_string(),
        };
    }

    pub(crate) fn can_move_disks(&self) -> bool {
        self.input_mode == InputMode::Normal && self.layer == AppLayer::Main
    }
}
