#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PartitionTableType {
    Gpt,
    Mbr,
}

impl PartitionTableType {
    pub(crate) const ALL: [Self; 2] = [Self::Gpt, Self::Mbr];

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Gpt => "GPT",
            Self::Mbr => "MBR",
        }
    }

    pub(crate) fn sfdisk_label(self) -> &'static str {
        match self {
            Self::Gpt => "gpt",
            Self::Mbr => "dos",
        }
    }

    pub(crate) fn default_partition_type_raw(self) -> &'static str {
        match self {
            Self::Gpt => "0FC63DAF-8483-4772-8E79-3D69D8477DE4",
            Self::Mbr => "83",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DetectedTableLabel {
    Gpt,
    Mbr,
    Sgi,
    Sun,
    Bsd,
    Mac,
    Loop,
    Unknown(String),
}

impl DetectedTableLabel {
    pub(crate) fn from_sfdisk_label(label: Option<&str>) -> Self {
        let Some(label) = label else {
            return Self::Unknown("none".to_string());
        };

        match label.trim().to_ascii_lowercase().as_str() {
            "gpt" => Self::Gpt,
            "dos" | "mbr" => Self::Mbr,
            "sgi" => Self::Sgi,
            "sun" => Self::Sun,
            "bsd" => Self::Bsd,
            "mac" => Self::Mac,
            "loop" => Self::Loop,
            other => Self::Unknown(other.to_string()),
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Gpt => "GPT",
            Self::Mbr => "MBR",
            Self::Sgi => "SGI",
            Self::Sun => "SUN",
            Self::Bsd => "BSD",
            Self::Mac => "MAC",
            Self::Loop => "LOOP",
            Self::Unknown(_) => "Unknown",
        }
    }

    pub(crate) fn editable_table_type(&self) -> Option<PartitionTableType> {
        match self {
            Self::Gpt => Some(PartitionTableType::Gpt),
            Self::Mbr => Some(PartitionTableType::Mbr),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PartitionTypeChoice {
    pub(crate) name: &'static str,
    pub(crate) gpt_type: Option<&'static str>,
    pub(crate) mbr_type: Option<&'static str>,
}

impl PartitionTypeChoice {
    pub(crate) fn raw_for(self, table_type: PartitionTableType) -> Option<&'static str> {
        match table_type {
            PartitionTableType::Gpt => self.gpt_type,
            PartitionTableType::Mbr => self.mbr_type,
        }
    }

    pub(crate) fn is_supported(self, table_type: PartitionTableType) -> bool {
        self.raw_for(table_type).is_some()
    }
}

pub(crate) const COMMON_PARTITION_TYPES: &[PartitionTypeChoice] = &[
    PartitionTypeChoice {
        name: "Linux filesystem",
        gpt_type: Some("0FC63DAF-8483-4772-8E79-3D69D8477DE4"),
        mbr_type: Some("83"),
    },
    PartitionTypeChoice {
        name: "Linux swap",
        gpt_type: Some("0657FD6D-A4AB-43C4-84E5-0933C84B4F4F"),
        mbr_type: Some("82"),
    },
    PartitionTypeChoice {
        name: "EFI System",
        gpt_type: Some("C12A7328-F81F-11D2-BA4B-00A0C93EC93B"),
        mbr_type: Some("EF"),
    },
    PartitionTypeChoice {
        name: "Microsoft basic data",
        gpt_type: Some("EBD0A0A2-B9E5-4433-87C0-68B6B72699C7"),
        mbr_type: Some("07"),
    },
    PartitionTypeChoice {
        name: "Linux LVM",
        gpt_type: Some("E6D6D379-F507-44C2-A23C-238F2A3DF928"),
        mbr_type: Some("8E"),
    },
    PartitionTypeChoice {
        name: "Linux RAID",
        gpt_type: Some("A19D880F-05FC-4D3B-A006-743F0F84911E"),
        mbr_type: Some("FD"),
    },
    PartitionTypeChoice {
        name: "BIOS boot",
        gpt_type: Some("21686148-6449-6E6F-744E-656564454649"),
        mbr_type: None,
    },
    PartitionTypeChoice {
        name: "Windows recovery",
        gpt_type: Some("DE94BBA4-06D1-4D40-A16A-BFD50179D6AC"),
        mbr_type: Some("27"),
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_partition_type_order_is_stable() {
        let names: Vec<&str> = COMMON_PARTITION_TYPES
            .iter()
            .map(|choice| choice.name)
            .collect();

        assert_eq!(
            names,
            vec![
                "Linux filesystem",
                "Linux swap",
                "EFI System",
                "Microsoft basic data",
                "Linux LVM",
                "Linux RAID",
                "BIOS boot",
                "Windows recovery",
            ]
        );
    }

    #[test]
    fn linux_filesystem_is_the_default_partition_type() {
        let linux = COMMON_PARTITION_TYPES[0];

        assert_eq!(linux.name, "Linux filesystem");
        assert_eq!(
            linux.raw_for(PartitionTableType::Gpt),
            Some(PartitionTableType::Gpt.default_partition_type_raw())
        );
        assert_eq!(
            linux.raw_for(PartitionTableType::Mbr),
            Some(PartitionTableType::Mbr.default_partition_type_raw())
        );
    }

    #[test]
    fn common_partition_type_raw_values_are_stable() {
        let expected = [
            (
                "Linux filesystem",
                Some("0FC63DAF-8483-4772-8E79-3D69D8477DE4"),
                Some("83"),
            ),
            (
                "Linux swap",
                Some("0657FD6D-A4AB-43C4-84E5-0933C84B4F4F"),
                Some("82"),
            ),
            (
                "EFI System",
                Some("C12A7328-F81F-11D2-BA4B-00A0C93EC93B"),
                Some("EF"),
            ),
            (
                "Microsoft basic data",
                Some("EBD0A0A2-B9E5-4433-87C0-68B6B72699C7"),
                Some("07"),
            ),
            (
                "Linux LVM",
                Some("E6D6D379-F507-44C2-A23C-238F2A3DF928"),
                Some("8E"),
            ),
            (
                "Linux RAID",
                Some("A19D880F-05FC-4D3B-A006-743F0F84911E"),
                Some("FD"),
            ),
            (
                "BIOS boot",
                Some("21686148-6449-6E6F-744E-656564454649"),
                None,
            ),
            (
                "Windows recovery",
                Some("DE94BBA4-06D1-4D40-A16A-BFD50179D6AC"),
                Some("27"),
            ),
        ];

        for (choice, (name, gpt, mbr)) in COMMON_PARTITION_TYPES.iter().zip(expected) {
            assert_eq!(choice.name, name);
            assert_eq!(choice.raw_for(PartitionTableType::Gpt), gpt);
            assert_eq!(choice.raw_for(PartitionTableType::Mbr), mbr);
        }
    }

    #[test]
    fn bios_boot_is_not_supported_on_mbr() {
        let bios_boot = COMMON_PARTITION_TYPES
            .iter()
            .find(|choice| choice.name == "BIOS boot")
            .unwrap();

        assert!(bios_boot.is_supported(PartitionTableType::Gpt));
        assert!(!bios_boot.is_supported(PartitionTableType::Mbr));
    }

    #[test]
    fn detected_table_label_maps_common_sfdisk_labels() {
        assert_eq!(
            DetectedTableLabel::from_sfdisk_label(Some("gpt")),
            DetectedTableLabel::Gpt
        );
        assert_eq!(
            DetectedTableLabel::from_sfdisk_label(Some("dos")),
            DetectedTableLabel::Mbr
        );
        assert_eq!(
            DetectedTableLabel::from_sfdisk_label(Some("mbr")),
            DetectedTableLabel::Mbr
        );
        assert_eq!(
            DetectedTableLabel::from_sfdisk_label(Some("sgi")),
            DetectedTableLabel::Sgi
        );
        assert_eq!(
            DetectedTableLabel::from_sfdisk_label(Some("sun")),
            DetectedTableLabel::Sun
        );
        assert_eq!(
            DetectedTableLabel::from_sfdisk_label(Some("bsd")),
            DetectedTableLabel::Bsd
        );
        assert_eq!(
            DetectedTableLabel::from_sfdisk_label(Some("mac")),
            DetectedTableLabel::Mac
        );
        assert_eq!(
            DetectedTableLabel::from_sfdisk_label(Some("loop")),
            DetectedTableLabel::Loop
        );
    }

    #[test]
    fn detected_table_label_keeps_unknown_raw_label() {
        let label = DetectedTableLabel::from_sfdisk_label(Some("weird"));

        assert_eq!(label.as_str(), "Unknown");
        assert!(matches!(label, DetectedTableLabel::Unknown(ref raw) if raw == "weird"));
        assert_eq!(label.editable_table_type(), None);
    }
}
