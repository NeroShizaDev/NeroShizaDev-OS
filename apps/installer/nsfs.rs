// ============================================================
// NSFS v1 — NeroShiza Storage Format
// ============================================================
// Первая версия дискового формата не пытается быть POSIX-ФС.
// Это карта областей диска + плоский индекс объектов с path-like
// именами вида `apps/demo.nhs` или `cfg/locale`.
//
// Слои:
//   1. Superblock  — сигнатура, версия, общая геометрия.
//   2. Regions     — последовательные области INDEX/JOURNAL/NHS/USER/SCRATCH.
//   3. Flat index  — фиксированные записи объектов без дерева inode.
//   4. Allocator   — монотонное выделение блоков внутри region.
// ============================================================

use core::cmp;

pub const NSFS_MAGIC: [u8; 4] = *b"NSFS";
pub const NSFS_VERSION: u16 = 1;
pub const NSFS_NAME_CAPACITY: usize = 64;
pub const NSFS_OWNER_CAPACITY: usize = 24;
pub const NSFS_REGION_COUNT: usize = 5;
pub const NSFS_BLOCK_SIZE_512: u32 = 512;
pub const NSFS_BLOCK_SIZE_4K: u32 = 4096;
pub const NSFS_SUPERBLOCK_SERIALIZED_LEN: usize = 126;
pub const NSFS_OBJECT_ENTRY_SERIALIZED_LEN: usize = 138;

const SUPERBLOCK_BLOCKS: u64 = 1;
const MIN_INDEX_BLOCKS: u64 = 32;
const MIN_JOURNAL_BLOCKS: u64 = 16;
const MIN_SCRATCH_BLOCKS: u64 = 16;
const MIN_DATA_BLOCKS: u64 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NsfsRegionKind {
    Index = 1,
    Journal = 2,
    NhsStore = 3,
    UserStore = 4,
    Scratch = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NsfsObjectKind {
    Empty = 0,
    KernelAsset = 1,
    NhsPackage = 2,
    Config = 3,
    Save = 4,
    Log = 5,
    Temp = 6,
    DirectoryMarker = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NsfsError {
    DiskTooSmall,
    BadBlockSize,
    RegionOverflow,
    BlockOutOfRange,
    BufferTooSmall,
    InvalidPath,
    InvalidSuperblock,
    InvalidRegion,
    InvalidObjectKind,
    NameTooLong,
    TableFull,
    DuplicatePath,
    EntryNotFound,
    RegionExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct NsfsRegionDescriptor {
    pub kind: NsfsRegionKind,
    pub start_block: u64,
    pub block_count: u64,
}

impl NsfsRegionDescriptor {
    pub const fn end_block(self) -> u64 {
        self.start_block + self.block_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct NsfsLayout {
    pub block_size: u32,
    pub total_blocks: u64,
    pub superblock_start: u64,
    pub regions: [NsfsRegionDescriptor; NSFS_REGION_COUNT],
}

impl NsfsLayout {
    pub fn plan(total_blocks: u64, block_size: u32) -> Result<Self, NsfsError> {
        if block_size != NSFS_BLOCK_SIZE_512 && block_size != NSFS_BLOCK_SIZE_4K {
            return Err(NsfsError::BadBlockSize);
        }

        let reserved = SUPERBLOCK_BLOCKS;
        let min_total = reserved
            + MIN_INDEX_BLOCKS
            + MIN_JOURNAL_BLOCKS
            + MIN_SCRATCH_BLOCKS
            + (MIN_DATA_BLOCKS * 2);
        if total_blocks < min_total {
            return Err(NsfsError::DiskTooSmall);
        }

        let index_blocks = cmp::max(MIN_INDEX_BLOCKS, total_blocks / 64);
        let journal_blocks = cmp::max(MIN_JOURNAL_BLOCKS, total_blocks / 128);

        let meta_blocks = reserved + index_blocks + journal_blocks;
        if meta_blocks >= total_blocks {
            return Err(NsfsError::RegionOverflow);
        }

        let data_budget = total_blocks - meta_blocks;
        let mut nhs_blocks = (data_budget * 45) / 100;
        let mut user_blocks = (data_budget * 35) / 100;

        if nhs_blocks < MIN_DATA_BLOCKS {
            nhs_blocks = MIN_DATA_BLOCKS;
        }
        if user_blocks < MIN_DATA_BLOCKS {
            user_blocks = MIN_DATA_BLOCKS;
        }

        let used_after_floor = nhs_blocks + user_blocks;
        if used_after_floor + MIN_SCRATCH_BLOCKS > data_budget {
            return Err(NsfsError::DiskTooSmall);
        }

        let scratch_blocks = data_budget - used_after_floor;
        if scratch_blocks < MIN_SCRATCH_BLOCKS {
            return Err(NsfsError::DiskTooSmall);
        }

        let mut cursor = SUPERBLOCK_BLOCKS;
        let index = NsfsRegionDescriptor {
            kind: NsfsRegionKind::Index,
            start_block: cursor,
            block_count: index_blocks,
        };
        cursor = index.end_block();

        let journal = NsfsRegionDescriptor {
            kind: NsfsRegionKind::Journal,
            start_block: cursor,
            block_count: journal_blocks,
        };
        cursor = journal.end_block();

        let nhs = NsfsRegionDescriptor {
            kind: NsfsRegionKind::NhsStore,
            start_block: cursor,
            block_count: nhs_blocks,
        };
        cursor = nhs.end_block();

        let user = NsfsRegionDescriptor {
            kind: NsfsRegionKind::UserStore,
            start_block: cursor,
            block_count: user_blocks,
        };
        cursor = user.end_block();

        let scratch = NsfsRegionDescriptor {
            kind: NsfsRegionKind::Scratch,
            start_block: cursor,
            block_count: scratch_blocks,
        };
        cursor = scratch.end_block();

        if cursor > total_blocks {
            return Err(NsfsError::RegionOverflow);
        }

        Ok(Self {
            block_size,
            total_blocks,
            superblock_start: 0,
            regions: [index, journal, nhs, user, scratch],
        })
    }

    pub fn region(&self, kind: NsfsRegionKind) -> Option<&NsfsRegionDescriptor> {
        self.regions.iter().find(|region| region.kind == kind)
    }

    pub fn to_superblock(&self) -> NsfsSuperblock {
        NsfsSuperblock {
            magic: NSFS_MAGIC,
            version: NSFS_VERSION,
            block_size: self.block_size,
            total_blocks: self.total_blocks,
            superblock_start: self.superblock_start,
            regions: self.regions,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct NsfsSuperblock {
    pub magic: [u8; 4],
    pub version: u16,
    pub block_size: u32,
    pub total_blocks: u64,
    pub superblock_start: u64,
    pub regions: [NsfsRegionDescriptor; NSFS_REGION_COUNT],
}

impl NsfsSuperblock {
    pub fn validate(&self) -> Result<(), NsfsError> {
        if self.magic != NSFS_MAGIC || self.version != NSFS_VERSION {
            return Err(NsfsError::InvalidSuperblock);
        }
        if self.block_size != NSFS_BLOCK_SIZE_512 && self.block_size != NSFS_BLOCK_SIZE_4K {
            return Err(NsfsError::BadBlockSize);
        }
        Ok(())
    }

    pub fn serialize_into(&self, out: &mut [u8]) -> Result<usize, NsfsError> {
        if out.len() < NSFS_SUPERBLOCK_SERIALIZED_LEN {
            return Err(NsfsError::BufferTooSmall);
        }

        out[..NSFS_SUPERBLOCK_SERIALIZED_LEN].fill(0);
        out[0..4].copy_from_slice(&self.magic);
        write_u16(&mut out[4..6], self.version);
        write_u32(&mut out[6..10], self.block_size);
        write_u64(&mut out[10..18], self.total_blocks);
        write_u64(&mut out[18..26], self.superblock_start);

        let mut cursor = 26;
        for region in &self.regions {
            write_u32(&mut out[cursor..cursor + 4], region.kind as u32);
            cursor += 4;
            write_u64(&mut out[cursor..cursor + 8], region.start_block);
            cursor += 8;
            write_u64(&mut out[cursor..cursor + 8], region.block_count);
            cursor += 8;
        }

        Ok(NSFS_SUPERBLOCK_SERIALIZED_LEN)
    }

    pub fn deserialize_from(input: &[u8]) -> Result<Self, NsfsError> {
        if input.len() < NSFS_SUPERBLOCK_SERIALIZED_LEN {
            return Err(NsfsError::BufferTooSmall);
        }

        let mut regions = [NsfsRegionDescriptor {
            kind: NsfsRegionKind::Index,
            start_block: 0,
            block_count: 0,
        }; NSFS_REGION_COUNT];

        let mut cursor = 26;
        for region in &mut regions {
            let kind = NsfsRegionKind::from_u32(read_u32(&input[cursor..cursor + 4]))?;
            cursor += 4;
            let start_block = read_u64(&input[cursor..cursor + 8]);
            cursor += 8;
            let block_count = read_u64(&input[cursor..cursor + 8]);
            cursor += 8;
            *region = NsfsRegionDescriptor {
                kind,
                start_block,
                block_count,
            };
        }

        let superblock = Self {
            magic: [input[0], input[1], input[2], input[3]],
            version: read_u16(&input[4..6]),
            block_size: read_u32(&input[6..10]),
            total_blocks: read_u64(&input[10..18]),
            superblock_start: read_u64(&input[18..26]),
            regions,
        };
        superblock.validate()?;
        Ok(superblock)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct NsfsObjectEntry {
    pub used: bool,
    pub kind: NsfsObjectKind,
    pub flags: u16,
    pub name_len: u8,
    pub name: [u8; NSFS_NAME_CAPACITY],
    pub owner_len: u8,
    pub owner: [u8; NSFS_OWNER_CAPACITY],
    pub start_block: u64,
    pub block_count: u64,
    pub size_bytes: u64,
    pub crc32: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

impl NsfsObjectEntry {
    pub const fn empty() -> Self {
        Self {
            used: false,
            kind: NsfsObjectKind::Empty,
            flags: 0,
            name_len: 0,
            name: [0; NSFS_NAME_CAPACITY],
            owner_len: 0,
            owner: [0; NSFS_OWNER_CAPACITY],
            start_block: 0,
            block_count: 0,
            size_bytes: 0,
            crc32: 0,
            created_at: 0,
            updated_at: 0,
        }
    }

    pub fn path(&self) -> &str {
        let len = self.name_len as usize;
        core::str::from_utf8(&self.name[..len]).unwrap_or("?")
    }

    pub fn owner_slug(&self) -> &str {
        let len = self.owner_len as usize;
        core::str::from_utf8(&self.owner[..len]).unwrap_or("")
    }

    pub fn serialize_into(&self, out: &mut [u8]) -> Result<usize, NsfsError> {
        if out.len() < NSFS_OBJECT_ENTRY_SERIALIZED_LEN {
            return Err(NsfsError::BufferTooSmall);
        }

        out[..NSFS_OBJECT_ENTRY_SERIALIZED_LEN].fill(0);
        out[0] = u8::from(self.used);
        out[1] = self.kind as u8;
        write_u16(&mut out[2..4], self.flags);
        out[4] = self.name_len;
        out[5..5 + NSFS_NAME_CAPACITY].copy_from_slice(&self.name);
        out[69] = self.owner_len;
        out[70..70 + NSFS_OWNER_CAPACITY].copy_from_slice(&self.owner);
        write_u64(&mut out[94..102], self.start_block);
        write_u64(&mut out[102..110], self.block_count);
        write_u64(&mut out[110..118], self.size_bytes);
        write_u32(&mut out[118..122], self.crc32);
        write_u64(&mut out[122..130], self.created_at);
        write_u64(&mut out[130..138], self.updated_at);
        Ok(NSFS_OBJECT_ENTRY_SERIALIZED_LEN)
    }

    pub fn deserialize_from(input: &[u8]) -> Result<Self, NsfsError> {
        if input.len() < NSFS_OBJECT_ENTRY_SERIALIZED_LEN {
            return Err(NsfsError::BufferTooSmall);
        }

        let name_len = input[4] as usize;
        if name_len > NSFS_NAME_CAPACITY {
            return Err(NsfsError::InvalidPath);
        }
        let owner_len = input[69] as usize;
        if owner_len > NSFS_OWNER_CAPACITY {
            return Err(NsfsError::InvalidPath);
        }

        let mut name = [0u8; NSFS_NAME_CAPACITY];
        name.copy_from_slice(&input[5..5 + NSFS_NAME_CAPACITY]);
        let mut owner = [0u8; NSFS_OWNER_CAPACITY];
        owner.copy_from_slice(&input[70..70 + NSFS_OWNER_CAPACITY]);

        Ok(Self {
            used: input[0] != 0,
            kind: NsfsObjectKind::from_u8(input[1])?,
            flags: read_u16(&input[2..4]),
            name_len: input[4],
            name,
            owner_len: input[69],
            owner,
            start_block: read_u64(&input[94..102]),
            block_count: read_u64(&input[102..110]),
            size_bytes: read_u64(&input[110..118]),
            crc32: read_u32(&input[118..122]),
            created_at: read_u64(&input[122..130]),
            updated_at: read_u64(&input[130..138]),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NsfsAllocation {
    pub start_block: u64,
    pub block_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NsfsRegionCursor {
    pub region: NsfsRegionDescriptor,
    pub next_free_block: u64,
}

impl NsfsRegionCursor {
    pub const fn new(region: NsfsRegionDescriptor) -> Self {
        Self {
            region,
            next_free_block: region.start_block,
        }
    }

    pub fn allocate_bytes(
        &mut self,
        byte_len: usize,
        block_size: u32,
    ) -> Result<NsfsAllocation, NsfsError> {
        let blocks = bytes_to_blocks(byte_len, block_size);
        let end = self
            .next_free_block
            .checked_add(blocks)
            .ok_or(NsfsError::RegionOverflow)?;
        if end > self.region.end_block() {
            return Err(NsfsError::RegionExhausted);
        }

        let allocation = NsfsAllocation {
            start_block: self.next_free_block,
            block_count: blocks,
        };
        self.next_free_block = end;
        Ok(allocation)
    }
}

pub struct NsfsIndex<const N: usize> {
    pub entries: [NsfsObjectEntry; N],
}

impl<const N: usize> NsfsIndex<N> {
    pub const fn new() -> Self {
        Self {
            entries: [NsfsObjectEntry::empty(); N],
        }
    }

    pub fn find(&self, path: &str) -> Result<Option<usize>, NsfsError> {
        let mut normalized = [0u8; NSFS_NAME_CAPACITY];
        let len = normalize_path(path, &mut normalized)?;

        for index in 0..N {
            let entry = &self.entries[index];
            if !entry.used {
                continue;
            }
            if entry.name_len as usize == len && entry.name[..len] == normalized[..len] {
                return Ok(Some(index));
            }
        }

        Ok(None)
    }

    pub fn insert(
        &mut self,
        path: &str,
        kind: NsfsObjectKind,
        flags: u16,
        allocation: NsfsAllocation,
        size_bytes: u64,
        crc32: u32,
        timestamp: u64,
    ) -> Result<usize, NsfsError> {
        self.insert_owned(
            path, "", kind, flags, allocation, size_bytes, crc32, timestamp,
        )
    }

    pub fn insert_owned(
        &mut self,
        path: &str,
        owner_slug: &str,
        kind: NsfsObjectKind,
        flags: u16,
        allocation: NsfsAllocation,
        size_bytes: u64,
        crc32: u32,
        timestamp: u64,
    ) -> Result<usize, NsfsError> {
        let mut normalized = [0u8; NSFS_NAME_CAPACITY];
        let len = normalize_path(path, &mut normalized)?;
        let mut owner = [0u8; NSFS_OWNER_CAPACITY];
        let owner_len = normalize_owner_slug(owner_slug, &mut owner)?;

        if self.find(path)?.is_some() {
            return Err(NsfsError::DuplicatePath);
        }

        for index in 0..N {
            if self.entries[index].used {
                continue;
            }

            let mut name = [0u8; NSFS_NAME_CAPACITY];
            name[..len].copy_from_slice(&normalized[..len]);
            self.entries[index] = NsfsObjectEntry {
                used: true,
                kind,
                flags,
                name_len: len as u8,
                name,
                owner_len: owner_len as u8,
                owner,
                start_block: allocation.start_block,
                block_count: allocation.block_count,
                size_bytes,
                crc32,
                created_at: timestamp,
                updated_at: timestamp,
            };
            return Ok(index);
        }

        Err(NsfsError::TableFull)
    }

    pub fn remove(&mut self, path: &str) -> Result<(), NsfsError> {
        let Some(index) = self.find(path)? else {
            return Err(NsfsError::EntryNotFound);
        };
        self.entries[index] = NsfsObjectEntry::empty();
        Ok(())
    }

    pub fn count_prefix(&self, prefix: &str) -> Result<usize, NsfsError> {
        let mut normalized = [0u8; NSFS_NAME_CAPACITY];
        let len = normalize_path(prefix, &mut normalized)?;
        let mut count = 0;
        for entry in &self.entries {
            if !entry.used {
                continue;
            }
            let entry_len = entry.name_len as usize;
            if entry_len >= len && entry.name[..len] == normalized[..len] {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn get(&self, path: &str) -> Result<Option<&NsfsObjectEntry>, NsfsError> {
        match self.find(path)? {
            Some(index) => Ok(Some(&self.entries[index])),
            None => Ok(None),
        }
    }

    pub fn used_count(&self) -> usize {
        self.entries.iter().filter(|entry| entry.used).count()
    }

    pub fn for_each(&self, mut f: impl FnMut(&NsfsObjectEntry)) {
        for entry in &self.entries {
            if entry.used {
                f(entry);
            }
        }
    }

    pub fn serialize_into(&self, out: &mut [u8]) -> Result<usize, NsfsError> {
        let required = N * NSFS_OBJECT_ENTRY_SERIALIZED_LEN;
        if out.len() < required {
            return Err(NsfsError::BufferTooSmall);
        }

        out[..required].fill(0);
        for index in 0..N {
            let start = index * NSFS_OBJECT_ENTRY_SERIALIZED_LEN;
            let end = start + NSFS_OBJECT_ENTRY_SERIALIZED_LEN;
            self.entries[index].serialize_into(&mut out[start..end])?;
        }
        Ok(required)
    }

    pub fn deserialize_from(input: &[u8]) -> Result<Self, NsfsError> {
        let required = N * NSFS_OBJECT_ENTRY_SERIALIZED_LEN;
        if input.len() < required {
            return Err(NsfsError::BufferTooSmall);
        }

        let mut entries = [NsfsObjectEntry::empty(); N];
        for (index, entry) in entries.iter_mut().enumerate() {
            let start = index * NSFS_OBJECT_ENTRY_SERIALIZED_LEN;
            let end = start + NSFS_OBJECT_ENTRY_SERIALIZED_LEN;
            *entry = NsfsObjectEntry::deserialize_from(&input[start..end])?;
        }
        Ok(Self { entries })
    }
}

pub struct NsfsRamDisk<const BYTES: usize> {
    bytes: [u8; BYTES],
}

impl<const BYTES: usize> NsfsRamDisk<BYTES> {
    pub const fn new() -> Self {
        Self { bytes: [0; BYTES] }
    }

    pub const fn capacity_bytes(&self) -> usize {
        BYTES
    }

    pub fn total_blocks(&self, block_size: u32) -> Result<u64, NsfsError> {
        if block_size == 0 {
            return Err(NsfsError::BadBlockSize);
        }
        Ok((BYTES / block_size as usize) as u64)
    }

    pub fn clear(&mut self) {
        self.bytes.fill(0);
    }

    pub fn read_blocks(
        &self,
        start_block: u64,
        block_count: u64,
        block_size: u32,
        out: &mut [u8],
    ) -> Result<(), NsfsError> {
        let byte_len = blocks_to_bytes(block_count, block_size)?;
        if out.len() < byte_len {
            return Err(NsfsError::BufferTooSmall);
        }

        let start = block_to_offset(start_block, block_size)?;
        let end = start
            .checked_add(byte_len)
            .ok_or(NsfsError::BlockOutOfRange)?;
        if end > BYTES {
            return Err(NsfsError::BlockOutOfRange);
        }

        out[..byte_len].copy_from_slice(&self.bytes[start..end]);
        Ok(())
    }

    pub fn write_blocks(
        &mut self,
        start_block: u64,
        block_size: u32,
        data: &[u8],
    ) -> Result<(), NsfsError> {
        let start = block_to_offset(start_block, block_size)?;
        let end = start
            .checked_add(data.len())
            .ok_or(NsfsError::BlockOutOfRange)?;
        if end > BYTES {
            return Err(NsfsError::BlockOutOfRange);
        }

        self.bytes[start..end].copy_from_slice(data);
        Ok(())
    }

    pub fn zero_blocks(
        &mut self,
        start_block: u64,
        block_count: u64,
        block_size: u32,
    ) -> Result<(), NsfsError> {
        let start = block_to_offset(start_block, block_size)?;
        let byte_len = blocks_to_bytes(block_count, block_size)?;
        let end = start
            .checked_add(byte_len)
            .ok_or(NsfsError::BlockOutOfRange)?;
        if end > BYTES {
            return Err(NsfsError::BlockOutOfRange);
        }

        self.bytes[start..end].fill(0);
        Ok(())
    }

    pub fn write_bytes(&mut self, byte_offset: usize, data: &[u8]) -> Result<(), NsfsError> {
        let end = byte_offset
            .checked_add(data.len())
            .ok_or(NsfsError::BlockOutOfRange)?;
        if end > BYTES {
            return Err(NsfsError::BlockOutOfRange);
        }

        self.bytes[byte_offset..end].copy_from_slice(data);
        Ok(())
    }

    pub fn zero_range(&mut self, byte_offset: usize, byte_len: usize) -> Result<(), NsfsError> {
        let end = byte_offset
            .checked_add(byte_len)
            .ok_or(NsfsError::BlockOutOfRange)?;
        if end > BYTES {
            return Err(NsfsError::BlockOutOfRange);
        }

        self.bytes[byte_offset..end].fill(0);
        Ok(())
    }

    pub unsafe fn slice(&self, byte_offset: usize, byte_len: usize) -> Option<&[u8]> {
        let end = byte_offset.checked_add(byte_len)?;
        if end > BYTES {
            return None;
        }
        Some(&self.bytes[byte_offset..end])
    }
}

pub struct NsfsVolume<const N: usize> {
    pub layout: NsfsLayout,
    pub superblock: NsfsSuperblock,
    pub index: NsfsIndex<N>,
    pub cursors: [NsfsRegionCursor; NSFS_REGION_COUNT],
}

impl<const N: usize> NsfsVolume<N> {
    pub fn format<const BYTES: usize>(
        disk: &mut NsfsRamDisk<BYTES>,
        total_blocks: u64,
        block_size: u32,
    ) -> Result<Self, NsfsError> {
        let layout = NsfsLayout::plan(total_blocks, block_size)?;
        let superblock = layout.to_superblock();
        let index = NsfsIndex::new();
        let cursors = layout.regions.map(NsfsRegionCursor::new);

        let volume = Self {
            layout,
            superblock,
            index,
            cursors,
        };

        disk.clear();
        volume.flush_superblock(disk)?;
        volume.flush_index(disk)?;
        Ok(volume)
    }

    pub fn mount<const BYTES: usize>(disk: &NsfsRamDisk<BYTES>) -> Result<Self, NsfsError> {
        let mut superblock_bytes = [0u8; NSFS_BLOCK_SIZE_512 as usize];
        disk.read_blocks(0, 1, NSFS_BLOCK_SIZE_512, &mut superblock_bytes)?;
        let superblock = NsfsSuperblock::deserialize_from(&superblock_bytes)?;
        let layout = NsfsLayout {
            block_size: superblock.block_size,
            total_blocks: superblock.total_blocks,
            superblock_start: superblock.superblock_start,
            regions: superblock.regions,
        };
        let index_region = layout
            .region(NsfsRegionKind::Index)
            .ok_or(NsfsError::InvalidRegion)?;
        let index_bytes_len = blocks_to_bytes(index_region.block_count, layout.block_size)?;
        let required = N * NSFS_OBJECT_ENTRY_SERIALIZED_LEN;
        if required > index_bytes_len {
            return Err(NsfsError::BufferTooSmall);
        }
        let mut index = NsfsIndex::new();
        for index_pos in 0..N {
            let start = block_to_offset(index_region.start_block, layout.block_size)?
                + index_pos * NSFS_OBJECT_ENTRY_SERIALIZED_LEN;
            let end = start + NSFS_OBJECT_ENTRY_SERIALIZED_LEN;
            if end > BYTES {
                return Err(NsfsError::BlockOutOfRange);
            }
            index.entries[index_pos] = NsfsObjectEntry::deserialize_from(&disk.bytes[start..end])?;
        }
        let mut volume = Self {
            layout,
            superblock,
            index,
            cursors: layout.regions.map(NsfsRegionCursor::new),
        };
        volume.rebuild_cursors_from_index()?;
        Ok(volume)
    }

    pub fn add_object<const BYTES: usize>(
        &mut self,
        disk: &mut NsfsRamDisk<BYTES>,
        path: &str,
        kind: NsfsObjectKind,
        region: NsfsRegionKind,
        flags: u16,
        payload: &[u8],
        crc32: u32,
        timestamp: u64,
    ) -> Result<usize, NsfsError> {
        self.add_object_owned(
            disk, path, "", kind, region, flags, payload, crc32, timestamp,
        )
    }

    pub fn add_object_owned<const BYTES: usize>(
        &mut self,
        disk: &mut NsfsRamDisk<BYTES>,
        path: &str,
        owner_slug: &str,
        kind: NsfsObjectKind,
        region: NsfsRegionKind,
        flags: u16,
        payload: &[u8],
        crc32: u32,
        timestamp: u64,
    ) -> Result<usize, NsfsError> {
        let cursor_index = self.region_cursor_index(region)?;
        let allocation =
            self.cursors[cursor_index].allocate_bytes(payload.len(), self.layout.block_size)?;
        let byte_offset = block_to_offset(allocation.start_block, self.layout.block_size)?;
        let allocated_bytes = blocks_to_bytes(allocation.block_count, self.layout.block_size)?;
        disk.zero_range(byte_offset, allocated_bytes)?;
        disk.write_bytes(byte_offset, payload)?;

        let inserted = self.index.insert_owned(
            path,
            owner_slug,
            kind,
            flags,
            allocation,
            payload.len() as u64,
            crc32,
            timestamp,
        )?;
        self.flush_index(disk)?;
        Ok(inserted)
    }

    pub fn remove_object<const BYTES: usize>(
        &mut self,
        disk: &mut NsfsRamDisk<BYTES>,
        path: &str,
    ) -> Result<(), NsfsError> {
        let Some(entry) = self.index.get(path)?.copied() else {
            return Err(NsfsError::EntryNotFound);
        };

        let byte_offset = block_to_offset(entry.start_block, self.layout.block_size)?;
        let byte_len = blocks_to_bytes(entry.block_count, self.layout.block_size)?;
        disk.zero_range(byte_offset, byte_len)?;
        self.index.remove(path)?;
        self.flush_index(disk)?;
        Ok(())
    }

    pub unsafe fn read_object<'a, const BYTES: usize>(
        &self,
        disk: &'a NsfsRamDisk<BYTES>,
        path: &str,
    ) -> Result<Option<&'a [u8]>, NsfsError> {
        let Some(entry) = self.index.get(path)? else {
            return Ok(None);
        };
        let byte_offset = block_to_offset(entry.start_block, self.layout.block_size)?;
        Ok(disk.slice(byte_offset, entry.size_bytes as usize))
    }

    pub fn flush_superblock<const BYTES: usize>(
        &self,
        disk: &mut NsfsRamDisk<BYTES>,
    ) -> Result<(), NsfsError> {
        let block_len = self.layout.block_size as usize;
        let mut block = [0u8; NSFS_BLOCK_SIZE_512 as usize];
        if block_len > block.len() {
            return Err(NsfsError::BadBlockSize);
        }
        self.superblock
            .serialize_into(&mut block[..NSFS_SUPERBLOCK_SERIALIZED_LEN])?;
        disk.write_blocks(
            self.superblock.superblock_start,
            self.layout.block_size,
            &block[..block_len],
        )
    }

    pub fn flush_index<const BYTES: usize>(
        &self,
        disk: &mut NsfsRamDisk<BYTES>,
    ) -> Result<(), NsfsError> {
        let region = self
            .layout
            .region(NsfsRegionKind::Index)
            .ok_or(NsfsError::InvalidRegion)?;
        let byte_len = blocks_to_bytes(region.block_count, self.layout.block_size)?;
        let required = N * NSFS_OBJECT_ENTRY_SERIALIZED_LEN;
        if required > byte_len {
            return Err(NsfsError::BufferTooSmall);
        }
        disk.zero_blocks(
            region.start_block,
            region.block_count,
            self.layout.block_size,
        )?;
        let base = block_to_offset(region.start_block, self.layout.block_size)?;
        for index in 0..N {
            let start = base + index * NSFS_OBJECT_ENTRY_SERIALIZED_LEN;
            let end = start + NSFS_OBJECT_ENTRY_SERIALIZED_LEN;
            if end > BYTES {
                return Err(NsfsError::BlockOutOfRange);
            }
            self.index.entries[index].serialize_into(&mut disk.bytes[start..end])?;
        }
        Ok(())
    }

    pub fn object_count(&self) -> usize {
        self.index.used_count()
    }

    pub fn used_bytes(&self) -> u64 {
        let mut used = 0;
        self.index.for_each(|entry| used += entry.size_bytes);
        used
    }

    fn rebuild_cursors_from_index(&mut self) -> Result<(), NsfsError> {
        self.cursors = self.layout.regions.map(NsfsRegionCursor::new);
        for entry in &self.index.entries {
            if !entry.used {
                continue;
            }
            if let Some(cursor_index) = self.region_cursor_index_for_block(entry.start_block) {
                let cursor = &mut self.cursors[cursor_index];
                let end_block = entry.start_block + entry.block_count;
                if end_block > cursor.next_free_block {
                    cursor.next_free_block = end_block;
                }
            }
        }
        Ok(())
    }

    fn region_cursor_index(&self, kind: NsfsRegionKind) -> Result<usize, NsfsError> {
        self.layout
            .regions
            .iter()
            .position(|region| region.kind == kind)
            .ok_or(NsfsError::InvalidRegion)
    }

    fn region_cursor_index_for_block(&self, block: u64) -> Option<usize> {
        self.layout
            .regions
            .iter()
            .position(|region| block >= region.start_block && block < region.end_block())
    }
}

pub fn bytes_to_blocks(byte_len: usize, block_size: u32) -> u64 {
    if byte_len == 0 {
        return 0;
    }
    let block_size = block_size as usize;
    byte_len.div_ceil(block_size) as u64
}

pub fn blocks_to_bytes(block_count: u64, block_size: u32) -> Result<usize, NsfsError> {
    let bytes = block_count
        .checked_mul(block_size as u64)
        .ok_or(NsfsError::BlockOutOfRange)?;
    usize::try_from(bytes).map_err(|_| NsfsError::BlockOutOfRange)
}

pub fn block_to_offset(block: u64, block_size: u32) -> Result<usize, NsfsError> {
    let offset = block
        .checked_mul(block_size as u64)
        .ok_or(NsfsError::BlockOutOfRange)?;
    usize::try_from(offset).map_err(|_| NsfsError::BlockOutOfRange)
}

pub fn normalize_path(path: &str, out: &mut [u8; NSFS_NAME_CAPACITY]) -> Result<usize, NsfsError> {
    let bytes = path.as_bytes();
    if bytes.is_empty() {
        return Err(NsfsError::InvalidPath);
    }

    let mut start = 0;
    while start < bytes.len() && bytes[start] == b'/' {
        start += 1;
    }

    let mut end = bytes.len();
    while end > start && bytes[end - 1] == b'/' {
        end -= 1;
    }

    if start == end {
        return Err(NsfsError::InvalidPath);
    }

    let mut out_len = 0usize;
    let mut last_was_slash = false;
    for &byte in &bytes[start..end] {
        if byte == 0 || byte == b'\\' || byte < 0x20 {
            return Err(NsfsError::InvalidPath);
        }
        if byte == b'/' {
            if last_was_slash {
                return Err(NsfsError::InvalidPath);
            }
            last_was_slash = true;
        } else {
            last_was_slash = false;
        }

        if out_len >= NSFS_NAME_CAPACITY {
            return Err(NsfsError::NameTooLong);
        }
        out[out_len] = byte;
        out_len += 1;
    }

    if out_len == 0 || last_was_slash {
        return Err(NsfsError::InvalidPath);
    }

    Ok(out_len)
}

pub fn normalize_owner_slug(
    owner: &str,
    out: &mut [u8; NSFS_OWNER_CAPACITY],
) -> Result<usize, NsfsError> {
    if owner.is_empty() {
        return Ok(0);
    }

    let bytes = owner.as_bytes();
    if bytes.len() > NSFS_OWNER_CAPACITY {
        return Err(NsfsError::NameTooLong);
    }

    for (index, &byte) in bytes.iter().enumerate() {
        let valid = byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_');
        if !valid {
            return Err(NsfsError::InvalidPath);
        }
        out[index] = byte;
    }

    Ok(bytes.len())
}

impl NsfsRegionKind {
    fn from_u32(value: u32) -> Result<Self, NsfsError> {
        match value {
            1 => Ok(Self::Index),
            2 => Ok(Self::Journal),
            3 => Ok(Self::NhsStore),
            4 => Ok(Self::UserStore),
            5 => Ok(Self::Scratch),
            _ => Err(NsfsError::InvalidRegion),
        }
    }
}

impl NsfsObjectKind {
    fn from_u8(value: u8) -> Result<Self, NsfsError> {
        match value {
            0 => Ok(Self::Empty),
            1 => Ok(Self::KernelAsset),
            2 => Ok(Self::NhsPackage),
            3 => Ok(Self::Config),
            4 => Ok(Self::Save),
            5 => Ok(Self::Log),
            6 => Ok(Self::Temp),
            7 => Ok(Self::DirectoryMarker),
            _ => Err(NsfsError::InvalidObjectKind),
        }
    }
}

fn write_u16(out: &mut [u8], value: u16) {
    out.copy_from_slice(&value.to_le_bytes());
}

fn write_u32(out: &mut [u8], value: u32) {
    out.copy_from_slice(&value.to_le_bytes());
}

fn write_u64(out: &mut [u8], value: u64) {
    out.copy_from_slice(&value.to_le_bytes());
}

fn read_u16(input: &[u8]) -> u16 {
    u16::from_le_bytes([input[0], input[1]])
}

fn read_u32(input: &[u8]) -> u32 {
    u32::from_le_bytes([input[0], input[1], input[2], input[3]])
}

fn read_u64(input: &[u8]) -> u64 {
    u64::from_le_bytes([
        input[0], input[1], input[2], input[3], input[4], input[5], input[6], input[7],
    ])
}

#[cfg(all(test, target_os = "none"))]
#[path = "nsfs_tests.rs"]
mod tests;
