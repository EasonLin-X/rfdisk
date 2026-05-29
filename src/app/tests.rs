use crossterm::event::{KeyCode, KeyEventKind};

use crate::{
    app::{
        controller::MAIN_MENU_LEN,
        event,
        state::{App, AppLayer, Focus, InputMode, TypePickerColumn, WriteConfirmKind},
    },
    model::{
        ChangeKind, DetectedTableLabel, DiskDevice, DiskGuard, DiskKind, PartitionInfo,
        PartitionTableType, RiskLevel, ScanStatus,
    },
};

fn test_disk() -> DiskDevice {
    DiskDevice {
        name: "sdb".to_string(),
        dev_path: "/dev/sdb".to_string(),
        size_bytes: 20 * 1024 * 1024 * 1024,
        kind: DiskKind::Hdd,
        serial: "test-disk".to_string(),
        guard: DiskGuard::None,
        model: "test".to_string(),
        table_label: DetectedTableLabel::Gpt,
        scan_status: ScanStatus::sfdisk(),
    }
}

fn protected_disk() -> DiskDevice {
    DiskDevice {
        guard: DiskGuard::System,
        ..test_disk()
    }
}

fn test_partition() -> PartitionInfo {
    PartitionInfo {
        dev_path: "/dev/sdb1".to_string(),
        start_sector: 2048,
        size_bytes: 1024 * 1024,
        fs_type: "ext4".to_string(),
        partition_type: "Linux filesystem".to_string(),
        partition_type_raw: "0FC63DAF-8483-4772-8E79-3D69D8477DE4".to_string(),
        uuid: String::new(),
        part_uuid: "part-uuid".to_string(),
        label: String::new(),
        part_name: "root".to_string(),
        mount_points: Vec::new(),
        is_swap: false,
    }
}

fn metadata_rich_partition() -> PartitionInfo {
    PartitionInfo {
        fs_type: "xfs".to_string(),
        uuid: "fs-uuid".to_string(),
        part_uuid: "part-uuid".to_string(),
        label: "data-label".to_string(),
        part_name: "data-part".to_string(),
        mount_points: vec!["/mnt/data".to_string()],
        is_swap: true,
        ..test_partition()
    }
}

#[test]
fn active_partitions_use_cached_partition_info_without_draft() {
    let disk = test_disk();
    let mut app = App {
        disks: vec![disk.clone()],
        current_disk_idx: 0,
        ..App::default()
    };
    app.partition_cache
        .insert(disk.stable_id(), vec![test_partition()]);

    let partitions = app.active_partitions();

    assert_eq!(partitions.len(), 1);
    assert_eq!(partitions[0].display_name, "/dev/sdb1");
    assert_eq!(partitions[0].part_uuid, "part-uuid");
}

#[test]
fn main_menu_has_no_type_slot() {
    let mut app = App::default();
    assert_eq!(app.menu_len(), MAIN_MENU_LEN);

    app.selected_menu = 2;
    let should_quit = event::handle_key_event(&mut app, KeyCode::Enter, KeyEventKind::Press);

    assert!(!should_quit);
    assert_eq!(app.input_mode, InputMode::Normal);
    assert_eq!(app.status, "No disk selected.");
}

#[test]
fn edit_type_enters_type_picker() {
    let disk = test_disk();
    let mut app = App {
        disks: vec![disk],
        current_disk_idx: 0,
        layer: AppLayer::Edit,
        focus: Focus::Partitions,
        selected_menu: 2,
        ..App::default()
    };

    event::handle_key_event(&mut app, KeyCode::Enter, KeyEventKind::Press);

    assert_eq!(app.input_mode, InputMode::TypePicker);
    assert_eq!(app.type_picker_column, TypePickerColumn::TableType);
}

#[test]
fn part_type_picker_changes_selected_partition_draft() {
    let disk = test_disk();
    let mut app = App {
        disks: vec![disk.clone()],
        current_disk_idx: 0,
        layer: AppLayer::Edit,
        focus: Focus::Partitions,
        selected_menu: 2,
        ..App::default()
    };
    app.partition_cache
        .insert(disk.stable_id(), vec![test_partition()]);
    app.enter_type_picker();
    app.type_picker_column = TypePickerColumn::PartType;
    app.type_picker_part_idx = 1;

    app.apply_type_picker_selection();

    let draft = app.active_draft().unwrap();
    assert_eq!(draft.partitions[0].partition_type, "Linux swap");
    assert_eq!(
        draft.partitions[0].partition_type_raw,
        "0657FD6D-A4AB-43C4-84E5-0933C84B4F4F"
    );
    assert!(app.status.contains("does not format"));
}

#[test]
fn part_type_picker_uses_mbr_type_code() {
    let mut disk = test_disk();
    disk.table_label = DetectedTableLabel::Mbr;
    let mut app = App {
        disks: vec![disk.clone()],
        current_disk_idx: 0,
        layer: AppLayer::Edit,
        focus: Focus::Partitions,
        selected_menu: 2,
        ..App::default()
    };
    app.partition_cache
        .insert(disk.stable_id(), vec![test_partition()]);
    app.enter_type_picker();
    app.type_picker_column = TypePickerColumn::PartType;
    app.type_picker_part_idx = 1;

    app.apply_type_picker_selection();

    assert_eq!(
        app.active_draft().unwrap().partitions[0].partition_type_raw,
        "82"
    );
}

#[test]
fn bios_boot_is_disabled_for_mbr_part_type_picker() {
    let mut disk = test_disk();
    disk.table_label = DetectedTableLabel::Mbr;
    let mut app = App {
        disks: vec![disk.clone()],
        current_disk_idx: 0,
        layer: AppLayer::Edit,
        focus: Focus::Partitions,
        selected_menu: 2,
        ..App::default()
    };
    app.partition_cache
        .insert(disk.stable_id(), vec![test_partition()]);
    app.enter_type_picker();
    app.type_picker_column = TypePickerColumn::PartType;
    app.type_picker_part_idx = 6;

    app.apply_type_picker_selection();

    assert!(app.drafts.is_empty());
    assert!(app.status.contains("BIOS boot is not supported for MBR"));
}

#[test]
fn part_type_picker_preserves_filesystem_metadata() {
    let disk = test_disk();
    let mut app = App {
        disks: vec![disk.clone()],
        current_disk_idx: 0,
        layer: AppLayer::Edit,
        focus: Focus::Partitions,
        selected_menu: 2,
        ..App::default()
    };
    app.partition_cache
        .insert(disk.stable_id(), vec![metadata_rich_partition()]);
    app.enter_type_picker();
    app.type_picker_column = TypePickerColumn::PartType;
    app.type_picker_part_idx = 3;

    app.apply_type_picker_selection();

    let partition = &app.active_draft().unwrap().partitions[0];
    assert_eq!(partition.partition_type, "Microsoft basic data");
    assert_eq!(partition.fs_type, "xfs");
    assert_eq!(partition.uuid, "fs-uuid");
    assert_eq!(partition.part_uuid, "part-uuid");
    assert_eq!(partition.fs_label, "data-label");
    assert_eq!(partition.part_name, "data-part");
    assert_eq!(partition.mount_points, vec!["/mnt/data".to_string()]);
    assert!(partition.is_swap);
}

#[test]
fn new_partition_defaults_to_linux_filesystem_type() {
    let disk = test_disk();
    let mut app = App {
        disks: vec![disk],
        current_disk_idx: 0,
        layer: AppLayer::Edit,
        focus: Focus::Partitions,
        selected_menu: 0,
        ..App::default()
    };

    app.start_new_partition();
    app.add_new_partition_from_input();

    let partition = &app.active_draft().unwrap().partitions[0];
    assert_eq!(partition.partition_type, "Linux filesystem");
    assert_eq!(
        partition.partition_type_raw,
        PartitionTableType::Gpt.default_partition_type_raw()
    );
    assert!(partition.fs_type.is_empty());
}

#[test]
fn part_type_picker_requires_partition_before_free_space() {
    let disk = test_disk();
    let mut app = App {
        disks: vec![disk],
        current_disk_idx: 0,
        layer: AppLayer::Edit,
        focus: Focus::Partitions,
        selected_menu: 2,
        ..App::default()
    };
    app.enter_type_picker();
    app.type_picker_column = TypePickerColumn::PartType;

    app.apply_type_picker_selection();

    assert!(app.drafts.is_empty());
    assert_eq!(app.status, "Create a partition first, then set its type.");
}

#[test]
fn changed_existing_part_type_is_high_risk_recreate() {
    let disk = test_disk();
    let mut app = App {
        disks: vec![disk.clone()],
        current_disk_idx: 0,
        layer: AppLayer::Edit,
        focus: Focus::Partitions,
        selected_menu: 2,
        ..App::default()
    };
    app.partition_cache
        .insert(disk.stable_id(), vec![test_partition()]);
    app.enter_type_picker();
    app.type_picker_column = TypePickerColumn::PartType;
    app.type_picker_part_idx = 1;
    app.apply_type_picker_selection();

    let preview = app.write_preview();

    assert_eq!(preview.risk, RiskLevel::High);
    assert!(preview.disks[0]
        .changes
        .iter()
        .any(|change| change.kind == ChangeKind::Recreate));
    assert!(preview.status_summary().contains("High risk"));
}

#[test]
fn mounted_part_type_change_is_blocked_by_preview() {
    let disk = test_disk();
    let mut partition = test_partition();
    partition.mount_points = vec!["/mnt/data".to_string()];
    let mut app = App {
        disks: vec![disk.clone()],
        current_disk_idx: 0,
        layer: AppLayer::Edit,
        focus: Focus::Partitions,
        selected_menu: 2,
        ..App::default()
    };
    app.partition_cache
        .insert(disk.stable_id(), vec![partition]);
    app.enter_type_picker();
    app.type_picker_column = TypePickerColumn::PartType;
    app.type_picker_part_idx = 1;
    app.apply_type_picker_selection();

    let preview = app.write_preview();

    assert!(preview.is_blocked());
}

#[test]
fn table_type_picker_changes_empty_disk_draft() {
    let disk = test_disk();
    let mut app = App {
        disks: vec![disk],
        current_disk_idx: 0,
        layer: AppLayer::Edit,
        focus: Focus::Partitions,
        selected_menu: 2,
        ..App::default()
    };
    app.enter_type_picker();
    app.type_picker_table_idx = 1;

    app.apply_type_picker_selection();

    let draft = app.active_draft().unwrap();
    assert_eq!(draft.table_type, PartitionTableType::Mbr);
    assert_eq!(app.input_mode, InputMode::Normal);
    assert!(app.status.contains("Low risk"));
}

#[test]
fn table_type_picker_warns_for_non_empty_disk() {
    let disk = test_disk();
    let mut app = App {
        disks: vec![disk.clone()],
        current_disk_idx: 0,
        layer: AppLayer::Edit,
        focus: Focus::Partitions,
        selected_menu: 2,
        ..App::default()
    };
    app.partition_cache
        .insert(disk.stable_id(), vec![test_partition()]);
    app.enter_type_picker();
    app.type_picker_table_idx = 1;

    app.apply_type_picker_selection();

    assert_eq!(
        app.active_draft().unwrap().table_type,
        PartitionTableType::Mbr
    );
    assert!(app.status.contains("High risk"));
    assert!(app
        .write_preview()
        .disks
        .iter()
        .any(|disk| disk.risk == RiskLevel::High));
}

#[test]
fn protected_disk_blocks_table_type_picker() {
    let disk = protected_disk();
    let mut app = App {
        disks: vec![disk],
        current_disk_idx: 0,
        layer: AppLayer::Edit,
        focus: Focus::Partitions,
        selected_menu: 2,
        ..App::default()
    };
    app.enter_type_picker();
    app.type_picker_table_idx = 1;

    app.apply_type_picker_selection();

    assert!(app.drafts.is_empty());
    assert_eq!(app.status, "System disk cannot be modified.");
}

#[test]
fn set_plan_enters_from_main_menu_with_existing_partition() {
    let disk = test_disk();
    let mut app = App {
        disks: vec![disk.clone()],
        current_disk_idx: 0,
        selected_menu: 2,
        ..App::default()
    };
    app.partition_cache
        .insert(disk.stable_id(), vec![test_partition()]);

    let should_quit = event::handle_key_event(&mut app, KeyCode::Enter, KeyEventKind::Press);

    assert!(!should_quit);
    assert_eq!(app.input_mode, InputMode::SetPlan);
    assert!(app.status.contains("still under development"));
}

#[test]
fn set_plan_entry_is_visible_but_development_only_with_partition_draft() {
    let disk = test_disk();
    let mut app = App {
        disks: vec![disk.clone()],
        current_disk_idx: 0,
        selected_menu: 2,
        ..App::default()
    };
    app.partition_cache
        .insert(disk.stable_id(), vec![test_partition()]);
    app.ensure_draft();

    app.enter_set_plan();

    assert_eq!(app.input_mode, InputMode::SetPlan);
    assert!(app.status.contains("still under development"));
}

#[test]
fn set_plan_keys_do_not_create_drafts_in_alpha() {
    let disk = test_disk();
    let mut app = App {
        disks: vec![disk.clone()],
        current_disk_idx: 0,
        ..App::default()
    };
    app.partition_cache
        .insert(disk.stable_id(), vec![test_partition()]);
    app.enter_set_plan();
    event::handle_key_event(&mut app, KeyCode::Enter, KeyEventKind::Press);
    event::handle_key_event(&mut app, KeyCode::Down, KeyEventKind::Press);
    event::handle_key_event(&mut app, KeyCode::Right, KeyEventKind::Press);

    assert_eq!(app.input_mode, InputMode::SetPlan);
    assert!(app.status.contains("still under development"));
}

#[test]
fn partition_drafts_start_partition_write_confirmation() {
    let disk = test_disk();
    let mut app = App {
        disks: vec![disk.clone()],
        current_disk_idx: 0,
        ..App::default()
    };
    app.partition_cache
        .insert(disk.stable_id(), vec![test_partition()]);
    app.ensure_draft().unwrap().table_type = PartitionTableType::Mbr;

    app.start_write_all();

    assert_eq!(app.input_mode, InputMode::WriteConfirm);
    assert_eq!(app.write_confirm_kind, WriteConfirmKind::Partition);
}
