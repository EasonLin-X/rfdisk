use crate::{
    algo::alignment::align_up,
    model::{DiskDevice, DraftPartition, FreeSpaceSegment, PartitionTableType, PendingState},
    util::sector::{bytes_to_sectors, first_usable_sector, last_usable_sector},
};

pub(crate) fn calculate_free_space(
    disk: &DiskDevice,
    table_type: PartitionTableType,
    partitions: &[DraftPartition],
    alignment: u64,
) -> Vec<FreeSpaceSegment> {
    let first = first_usable_sector();
    let last = last_usable_sector(disk, table_type);
    if first > last {
        return Vec::new();
    }

    let mut occupied: Vec<(u64, u64)> = partitions
        .iter()
        .filter(|partition| partition.pending != PendingState::Deleted)
        .filter_map(|partition| {
            let size = bytes_to_sectors(partition.size_bytes);
            if size == 0 {
                return None;
            }
            let start = partition.start_sector.max(first);
            let end = partition
                .start_sector
                .saturating_add(size)
                .saturating_sub(1)
                .min(last);
            (start <= end).then_some((start, end))
        })
        .collect();

    occupied.sort_by_key(|(start, _)| *start);

    let mut free = Vec::new();
    let mut cursor = first;

    for (start, end) in occupied {
        if start > cursor {
            push_free_segment(&mut free, cursor, start.saturating_sub(1), alignment);
        }
        cursor = cursor.max(end.saturating_add(1));
        if cursor > last {
            return free;
        }
    }

    if cursor <= last {
        push_free_segment(&mut free, cursor, last, alignment);
    }

    free
}

fn push_free_segment(free: &mut Vec<FreeSpaceSegment>, start: u64, end: u64, alignment: u64) {
    if start > end {
        return;
    }

    let aligned_start = align_up(start, alignment);
    if aligned_start > end {
        return;
    }

    free.push(FreeSpaceSegment::new(
        aligned_start,
        end.saturating_sub(aligned_start).saturating_add(1),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DetectedTableLabel, DiskGuard, DiskKind, PendingState, ScanStatus};

    fn disk_with_sectors(sectors: u64) -> DiskDevice {
        DiskDevice {
            name: "sdb".to_string(),
            dev_path: "/dev/sdb".to_string(),
            size_bytes: sectors.saturating_mul(512),
            kind: DiskKind::Hdd,
            serial: "test".to_string(),
            guard: DiskGuard::None,
            model: "test".to_string(),
            table_label: DetectedTableLabel::Gpt,
            scan_status: ScanStatus::sfdisk(),
        }
    }

    fn part(start_sector: u64, size_sectors: u64) -> DraftPartition {
        DraftPartition {
            display_name: "part".to_string(),
            dev_path: Some("/dev/sdb1".to_string()),
            start_sector,
            size_bytes: size_sectors.saturating_mul(512),
            partition_type: "Linux filesystem".to_string(),
            partition_type_raw: "0FC63DAF-8483-4772-8E79-3D69D8477DE4".to_string(),
            fs_type: "Linux filesystem".to_string(),
            uuid: String::new(),
            part_uuid: String::new(),
            fs_label: String::new(),
            part_name: String::new(),
            mount_points: Vec::new(),
            is_swap: false,
            pending: PendingState::Existing,
            original_start_sector: Some(start_sector),
            original_size_bytes: Some(size_sectors.saturating_mul(512)),
            original_partition_type_raw: Some("0FC63DAF-8483-4772-8E79-3D69D8477DE4".to_string()),
            original_part_name: Some(String::new()),
        }
    }

    #[test]
    fn empty_disk_has_one_free_space_segment() {
        let disk = disk_with_sectors(100_000);
        let free = calculate_free_space(&disk, PartitionTableType::Gpt, &[], 2048);

        assert_eq!(free, vec![FreeSpaceSegment::new(2048, 97_919)]);
    }

    #[test]
    fn detects_middle_gap() {
        let disk = disk_with_sectors(100_000);
        let partitions = vec![part(2048, 10_000), part(30_720, 10_000)];
        let free = calculate_free_space(&disk, PartitionTableType::Gpt, &partitions, 2048);

        assert_eq!(free[0], FreeSpaceSegment::new(12_288, 18_432));
        assert_eq!(free[1], FreeSpaceSegment::new(40_960, 59_007));
    }

    #[test]
    fn adjacent_partitions_do_not_create_gap() {
        let disk = disk_with_sectors(50_000);
        let partitions = vec![part(2048, 2048), part(4096, 2048)];
        let free = calculate_free_space(&disk, PartitionTableType::Gpt, &partitions, 2048);

        assert_eq!(free, vec![FreeSpaceSegment::new(6144, 43_823)]);
    }

    #[test]
    fn alignment_drops_tiny_gap() {
        let disk = disk_with_sectors(20_000);
        let partitions = vec![part(2048, 2049), part(5001, 2048)];
        let free = calculate_free_space(&disk, PartitionTableType::Gpt, &partitions, 2048);

        assert_eq!(free, vec![FreeSpaceSegment::new(8192, 11_775)]);
    }

    #[test]
    fn deleted_partitions_are_treated_as_free() {
        let disk = disk_with_sectors(20_000);
        let mut deleted = part(2048, 2048);
        deleted.pending = PendingState::Deleted;
        let free = calculate_free_space(&disk, PartitionTableType::Gpt, &[deleted], 2048);

        assert_eq!(free, vec![FreeSpaceSegment::new(2048, 17_919)]);
    }
}
