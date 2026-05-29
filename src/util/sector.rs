use crate::model::{DiskDevice, PartitionTableType};

pub(crate) fn bytes_to_sectors(bytes: u64) -> u64 {
    bytes.div_ceil(512)
}

pub(crate) fn sector_count_to_bytes(sectors: u64) -> u64 {
    sectors.saturating_mul(512)
}

pub(crate) fn disk_sector_count(disk: &DiskDevice) -> u64 {
    disk.size_bytes / 512
}

pub(crate) fn first_usable_sector() -> u64 {
    2048
}

pub(crate) fn last_usable_sector(disk: &DiskDevice, table_type: PartitionTableType) -> u64 {
    let sectors = disk_sector_count(disk);
    match table_type {
        PartitionTableType::Gpt => sectors.saturating_sub(34),
        PartitionTableType::Mbr => sectors.saturating_sub(1),
    }
}
