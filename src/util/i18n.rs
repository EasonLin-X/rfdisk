#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Lang {
    En,
    ZhCn,
}

impl Lang {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "en" | "en-US" | "english" => Some(Self::En),
            "zh" | "zh-CN" | "zh_cn" | "zh-CN.UTF-8" | "cn" | "chinese" | "简体中文" => {
                Some(Self::ZhCn)
            }
            _ => None,
        }
    }

    pub(crate) fn from_locale(value: &str) -> Option<Self> {
        let lower = value.to_ascii_lowercase();
        if lower.starts_with("zh") {
            Some(Self::ZhCn)
        } else if lower.starts_with("en") {
            Some(Self::En)
        } else {
            None
        }
    }

    pub(crate) fn as_config_value(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::ZhCn => "zh-CN",
        }
    }
}

impl Default for Lang {
    fn default() -> Self {
        Self::En
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Msg {
    MainTitle,
    EditTitle,
    SizeTitle,
    TypePickerTitle,
    SetPlanTitle,
    WriteConfirmTitle,
    SizeLabel,
    DefaultLabel,
    TypePickerPrompt,
    SetPlanPrompt,
    ConfirmLabel,
    MainSelect,
    MainRefresh,
    MainSetPlan,
    MainWrite,
    MainQuit,
    EditNew,
    EditDelete,
    EditType,
    EditCommit,
    EditCancel,
    PhysicalDisks,
    PhysicalDisksFocused,
    PhysicalDisksTab,
    NoDiskDetected,
    DiskName,
    DiskGuard,
    DiskPath,
    DiskSize,
    DiskTable,
    DiskKind,
    DiskModel,
    DiskSerial,
    PartitionTable,
    NoSelectedDisk,
    Partition,
    Start,
    PartType,
    TableType,
    Fs,
    Mount,
    Draft,
    FreeSpace,
    Unallocated,
    Current,
    Disabled,
    Status,
    SelectedNone,
    Selected,
    Scan,
    PermissionLimited,
    ScanFallback,
    ScanFallbackDetail,
    Reason,
    CommittedDraftsPrefix,
    CommittedDraftsWrite,
    CommittedDraftsSuffix,
    TypePickerStatusPrefix,
    TypePickerStatusPart,
    TypePickerStatusMiddle,
    TypePickerStatusTable,
    TypePickerStatusSuffix,
    SetPlanStatusPrefix,
    SetPlanUnderDevelopment,
    SetPlanStatusSuffix,
    WriteConfirmHint,
    SetPlanAlphaFocus1,
    SetPlanAlphaFocus2,
    SetPlanReturnLater,
    Press,
    Or,
    ToReturn,
}

pub(crate) fn tr(lang: Lang, msg: Msg) -> &'static str {
    match lang {
        Lang::En => en(msg),
        Lang::ZhCn => zh_cn(msg),
    }
}

pub(crate) fn found_disks(lang: Lang, count: usize) -> String {
    match lang {
        Lang::En => format!("Found {count} disk(s)."),
        Lang::ZhCn => format!("发现 {count} 块磁盘。"),
    }
}

pub(crate) fn refresh_complete(lang: Lang, count: usize) -> String {
    match lang {
        Lang::En => format!("Refresh complete. Found {count} disk(s)."),
        Lang::ZhCn => format!("刷新完成。发现 {count} 块磁盘。"),
    }
}

pub(crate) fn refresh_failed(lang: Lang, err: &str) -> String {
    match lang {
        Lang::En => format!("Refresh failed: {err}"),
        Lang::ZhCn => format!("刷新失败: {err}"),
    }
}

fn en(msg: Msg) -> &'static str {
    match msg {
        Msg::MainTitle => " Main [Left/Right, Enter]  Tab/Select Edit, R Refresh, Q Quit ",
        Msg::EditTitle => {
            " Edit [Left/Right, Enter]  Commit saves draft, Q Cancel, hold W on Delete "
        }
        Msg::SizeTitle => " New Partition Size [Enter confirm, Esc cancel] ",
        Msg::TypePickerTitle => {
            " Type Picker [Left/Right column, Up/Down select, Enter apply, Esc/Q back] "
        }
        Msg::SetPlanTitle => " Set(plan) [under development, Esc/Q back] ",
        Msg::WriteConfirmTitle => {
            " Write Confirmation [type required phrase, Enter confirm, Esc cancel] "
        }
        Msg::SizeLabel => "Size: ",
        Msg::DefaultLabel => "Default: ",
        Msg::TypePickerPrompt => {
            "Choose Part Type for the selected partition or Table Type for draft GPT/MBR."
        }
        Msg::SetPlanPrompt => "Set(plan) is still under development.",
        Msg::ConfirmLabel => "Confirm: ",
        Msg::MainSelect => "Select",
        Msg::MainRefresh => "Refresh",
        Msg::MainSetPlan => "Set(plan)",
        Msg::MainWrite => "Write",
        Msg::MainQuit => "Quit",
        Msg::EditNew => "New",
        Msg::EditDelete => "Delete",
        Msg::EditType => "Type",
        Msg::EditCommit => "Commit",
        Msg::EditCancel => "Cancel",
        Msg::PhysicalDisks => " Physical Disks ",
        Msg::PhysicalDisksFocused => " Physical Disks [focused: Up/Down] ",
        Msg::PhysicalDisksTab => " Physical Disks [Tab to focus] ",
        Msg::NoDiskDetected => {
            "No disk device detected.\nRun with sudo on Linux, then press R to rescan /sys."
        }
        Msg::DiskName => "Name",
        Msg::DiskGuard => "Guard",
        Msg::DiskPath => "Path",
        Msg::DiskSize => "Size",
        Msg::DiskTable => "Table",
        Msg::DiskKind => "Kind",
        Msg::DiskModel => "Model",
        Msg::DiskSerial => "Serial",
        Msg::PartitionTable => " Partition Table ",
        Msg::NoSelectedDisk => "No disk selected.",
        Msg::Partition => "Partition",
        Msg::Start => "Start",
        Msg::PartType => "Part Type",
        Msg::TableType => "Table Type",
        Msg::Fs => "FS",
        Msg::Mount => "Mount",
        Msg::Draft => "Draft",
        Msg::FreeSpace => "[free space]",
        Msg::Unallocated => "unallocated",
        Msg::Current => "current",
        Msg::Disabled => "disabled",
        Msg::Status => " Status ",
        Msg::SelectedNone => "selected: none",
        Msg::Selected => "selected: ",
        Msg::Scan => " | scan: ",
        Msg::PermissionLimited => "Permission limited: ",
        Msg::ScanFallback => "Scan fallback: ",
        Msg::ScanFallbackDetail => {
            "using /sys fallback; partition type and table metadata may be incomplete."
        }
        Msg::Reason => " reason: ",
        Msg::CommittedDraftsPrefix => "Committed drafts exist. Choose ",
        Msg::CommittedDraftsWrite => "Write",
        Msg::CommittedDraftsSuffix => " to apply all, or Select a highlighted disk to edit/cancel.",
        Msg::TypePickerStatusPrefix => "Type picker: ",
        Msg::TypePickerStatusPart => "Part Type",
        Msg::TypePickerStatusMiddle => " changes the selected partition draft; ",
        Msg::TypePickerStatusTable => "Table Type",
        Msg::TypePickerStatusSuffix => " changes the draft GPT/MBR target.",
        Msg::SetPlanStatusPrefix => "Set(plan): ",
        Msg::SetPlanUnderDevelopment => "still under development",
        Msg::SetPlanStatusSuffix => ". The alpha focuses on partition-table editing.",
        Msg::WriteConfirmHint => "  (type yes or no)",
        Msg::SetPlanAlphaFocus1 => "The alpha release focuses on partition-table editing:",
        Msg::SetPlanAlphaFocus2 => "scan, refresh, New, Delete, Type, preview, and Write.",
        Msg::SetPlanReturnLater => "Filesystem, mount, swap, and RAID setup will return later.",
        Msg::Press => "Press ",
        Msg::Or => " or ",
        Msg::ToReturn => " to return.",
    }
}

fn zh_cn(msg: Msg) -> &'static str {
    match msg {
        Msg::MainTitle => " 主菜单 [Left/Right, Enter]  Tab/选择 编辑, R 刷新, Q 退出 ",
        Msg::EditTitle => " 编辑 [Left/Right, Enter]  提交保存草稿, Q 取消, Delete 上长按 W ",
        Msg::SizeTitle => " 新建分区大小 [Enter 确认, Esc 取消] ",
        Msg::TypePickerTitle => {
            " 类型选择 [Left/Right 切换列, Up/Down 选择, Enter 应用, Esc/Q 返回] "
        }
        Msg::SetPlanTitle => " 设置计划 [还在研发中, Esc/Q 返回] ",
        Msg::WriteConfirmTitle => " 写入确认 [输入指定内容, Enter 确认, Esc 取消] ",
        Msg::SizeLabel => "大小: ",
        Msg::DefaultLabel => "默认: ",
        Msg::TypePickerPrompt => "选择当前分区的分区用途类型，或选择草稿的 GPT/MBR 分区表类型。",
        Msg::SetPlanPrompt => "设置计划还在研发中。",
        Msg::ConfirmLabel => "确认: ",
        Msg::MainSelect => "选择",
        Msg::MainRefresh => "刷新",
        Msg::MainSetPlan => "设置计划",
        Msg::MainWrite => "写入",
        Msg::MainQuit => "退出",
        Msg::EditNew => "新建",
        Msg::EditDelete => "删除",
        Msg::EditType => "类型",
        Msg::EditCommit => "提交",
        Msg::EditCancel => "取消",
        Msg::PhysicalDisks => " 物理磁盘 ",
        Msg::PhysicalDisksFocused => " 物理磁盘 [焦点: Up/Down] ",
        Msg::PhysicalDisksTab => " 物理磁盘 [Tab 切换焦点] ",
        Msg::NoDiskDetected => {
            "未检测到磁盘设备。\n请在 Linux 上使用 sudo 运行，然后按 R 重新扫描 /sys。"
        }
        Msg::DiskName => "磁盘名",
        Msg::DiskGuard => "保护",
        Msg::DiskPath => "路径",
        Msg::DiskSize => "大小",
        Msg::DiskTable => "分区表",
        Msg::DiskKind => "类型",
        Msg::DiskModel => "型号",
        Msg::DiskSerial => "序列号",
        Msg::PartitionTable => " 分区表 ",
        Msg::NoSelectedDisk => "未选择磁盘。",
        Msg::Partition => "分区",
        Msg::Start => "起点",
        Msg::PartType => "分区用途",
        Msg::TableType => "分区表类型",
        Msg::Fs => "文件系统",
        Msg::Mount => "挂载点",
        Msg::Draft => "草稿",
        Msg::FreeSpace => "[空闲空间]",
        Msg::Unallocated => "未分配",
        Msg::Current => "当前",
        Msg::Disabled => "不可用",
        Msg::Status => " 状态 ",
        Msg::SelectedNone => "已选: 无",
        Msg::Selected => "已选: ",
        Msg::Scan => " | 扫描: ",
        Msg::PermissionLimited => "权限受限: ",
        Msg::ScanFallback => "扫描降级: ",
        Msg::ScanFallbackDetail => "正在使用 /sys fallback；分区类型和分区表元数据可能不完整。",
        Msg::Reason => " 原因: ",
        Msg::CommittedDraftsPrefix => "存在已提交草稿。选择 ",
        Msg::CommittedDraftsWrite => "写入",
        Msg::CommittedDraftsSuffix => " 应用全部，或选择高亮磁盘继续编辑/取消。",
        Msg::TypePickerStatusPrefix => "类型选择: ",
        Msg::TypePickerStatusPart => "分区用途",
        Msg::TypePickerStatusMiddle => " 修改当前分区草稿；",
        Msg::TypePickerStatusTable => "分区表类型",
        Msg::TypePickerStatusSuffix => " 修改草稿 GPT/MBR 目标。",
        Msg::SetPlanStatusPrefix => "设置计划: ",
        Msg::SetPlanUnderDevelopment => "还在研发中",
        Msg::SetPlanStatusSuffix => "。alpha 版本专注分区表编辑。",
        Msg::WriteConfirmHint => "  (输入 yes 或 no)",
        Msg::SetPlanAlphaFocus1 => "alpha 版本专注分区表编辑:",
        Msg::SetPlanAlphaFocus2 => "扫描、刷新、新建、删除、类型、预览和写入。",
        Msg::SetPlanReturnLater => "文件系统、挂载、swap 和 RAID 配置会在后续版本回归。",
        Msg::Press => "按 ",
        Msg::Or => " 或 ",
        Msg::ToReturn => " 返回。",
    }
}
