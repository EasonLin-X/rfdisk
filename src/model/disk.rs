use crate::model::enums::{DetectedTableLabel, PartitionTableType};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DiskDevice {
    pub(crate) name: String,
    pub(crate) dev_path: String,
    pub(crate) size_bytes: u64,
    pub(crate) kind: DiskKind,
    pub(crate) serial: String,
    pub(crate) guard: DiskGuard,
    pub(crate) model: String,
    pub(crate) table_label: DetectedTableLabel,
    pub(crate) scan_status: ScanStatus,
}

impl DiskDevice {
    pub(crate) fn stable_id(&self) -> String {
        if self.serial.starts_with("NO_SERIAL_") {
            format!("{}:{}", self.name, self.size_bytes)
        } else {
            self.serial.clone()
        }
    }

    pub(crate) fn is_protected(&self) -> bool {
        self.guard.is_protected()
    }

    pub(crate) fn guard_message(&self) -> &'static str {
        match self.guard {
            DiskGuard::None => "Disk is not guarded.",
            DiskGuard::System => "System disk cannot be modified.",
            DiskGuard::Mounted => "Mounted disk cannot be modified. Unmount it first.",
            DiskGuard::Swap => "Swap disk cannot be modified. Disable swap first.",
            DiskGuard::Used => "Disk is in use and cannot be modified.",
        }
    }

    pub(crate) fn editable_table_type(&self) -> Option<PartitionTableType> {
        self.table_label.editable_table_type()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiskGuard {
    None,
    System,
    Mounted,
    Swap,
    #[allow(dead_code)]
    Used,
}

impl DiskGuard {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::System => "system",
            Self::Mounted => "mounted",
            Self::Swap => "swap",
            Self::Used => "used",
        }
    }

    pub(crate) fn is_protected(&self) -> bool {
        matches!(self, Self::System | Self::Used)
    }

    pub(crate) fn is_guarded(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub(crate) fn priority(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Mounted => 1,
            Self::Swap => 2,
            Self::Used => 3,
            Self::System => 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScanStatus {
    pub(crate) source: ScanSource,
    pub(crate) reason: Option<String>,
    pub(crate) permission_limited: bool,
}

impl ScanStatus {
    pub(crate) fn sfdisk() -> Self {
        Self {
            source: ScanSource::Sfdisk,
            reason: None,
            permission_limited: false,
        }
    }

    pub(crate) fn sysfs_fallback(reason: String) -> Self {
        let lowered = reason.to_ascii_lowercase();
        let permission_limited = lowered.contains("permission")
            || lowered.contains("operation not permitted")
            || lowered.contains("access denied");

        Self {
            source: ScanSource::SysfsFallback,
            reason: Some(reason),
            permission_limited,
        }
    }

    pub(crate) fn label(&self) -> &'static str {
        match self.source {
            ScanSource::Sfdisk => "sfdisk",
            ScanSource::SysfsFallback => "sysfs fallback",
        }
    }

    pub(crate) fn is_degraded(&self) -> bool {
        self.source == ScanSource::SysfsFallback
    }

    pub(crate) fn short_reason(&self) -> Option<String> {
        self.reason.as_ref().map(|reason| {
            let trimmed = reason.trim();
            let mut chars = trimmed.chars();
            let short: String = chars.by_ref().take(96).collect();
            if chars.next().is_some() {
                format!("{short}...")
            } else {
                short
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScanSource {
    Sfdisk,
    SysfsFallback,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DiskKind {
    Hdd,
    Ssd,
    Nvme,
    Unknown,
}

impl DiskKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Hdd => "HDD",
            Self::Ssd => "SSD",
            Self::Nvme => "NVMe",
            Self::Unknown => "Unknown",
        }
    }
}
