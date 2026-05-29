#[derive(Clone, Debug)]
pub(crate) struct PartitionInfo {
    pub(crate) dev_path: String,
    pub(crate) start_sector: u64,
    pub(crate) size_bytes: u64,
    pub(crate) fs_type: String,
    pub(crate) partition_type: String,
    pub(crate) partition_type_raw: String,
    pub(crate) uuid: String,
    pub(crate) part_uuid: String,
    pub(crate) label: String,
    pub(crate) part_name: String,
    pub(crate) mount_points: Vec<String>,
    pub(crate) is_swap: bool,
}
