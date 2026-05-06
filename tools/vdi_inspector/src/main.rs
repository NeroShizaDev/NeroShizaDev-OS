use anyhow::{anyhow, bail, Context, Result};
use ext4_view::{Ext4, Ext4Read};
use gptman::GPT;
use mbrman::MBR;
use ntfs::indexes::NtfsFileNameIndex;
use ntfs::{Ntfs, NtfsFile, NtfsReadSeek};
use std::env;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom, Write};
use std::ops::RangeInclusive;
use std::path::Path;

const VDI_IMAGE_SIGNATURE: u32 = 0xbeda107f;
const VDI_IMAGE_TYPE_NORMAL: u32 = 1;
const VDI_IMAGE_TYPE_FIXED: u32 = 2;
const VDI_IMAGE_BLOCK_FREE: u32 = u32::MAX;
const VDI_IMAGE_BLOCK_ZERO: u32 = u32::MAX - 1;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage(&args[0]);
        bail!("not enough arguments")
    }

    let image_path = &args[1];
    let command = &args[2];

    match command.as_str() {
        "info" => cmd_info(image_path),
        "parts" => cmd_parts(image_path),
        "ls" => {
            let partition_index = parse_partition_index(args.get(3))?;
            let path = args.get(4).map(String::as_str).unwrap_or("/");
            cmd_ls(image_path, partition_index, path)
        }
        "cat" => {
            let partition_index = parse_partition_index(args.get(3))?;
            let path = args
                .get(4)
                .map(String::as_str)
                .ok_or_else(|| anyhow!("cat requires a path inside the partition"))?;
            cmd_cat(image_path, partition_index, path)
        }
        _ => {
            print_usage(&args[0]);
            bail!("unknown command: {command}")
        }
    }
}

fn print_usage(program: &str) {
    eprintln!("Usage:");
    eprintln!("  {program} <disk.vdi> info");
    eprintln!("  {program} <disk.vdi> parts");
    eprintln!("  {program} <disk.vdi> ls <partition-index> [path]");
    eprintln!("  {program} <disk.vdi> cat <partition-index> <path>");
}

fn parse_partition_index(value: Option<&String>) -> Result<u32> {
    let value = value.ok_or_else(|| anyhow!("missing partition index"))?;
    value
        .parse::<u32>()
        .with_context(|| format!("invalid partition index: {value}"))
}

#[derive(Debug, Clone, Copy)]
enum FsKind {
    Ntfs,
    Ext,
    Unknown,
}

impl FsKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ntfs => "ntfs",
            Self::Ext => "ext2/3/4",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
enum PartitionScheme {
    Gpt,
    Mbr,
}

impl PartitionScheme {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Gpt => "gpt",
            Self::Mbr => "mbr",
        }
    }
}

#[derive(Debug, Clone)]
struct PartitionRecord {
    scheme: PartitionScheme,
    index: u32,
    start_byte: u64,
    size_bytes: u64,
    label: String,
}

impl PartitionRecord {
    fn end_byte(&self) -> u64 {
        self.start_byte + self.size_bytes.saturating_sub(1)
    }
}

#[derive(Debug)]
struct VdiHeader {
    version: u32,
    image_type: u32,
    off_blocks: u32,
    off_data: u32,
    disk_size: u64,
    block_size: u32,
    block_extra_size: u32,
    blocks: u32,
    blocks_allocated: u32,
}

impl VdiHeader {
    fn image_type_name(&self) -> &'static str {
        match self.image_type {
            VDI_IMAGE_TYPE_NORMAL => "dynamic",
            VDI_IMAGE_TYPE_FIXED => "fixed",
            _ => "unsupported",
        }
    }

    fn total_block_data(&self) -> u64 {
        u64::from(self.block_size) + u64::from(self.block_extra_size)
    }

    fn first_block_data_offset(&self) -> u64 {
        u64::from(self.off_data) + u64::from(self.block_extra_size)
    }
}

struct VdiImage {
    file: File,
    header: VdiHeader,
    block_map: Vec<u32>,
    pos: u64,
}

impl VdiImage {
    fn open(path: &Path) -> Result<Self> {
        let mut file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;

        let mut preheader = [0u8; 72];
        file.read_exact(&mut preheader)
            .with_context(|| format!("failed to read VDI preheader from {}", path.display()))?;

        let signature = le_u32(&preheader[64..68]);
        if signature != VDI_IMAGE_SIGNATURE {
            bail!("{} is not a VDI image", path.display());
        }

        let version = le_u32(&preheader[68..72]);
        if (version >> 16) != 1 {
            bail!("unsupported VDI version 0x{version:08x}");
        }

        file.seek(SeekFrom::Start(72))?;
        let mut first_header = [0u8; 4];
        file.read_exact(&mut first_header)?;
        let header_size = le_u32(&first_header) as usize;
        if header_size < 320 {
            bail!("VDI header is too small: {header_size}");
        }

        file.seek(SeekFrom::Start(72))?;
        let mut header_bytes = vec![0u8; header_size];
        file.read_exact(&mut header_bytes)?;

        let header = VdiHeader {
            version,
            image_type: le_u32(&header_bytes[4..8]),
            off_blocks: le_u32(&header_bytes[268..272]),
            off_data: le_u32(&header_bytes[272..276]),
            disk_size: le_u64(&header_bytes[296..304]),
            block_size: le_u32(&header_bytes[304..308]),
            block_extra_size: le_u32(&header_bytes[308..312]),
            blocks: le_u32(&header_bytes[312..316]),
            blocks_allocated: le_u32(&header_bytes[316..320]),
        };

        if header.image_type != VDI_IMAGE_TYPE_NORMAL && header.image_type != VDI_IMAGE_TYPE_FIXED {
            bail!("unsupported VDI image type {}", header.image_type);
        }
        if header.block_size == 0 || !header.block_size.is_power_of_two() {
            bail!("invalid VDI block size {}", header.block_size);
        }

        let mut block_map = Vec::new();
        if header.image_type == VDI_IMAGE_TYPE_NORMAL {
            file.seek(SeekFrom::Start(u64::from(header.off_blocks)))?;
            block_map.resize(header.blocks as usize, 0);
            for entry in &mut block_map {
                let mut bytes = [0u8; 4];
                file.read_exact(&mut bytes)?;
                *entry = u32::from_le_bytes(bytes);
            }
        }

        Ok(Self {
            file,
            header,
            block_map,
            pos: 0,
        })
    }

    fn read_at(&mut self, start: u64, dst: &mut [u8]) -> Result<()> {
        if start.checked_add(dst.len() as u64).unwrap_or(u64::MAX) > self.header.disk_size {
            bail!("read past virtual disk end")
        }

        let mut done = 0usize;
        while done < dst.len() {
            let logical_offset = start + done as u64;
            let block_index = (logical_offset / u64::from(self.header.block_size)) as usize;
            let offset_in_block = (logical_offset % u64::from(self.header.block_size)) as usize;
            let block_left = self.header.block_size as usize - offset_in_block;
            let chunk_len = block_left.min(dst.len() - done);
            let chunk = &mut dst[done..done + chunk_len];

            match self.header.image_type {
                VDI_IMAGE_TYPE_FIXED => {
                    let file_offset = u64::from(self.header.off_data) + logical_offset;
                    self.file.seek(SeekFrom::Start(file_offset))?;
                    self.file.read_exact(chunk)?;
                }
                VDI_IMAGE_TYPE_NORMAL => {
                    let block_ptr = *self
                        .block_map
                        .get(block_index)
                        .ok_or_else(|| anyhow!("VDI block index out of range"))?;
                    if block_ptr == VDI_IMAGE_BLOCK_FREE || block_ptr == VDI_IMAGE_BLOCK_ZERO {
                        chunk.fill(0);
                    } else {
                        let file_offset = u64::from(block_ptr) * self.header.total_block_data()
                            + self.header.first_block_data_offset()
                            + offset_in_block as u64;
                        self.file.seek(SeekFrom::Start(file_offset))?;
                        self.file.read_exact(chunk)?;
                    }
                }
                _ => unreachable!(),
            }

            done += chunk_len;
        }

        Ok(())
    }
}

impl Read for VdiImage {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.header.disk_size {
            return Ok(0);
        }

        let max = usize::try_from((self.header.disk_size - self.pos).min(buf.len() as u64)).unwrap_or(buf.len());
        self.read_at(self.pos, &mut buf[..max])
            .map_err(io::Error::other)?;
        self.pos += max as u64;
        Ok(max)
    }
}

impl Seek for VdiImage {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(v) => v as i128,
            SeekFrom::Current(v) => self.pos as i128 + v as i128,
            SeekFrom::End(v) => self.header.disk_size as i128 + v as i128,
        };

        if new_pos < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "negative seek"));
        }

        self.pos = new_pos as u64;
        Ok(self.pos)
    }
}

struct PartitionReader {
    image: VdiImage,
    start: u64,
    len: u64,
    pos: u64,
}

impl PartitionReader {
    fn open(image_path: &Path, part: &PartitionRecord) -> Result<Self> {
        Ok(Self {
            image: VdiImage::open(image_path)?,
            start: part.start_byte,
            len: part.size_bytes,
            pos: 0,
        })
    }

    fn read_exact_at(&mut self, start: u64, dst: &mut [u8]) -> Result<()> {
        if start.checked_add(dst.len() as u64).unwrap_or(u64::MAX) > self.len {
            bail!("read past partition end")
        }
        self.image.read_at(self.start + start, dst)
    }
}

impl Read for PartitionReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.len {
            return Ok(0);
        }
        let max = usize::try_from((self.len - self.pos).min(buf.len() as u64)).unwrap_or(buf.len());
        self.read_exact_at(self.pos, &mut buf[..max])
            .map_err(io::Error::other)?;
        self.pos += max as u64;
        Ok(max)
    }
}

impl Seek for PartitionReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(v) => v as i128,
            SeekFrom::Current(v) => self.pos as i128 + v as i128,
            SeekFrom::End(v) => self.len as i128 + v as i128,
        };
        if new_pos < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "negative seek"));
        }
        self.pos = new_pos as u64;
        Ok(self.pos)
    }
}

impl Ext4Read for PartitionReader {
    fn read(&mut self, start_byte: u64, dst: &mut [u8]) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.read_exact_at(start_byte, dst)
            .map_err(|err| Box::new(io::Error::other(err.to_string())) as Box<dyn std::error::Error + Send + Sync + 'static>)
    }
}

fn cmd_info(image_path: &str) -> Result<()> {
    let image = VdiImage::open(Path::new(image_path))?;
    println!("file: {image_path}");
    println!("vdi_version: 0x{:08x}", image.header.version);
    println!("image_type: {}", image.header.image_type_name());
    println!("virtual_size: {} bytes", image.header.disk_size);
    println!("block_size: {} bytes", image.header.block_size);
    println!("block_extra: {} bytes", image.header.block_extra_size);
    println!("blocks: {}", image.header.blocks);
    println!("blocks_allocated: {}", image.header.blocks_allocated);
    println!("blocks_offset: {}", image.header.off_blocks);
    println!("data_offset: {}", image.header.off_data);
    Ok(())
}

fn cmd_parts(image_path: &str) -> Result<()> {
    let partitions = detect_partitions(image_path)?;
    if partitions.is_empty() {
        println!("no partitions found");
        return Ok(());
    }

    for part in partitions {
        let fs_kind = detect_partition_fs(image_path, &part).unwrap_or(FsKind::Unknown);
        println!(
            "#{:>2}  {:<3}  {:>12}..{:>12}  {:>12} bytes  {:<10}  {}",
            part.index,
            part.scheme.as_str(),
            part.start_byte,
            part.end_byte(),
            part.size_bytes,
            fs_kind.as_str(),
            part.label,
        );
    }
    Ok(())
}

fn cmd_ls(image_path: &str, partition_index: u32, path: &str) -> Result<()> {
    let part = partition_by_index(image_path, partition_index)?;
    match detect_partition_fs(image_path, &part)? {
        FsKind::Ntfs => ls_ntfs(image_path, &part, path),
        FsKind::Ext => ls_ext(image_path, &part, path),
        FsKind::Unknown => bail!("partition {} is not NTFS/ext4", partition_index),
    }
}

fn cmd_cat(image_path: &str, partition_index: u32, path: &str) -> Result<()> {
    let part = partition_by_index(image_path, partition_index)?;
    match detect_partition_fs(image_path, &part)? {
        FsKind::Ntfs => cat_ntfs(image_path, &part, path),
        FsKind::Ext => cat_ext(image_path, &part, path),
        FsKind::Unknown => bail!("partition {} is not NTFS/ext4", partition_index),
    }
}

fn detect_partitions(image_path: &str) -> Result<Vec<PartitionRecord>> {
    let path = Path::new(image_path);

    let mut image = VdiImage::open(path)?;
    if let Ok(gpt) = GPT::find_from(&mut image) {
        let mut parts = Vec::new();
        for (index, partition) in gpt.iter() {
            if !partition.is_used() {
                continue;
            }
            let range = gpt.get_partition_byte_range(index)?;
            parts.push(PartitionRecord {
                scheme: PartitionScheme::Gpt,
                index,
                start_byte: *range.start(),
                size_bytes: inclusive_len(&range),
                label: partition.partition_name.as_str().to_string(),
            });
        }
        return Ok(parts);
    }

    let mut image = VdiImage::open(path)?;
    let mbr = MBR::read_from(&mut image, 512)?;
    let mut parts = Vec::new();
    for (index, partition) in mbr.iter() {
        if !partition.is_used() || partition.is_extended() {
            continue;
        }
        parts.push(PartitionRecord {
            scheme: PartitionScheme::Mbr,
            index: index as u32,
            start_byte: u64::from(partition.starting_lba) * u64::from(mbr.sector_size),
            size_bytes: u64::from(partition.sectors) * u64::from(mbr.sector_size),
            label: format!("mbr type 0x{:02x}", partition.sys),
        });
    }
    Ok(parts)
}

fn partition_by_index(image_path: &str, index: u32) -> Result<PartitionRecord> {
    detect_partitions(image_path)?
        .into_iter()
        .find(|part| part.index == index)
        .ok_or_else(|| anyhow!("partition {index} not found"))
}

fn detect_partition_fs(image_path: &str, part: &PartitionRecord) -> Result<FsKind> {
    let mut reader = PartitionReader::open(Path::new(image_path), part)?;
    let mut ntfs_sig = [0u8; 8];
    reader.read_exact_at(3, &mut ntfs_sig)?;
    if &ntfs_sig == b"NTFS    " {
        return Ok(FsKind::Ntfs);
    }

    let mut ext_magic = [0u8; 2];
    if reader.read_exact_at(1024 + 56, &mut ext_magic).is_ok() && u16::from_le_bytes(ext_magic) == 0xef53 {
        return Ok(FsKind::Ext);
    }

    Ok(FsKind::Unknown)
}

fn ls_ext(image_path: &str, part: &PartitionRecord, path: &str) -> Result<()> {
    let fs = Ext4::load(Box::new(PartitionReader::open(Path::new(image_path), part)?))?;
    let path = normalize_ext_path(path);
    for entry in fs.read_dir(path.as_str())? {
        let entry = entry?;
        println!("{}", entry.path().display());
    }
    Ok(())
}

fn cat_ext(image_path: &str, part: &PartitionRecord, path: &str) -> Result<()> {
    let fs = Ext4::load(Box::new(PartitionReader::open(Path::new(image_path), part)?))?;
    let path = normalize_ext_path(path);
    let data = fs.read(path.as_str())?;
    io::stdout().write_all(&data)?;
    Ok(())
}

fn normalize_ext_path(path: &str) -> String {
    if path.is_empty() {
        return "/".to_string();
    }
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn ls_ntfs(image_path: &str, part: &PartitionRecord, path: &str) -> Result<()> {
    let reader = PartitionReader::open(Path::new(image_path), part)?;
    let mut fs = BufReader::new(reader);
    let mut ntfs = Ntfs::new(&mut fs)?;
    ntfs.read_upcase_table(&mut fs)?;
    let dir = resolve_ntfs_path(&ntfs, &mut fs, path)?;
    if !dir.is_directory() {
        bail!("{path} is not a directory")
    }

    let index = dir.directory_index(&mut fs)?;
    let mut iter = index.entries();
    while let Some(entry) = iter.next(&mut fs) {
        let entry = entry?;
        let file_name = entry
            .key()
            .expect("key must exist for a found Index Entry")?;
        let kind = if file_name.is_directory() { "<DIR>" } else { "FILE " };
        println!("{} {}", kind, file_name.name().to_string_lossy());
    }
    Ok(())
}

fn cat_ntfs(image_path: &str, part: &PartitionRecord, path: &str) -> Result<()> {
    let reader = PartitionReader::open(Path::new(image_path), part)?;
    let mut fs = BufReader::new(reader);
    let mut ntfs = Ntfs::new(&mut fs)?;
    ntfs.read_upcase_table(&mut fs)?;
    let file = resolve_ntfs_path(&ntfs, &mut fs, path)?;
    if file.is_directory() {
        bail!("{path} is a directory")
    }

    let data_item = file
        .data(&mut fs, "")
        .ok_or_else(|| anyhow!("file has no default $DATA stream"))??;
    let data_attribute = data_item.to_attribute()?;
    let mut data_value = data_attribute.value(&mut fs)?;
    let mut buffer = [0u8; 4096];
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    loop {
        let bytes_read = data_value.read(&mut fs, &mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        stdout.write_all(&buffer[..bytes_read])?;
    }

    Ok(())
}

fn resolve_ntfs_path<'n, T>(ntfs: &'n Ntfs, fs: &mut T, path: &str) -> Result<NtfsFile<'n>>
where
    T: Read + Seek,
{
    let mut current = ntfs.root_directory(fs)?;
    let trimmed = path.trim_matches(|ch| ch == '/' || ch == '\\');
    if trimmed.is_empty() {
        return Ok(current);
    }

    for component in trimmed.split(['/', '\\']).filter(|part| !part.is_empty()) {
        let index = current.directory_index(fs)?;
        let mut finder = index.finder();
        let maybe_entry = NtfsFileNameIndex::find(&mut finder, ntfs, fs, component);
        let entry = match maybe_entry {
            Some(entry) => entry?,
            None => bail!("NTFS path component not found: {component}"),
        };
        current = entry.to_file(ntfs, fs)?;
    }

    Ok(current)
}

fn inclusive_len(range: &RangeInclusive<u64>) -> u64 {
    range.end().saturating_sub(*range.start()).saturating_add(1)
}

fn le_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().expect("u32 slice"))
}

fn le_u64(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().expect("u64 slice"))
}