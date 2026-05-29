#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FreeSpaceSegment {
    pub(crate) start_sector: u64,
    pub(crate) size_sectors: u64,
}

impl FreeSpaceSegment {
    pub(crate) fn new(start_sector: u64, size_sectors: u64) -> Self {
        Self {
            start_sector,
            size_sectors,
        }
    }

    pub(crate) fn size_bytes(self) -> u64 {
        self.size_sectors.saturating_mul(512)
    }
}
