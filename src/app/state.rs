use std::collections::HashMap;

use ratatui::widgets::TableState;

use crate::model::{
    DiskDevice, DraftConfig, DraftPartition, FreeSpaceSegment, PartitionInfo, PartitionTableType,
};
use crate::util::i18n::Lang;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InputMode {
    Normal,
    Size,
    TypePicker,
    SetPlan,
    WriteConfirm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AppLayer {
    Edit,
    Main,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Focus {
    Disks,
    Partitions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TypePickerColumn {
    PartType,
    TableType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WriteConfirmKind {
    Partition,
}

#[derive(Clone, Debug)]
pub(crate) enum PartitionRow {
    Partition {
        draft_index: usize,
        partition: DraftPartition,
    },
    FreeSpace(FreeSpaceSegment),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SizeAdjustment {
    pub(crate) requested_bytes: u64,
    pub(crate) adjusted_bytes: u64,
}

pub(crate) struct App {
    pub(crate) lang: Lang,
    pub(crate) selected_menu: usize,
    pub(crate) current_disk_idx: usize,
    pub(crate) current_partition_idx: usize,
    pub(crate) disk_table_state: TableState,
    pub(crate) partition_table_state: TableState,
    pub(crate) disks: Vec<DiskDevice>,
    pub(crate) partition_cache: HashMap<String, Vec<PartitionInfo>>,
    pub(crate) drafts: HashMap<String, DraftConfig>,
    pub(crate) default_table_type: PartitionTableType,
    pub(crate) input_mode: InputMode,
    pub(crate) input: String,
    pub(crate) write_confirm_kind: WriteConfirmKind,
    pub(crate) input_is_default: bool,
    pub(crate) pending_size_adjustment: Option<SizeAdjustment>,
    pub(crate) pending_new_start_sector: Option<u64>,
    pub(crate) pending_new_available_bytes: u64,
    pub(crate) type_picker_column: TypePickerColumn,
    pub(crate) type_picker_part_idx: usize,
    pub(crate) type_picker_table_idx: usize,
    pub(crate) layer: AppLayer,
    pub(crate) focus: Focus,
    pub(crate) status: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            lang: Lang::En,
            selected_menu: 1,
            current_disk_idx: 0,
            current_partition_idx: 0,
            disk_table_state: TableState::default(),
            partition_table_state: TableState::default(),
            disks: Vec::new(),
            partition_cache: HashMap::new(),
            drafts: HashMap::new(),
            default_table_type: PartitionTableType::Gpt,
            input_mode: InputMode::Normal,
            input: String::new(),
            write_confirm_kind: WriteConfirmKind::Partition,
            input_is_default: false,
            pending_size_adjustment: None,
            pending_new_start_sector: None,
            pending_new_available_bytes: 0,
            type_picker_column: TypePickerColumn::TableType,
            type_picker_part_idx: 0,
            type_picker_table_idx: 0,
            layer: AppLayer::Main,
            focus: Focus::Disks,
            status: "Main layer. Up/Down selects disks; Select edits current disk draft."
                .to_string(),
        }
    }
}
