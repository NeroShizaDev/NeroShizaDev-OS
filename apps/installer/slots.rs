// ============================================================
// APP SLOTS — совместимый фасад поверх NSFS RAM-disk
// ============================================================
// Снаружи installer/runtime по-прежнему видят слоты 0..15,
// но физически данные теперь лежат в NSFS v1 на RAM-disk.
//
// Это даёт три вещи сразу:
//   1. Сырые сектора и superblock/index уже существуют.
//   2. NHS-пакеты пишутся в region NhsStore, а не в отдельный BSS-массив.
//   3. На диске можно хранить не только apps, но и kernel assets.
// ============================================================

use super::nsfs::{
    self, NSFS_BLOCK_SIZE_512, NsfsIndex, NsfsObjectEntry, NsfsObjectKind, NsfsRamDisk,
    NsfsRegionKind, NsfsVolume,
};
use super::{catalog, registry};
use core::mem::MaybeUninit;
use core::ptr::{addr_of_mut, write_bytes};

pub const SLOT_SIZE: usize = 64 * 1024;
pub const MAX_SLOTS: usize = 32;
pub const NSFS_DISK_BYTES: usize = 2 * 1024 * 1024;
pub const NSFS_MAX_OBJECTS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemovalStats {
    pub removed_objects: u16,
    pub removed_bytes: u64,
}

static mut SLOT_OCCUPIED: [bool; MAX_SLOTS] = [false; MAX_SLOTS];
static mut SLOT_USED: [usize; MAX_SLOTS] = [0; MAX_SLOTS];
static mut DISK: NsfsRamDisk<NSFS_DISK_BYTES> = NsfsRamDisk::new();
static mut VOLUME: MaybeUninit<NsfsVolume<NSFS_MAX_OBJECTS>> = MaybeUninit::uninit();
static mut VOLUME_READY: bool = false;

const KERNEL_BOOT_CFG: &[u8] = b"kernel=neroshiza_dev_os\nlocale=ru-RU\nshell=hybrid\n";
const KERNEL_MODULES_LST: &[u8] =
    b"core/lib.rs\ncore/main.rs\ncore/interrupts.rs\napps/shell/commands.rs\nfonts/unicode.rs\n";
const KERNEL_BANNER_TXT: &[u8] = b"NeroShizaDev-OS kernel RAM-disk asset\n";

// ============================================================
// Публичный API
// ============================================================

/// Находит первый свободный слот. Возвращает индекс или None.
///
/// Аналог: Windows ищет папку в Program Files.
pub fn find_free_slot() -> Option<usize> {
    if !ensure_storage_ready() {
        return None;
    }
    unsafe {
        for i in 0..MAX_SLOTS {
            if !SLOT_OCCUPIED[i] {
                return Some(i);
            }
        }
        None
    }
}

/// Количество занятых слотов.
pub fn used_count() -> usize {
    if !ensure_storage_ready() {
        return 0;
    }
    unsafe {
        let mut count = 0;
        for i in 0..MAX_SLOTS {
            if SLOT_OCCUPIED[i] {
                count += 1;
            }
        }
        count
    }
}

/// Количество свободных слотов.
pub fn free_count() -> usize {
    MAX_SLOTS - used_count()
}

/// Устанавливает данные в слот: копирует полный `.nhs` blob в APP_MEMORY[slot].
///
/// Аналог: Windows копирует весь package blob в C:\Program Files\AppName\.
///
/// Возвращает false если слот занят или данные слишком большие.
pub fn install_to_slot(slot: usize, data: &[u8]) -> bool {
    if slot >= MAX_SLOTS {
        return false;
    }
    if data.len() > SLOT_SIZE {
        return false;
    }

    if !ensure_storage_ready() {
        return false;
    }

    unsafe {
        if SLOT_OCCUPIED[slot] {
            return false;
        }

        let mut path_buf = [0u8; nsfs::NSFS_NAME_CAPACITY];
        let path = match slot_path(slot, &mut path_buf) {
            Some(path) => path,
            None => return false,
        };
        let mut owner_buf = [0u8; nsfs::NSFS_OWNER_CAPACITY];
        let owner = match slot_owner_slug(slot, &mut owner_buf) {
            Some(owner) => owner,
            None => return false,
        };

        match with_volume_mut(|volume, disk| {
            volume.add_object_owned(
                disk,
                path,
                owner,
                NsfsObjectKind::NhsPackage,
                NsfsRegionKind::NhsStore,
                0,
                data,
                0,
                slot as u64,
            )
        }) {
            Ok(_) => {
                SLOT_USED[slot] = data.len();
                SLOT_OCCUPIED[slot] = true;
                true
            }
            Err(_) => false,
        }
    }
}

/// Удаляет приложение из слота: обнуляет память, освобождает.
///
/// Аналог: Windows Uninstall → удаление папки из Program Files.
pub fn uninstall_slot(slot: usize) -> bool {
    if slot >= MAX_SLOTS {
        return false;
    }

    if !ensure_storage_ready() {
        return false;
    }

    unsafe {
        if !SLOT_OCCUPIED[slot] {
            return false;
        }

        let mut path_buf = [0u8; nsfs::NSFS_NAME_CAPACITY];
        let path = match slot_path(slot, &mut path_buf) {
            Some(path) => path,
            None => return false,
        };

        if with_volume_mut(|volume, disk| volume.remove_object(disk, path)).is_err() {
            return false;
        }

        SLOT_USED[slot] = 0;
        SLOT_OCCUPIED[slot] = false;
    }

    true
}

/// Проверяет занятость слота.
pub fn is_occupied(slot: usize) -> bool {
    if slot >= MAX_SLOTS {
        return false;
    }
    if !ensure_storage_ready() {
        return false;
    }
    unsafe { SLOT_OCCUPIED[slot] }
}

/// Возвращает ссылку на данные установленного приложения.
///
/// SAFETY: возвращает &[u8] на статическую память в BSS.
/// Вызывающий не должен удерживать ссылку через uninstall_slot().
pub fn read_slot(slot: usize) -> Option<&'static [u8]> {
    if slot >= MAX_SLOTS {
        return None;
    }

    if !ensure_storage_ready() {
        return None;
    }

    unsafe {
        if !SLOT_OCCUPIED[slot] {
            return None;
        }

        let mut path_buf = [0u8; nsfs::NSFS_NAME_CAPACITY];
        let path = slot_path(slot, &mut path_buf)?;
        read_path_inner(path)
    }
}

/// Возвращает размер данных в слоте.
pub fn slot_used_bytes(slot: usize) -> usize {
    if slot >= MAX_SLOTS {
        return 0;
    }
    if !ensure_storage_ready() {
        return 0;
    }
    unsafe { SLOT_USED[slot] }
}

pub fn clear_slot_state(slot: usize) -> bool {
    if slot >= MAX_SLOTS {
        return false;
    }
    unsafe {
        SLOT_OCCUPIED[slot] = false;
        SLOT_USED[slot] = 0;
    }
    true
}

pub fn disk_capacity_bytes() -> usize {
    NSFS_DISK_BYTES
}

pub fn disk_used_bytes() -> usize {
    if !ensure_storage_ready() {
        return 0;
    }
    unsafe { with_volume(|volume, _| volume.used_bytes() as usize) }
}

pub fn disk_object_count() -> usize {
    if !ensure_storage_ready() {
        return 0;
    }
    unsafe { with_volume(|volume, _| volume.object_count()) }
}

pub fn for_each_disk_object(mut f: impl FnMut(&NsfsObjectEntry)) {
    if !ensure_storage_ready() {
        return;
    }
    unsafe {
        with_volume(|volume, _| {
            volume.index.for_each(|entry| f(entry));
        });
    }
}

pub fn read_path(path: &str) -> Option<&'static [u8]> {
    if !ensure_storage_ready() {
        return None;
    }
    unsafe { read_path_inner(path) }
}

pub fn remove_path(path: &str) -> bool {
    if !ensure_storage_ready() {
        return false;
    }

    unsafe { with_volume_mut(|volume, disk| volume.remove_object(disk, path)).is_ok() }
}

pub fn scan_prefix_stats(prefix: &str) -> RemovalStats {
    if !ensure_storage_ready() {
        return RemovalStats {
            removed_objects: 0,
            removed_bytes: 0,
        };
    }

    let mut stats = RemovalStats {
        removed_objects: 0,
        removed_bytes: 0,
    };
    for_each_disk_object(|entry| {
        if entry.path().starts_with(prefix) {
            stats.removed_objects += 1;
            stats.removed_bytes += entry.size_bytes;
        }
    });
    stats
}

pub fn remove_prefix(prefix: &str) -> RemovalStats {
    if !ensure_storage_ready() {
        return RemovalStats {
            removed_objects: 0,
            removed_bytes: 0,
        };
    }

    let mut names = [[0u8; nsfs::NSFS_NAME_CAPACITY]; NSFS_MAX_OBJECTS];
    let mut lengths = [0usize; NSFS_MAX_OBJECTS];
    let mut count = 0usize;
    let mut removed_bytes = 0u64;

    for_each_disk_object(|entry| {
        if count >= NSFS_MAX_OBJECTS || !entry.path().starts_with(prefix) {
            return;
        }
        let path = entry.path().as_bytes();
        names[count][..path.len()].copy_from_slice(path);
        lengths[count] = path.len();
        removed_bytes += entry.size_bytes;
        count += 1;
    });

    let mut removed_objects = 0u16;
    for index in 0..count {
        if let Ok(path) = core::str::from_utf8(&names[index][..lengths[index]]) {
            if remove_path(path) {
                removed_objects += 1;
            }
        }
    }

    RemovalStats {
        removed_objects,
        removed_bytes,
    }
}

pub fn scan_owner_stats(owner_slug: &str) -> RemovalStats {
    if !ensure_storage_ready() {
        return RemovalStats {
            removed_objects: 0,
            removed_bytes: 0,
        };
    }

    let mut stats = RemovalStats {
        removed_objects: 0,
        removed_bytes: 0,
    };
    for_each_disk_object(|entry| {
        if entry.owner_slug() == owner_slug {
            stats.removed_objects += 1;
            stats.removed_bytes += entry.size_bytes;
        }
    });
    stats
}

pub fn remove_owner(owner_slug: &str) -> RemovalStats {
    if !ensure_storage_ready() {
        return RemovalStats {
            removed_objects: 0,
            removed_bytes: 0,
        };
    }

    let mut names = [[0u8; nsfs::NSFS_NAME_CAPACITY]; NSFS_MAX_OBJECTS];
    let mut lengths = [0usize; NSFS_MAX_OBJECTS];
    let mut count = 0usize;
    let mut removed_bytes = 0u64;

    for_each_disk_object(|entry| {
        if count >= NSFS_MAX_OBJECTS || entry.owner_slug() != owner_slug {
            return;
        }
        let path = entry.path().as_bytes();
        names[count][..path.len()].copy_from_slice(path);
        lengths[count] = path.len();
        removed_bytes += entry.size_bytes;
        count += 1;
    });

    let mut removed_objects = 0u16;
    for index in 0..count {
        if let Ok(path) = core::str::from_utf8(&names[index][..lengths[index]]) {
            if remove_path(path) {
                removed_objects += 1;
            }
        }
    }

    RemovalStats {
        removed_objects,
        removed_bytes,
    }
}

pub fn for_each_owned_object(owner_slug: &str, mut f: impl FnMut(&NsfsObjectEntry)) {
    if !ensure_storage_ready() {
        return;
    }
    for_each_disk_object(|entry| {
        if entry.owner_slug() == owner_slug {
            f(entry);
        }
    });
}

pub fn builtin_catalog_path<'a>(
    slug: &str,
    out: &'a mut [u8; nsfs::NSFS_NAME_CAPACITY],
) -> Option<&'a str> {
    catalog_path(slug, out)
}

#[allow(static_mut_refs)]
fn ensure_storage_ready() -> bool {
    unsafe {
        if VOLUME_READY {
            return true;
        }

        let total_blocks = match DISK.total_blocks(NSFS_BLOCK_SIZE_512) {
            Ok(blocks) => blocks,
            Err(_) => return false,
        };
        let layout = match nsfs::NsfsLayout::plan(total_blocks, NSFS_BLOCK_SIZE_512) {
            Ok(layout) => layout,
            Err(_) => return false,
        };
        let superblock = layout.to_superblock();
        let volume_ptr = VOLUME.as_mut_ptr();

        // NsfsVolume<64> contains a ~9KB index table. Building it as a local
        // temporary overflows the tiny runtime stack used by the shell path,
        // so initialize the static storage in place instead.
        addr_of_mut!((*volume_ptr).layout).write(layout);
        addr_of_mut!((*volume_ptr).superblock).write(superblock);
        write_bytes(
            addr_of_mut!((*volume_ptr).index).cast::<u8>(),
            0,
            core::mem::size_of::<NsfsIndex<NSFS_MAX_OBJECTS>>(),
        );
        addr_of_mut!((*volume_ptr).cursors).write(layout.regions.map(nsfs::NsfsRegionCursor::new));

        let volume = &mut *volume_ptr;
        DISK.clear();
        if volume.flush_superblock(&mut DISK).is_err() {
            return false;
        }
        if volume.flush_index(&mut DISK).is_err() {
            return false;
        }

        if seed_kernel_assets(volume, &mut DISK).is_err() {
            return false;
        }
        if seed_builtin_apps(volume, &mut DISK).is_err() {
            return false;
        }

        VOLUME_READY = true;
        true
    }
}

#[allow(static_mut_refs)]
unsafe fn with_volume_mut<R>(
    f: impl FnOnce(&mut NsfsVolume<NSFS_MAX_OBJECTS>, &mut NsfsRamDisk<NSFS_DISK_BYTES>) -> R,
) -> R {
    let volume = VOLUME.assume_init_mut();
    f(volume, &mut DISK)
}

#[allow(static_mut_refs)]
unsafe fn with_volume<R>(
    f: impl FnOnce(&NsfsVolume<NSFS_MAX_OBJECTS>, &NsfsRamDisk<NSFS_DISK_BYTES>) -> R,
) -> R {
    let volume = VOLUME.assume_init_ref();
    f(volume, &DISK)
}

unsafe fn seed_kernel_assets(
    volume: &mut NsfsVolume<NSFS_MAX_OBJECTS>,
    disk: &mut NsfsRamDisk<NSFS_DISK_BYTES>,
) -> Result<(), nsfs::NsfsError> {
    volume.add_object_owned(
        disk,
        "kernel/boot.cfg",
        "system.kernel",
        NsfsObjectKind::Config,
        NsfsRegionKind::UserStore,
        0,
        KERNEL_BOOT_CFG,
        0,
        1,
    )?;
    volume.add_object_owned(
        disk,
        "kernel/modules.lst",
        "system.kernel",
        NsfsObjectKind::KernelAsset,
        NsfsRegionKind::UserStore,
        0,
        KERNEL_MODULES_LST,
        0,
        2,
    )?;
    volume.add_object_owned(
        disk,
        "kernel/banner.txt",
        "system.kernel",
        NsfsObjectKind::KernelAsset,
        NsfsRegionKind::UserStore,
        0,
        KERNEL_BANNER_TXT,
        0,
        3,
    )?;
    Ok(())
}

unsafe fn seed_builtin_apps(
    volume: &mut NsfsVolume<NSFS_MAX_OBJECTS>,
    disk: &mut NsfsRamDisk<NSFS_DISK_BYTES>,
) -> Result<(), nsfs::NsfsError> {
    for (slot, spec) in catalog::BUILTIN_APPS.iter().enumerate() {
        if slot >= MAX_SLOTS {
            break;
        }

        let mut path_buf = [0u8; nsfs::NSFS_NAME_CAPACITY];
        let path = match catalog_path(spec.slug, &mut path_buf) {
            Some(path) => path,
            None => continue,
        };

        let mut payload = [0u8; 192];
        let size = build_builtin_payload(spec, &mut payload);
        volume.add_object_owned(
            disk,
            path,
            spec.slug,
            NsfsObjectKind::Config,
            NsfsRegionKind::UserStore,
            0,
            &payload[..size],
            0,
            slot as u64 + 10,
        )?;

        SLOT_OCCUPIED[slot] = true;
        SLOT_USED[slot] = size;
        registry::register_builtin(slot, spec, size as u32);
    }
    Ok(())
}

#[allow(static_mut_refs)]
unsafe fn read_path_inner(path: &str) -> Option<&'static [u8]> {
    if !VOLUME_READY {
        return None;
    }
    let volume = VOLUME.assume_init_ref();
    let entry = volume.index.get(path).ok()??;
    let byte_offset = nsfs::block_to_offset(entry.start_block, NSFS_BLOCK_SIZE_512).ok()?;
    DISK.slice(byte_offset, entry.size_bytes as usize)
}

fn slot_path<'a>(slot: usize, out: &'a mut [u8; nsfs::NSFS_NAME_CAPACITY]) -> Option<&'a str> {
    if slot >= MAX_SLOTS {
        return None;
    }

    let mut cursor = 0usize;
    let prefix = b"apps/slot";
    out[..prefix.len()].copy_from_slice(prefix);
    cursor += prefix.len();

    let digits_start = cursor;
    cursor += write_decimal(&mut out[cursor..], slot as u32)?;

    let suffix = b".nhs";
    out[cursor..cursor + suffix.len()].copy_from_slice(suffix);
    cursor += suffix.len();

    if digits_start == prefix.len() && cursor > out.len() {
        return None;
    }

    core::str::from_utf8(&out[..cursor]).ok()
}

fn slot_owner_slug<'a>(
    slot: usize,
    out: &'a mut [u8; nsfs::NSFS_OWNER_CAPACITY],
) -> Option<&'a str> {
    let prefix = b"nhs.slot";
    if prefix.len() >= out.len() {
        return None;
    }

    out[..prefix.len()].copy_from_slice(prefix);
    let digits = write_decimal(&mut out[prefix.len()..], slot as u32)?;
    core::str::from_utf8(&out[..prefix.len() + digits]).ok()
}

fn catalog_path<'a>(slug: &str, out: &'a mut [u8; nsfs::NSFS_NAME_CAPACITY]) -> Option<&'a str> {
    let prefix = b"apps/catalog/";
    let suffix = b".app";
    let slug_bytes = slug.as_bytes();
    let total = prefix.len() + slug_bytes.len() + suffix.len();
    if total > out.len() {
        return None;
    }

    out[..prefix.len()].copy_from_slice(prefix);
    out[prefix.len()..prefix.len() + slug_bytes.len()].copy_from_slice(slug_bytes);
    out[prefix.len() + slug_bytes.len()..total].copy_from_slice(suffix);
    core::str::from_utf8(&out[..total]).ok()
}

fn build_builtin_payload(spec: &catalog::BuiltinAppSpec, out: &mut [u8; 192]) -> usize {
    out.fill(0);
    let mut cursor = 0usize;
    cursor += write_kv_line(out, cursor, "name", spec.name);
    cursor += write_kv_line(out, cursor, "slug", spec.slug);
    cursor += write_kv_line(out, cursor, "author", spec.author);
    cursor += write_kv_line(out, cursor, "summary", spec.summary);
    cursor += write_kv_line(out, cursor, "kind", app_kind_name(spec.kind));
    cursor
}

fn write_kv_line(out: &mut [u8; 192], cursor: usize, key: &str, value: &str) -> usize {
    let key_bytes = key.as_bytes();
    let value_bytes = value.as_bytes();
    let needed = key_bytes.len() + 1 + value_bytes.len() + 1;
    if cursor + needed > out.len() {
        return 0;
    }

    out[cursor..cursor + key_bytes.len()].copy_from_slice(key_bytes);
    let mut offset = cursor + key_bytes.len();
    out[offset] = b'=';
    offset += 1;
    out[offset..offset + value_bytes.len()].copy_from_slice(value_bytes);
    offset += value_bytes.len();
    out[offset] = b'\n';
    needed
}

fn app_kind_name(kind: crate::apps::activity::AppKind) -> &'static str {
    match kind {
        crate::apps::activity::AppKind::Games => "games",
        crate::apps::activity::AppKind::Calculator => "calculator",
        crate::apps::activity::AppKind::Jackal => "jackal",
        crate::apps::activity::AppKind::Menger => "menger",
        crate::apps::activity::AppKind::Voodoo => "voodoo",
        crate::apps::activity::AppKind::Chronos => "chronos",
        crate::apps::activity::AppKind::Rtc => "rtc",
        crate::apps::activity::AppKind::Rng => "rng",
        crate::apps::activity::AppKind::Beeper => "beeper",
        crate::apps::activity::AppKind::Fpu => "fpu",
        crate::apps::activity::AppKind::Locale => "locale",
        crate::apps::activity::AppKind::Nhs => "nhs",
        crate::apps::activity::AppKind::Launcher => "launcher",
        crate::apps::activity::AppKind::Doom => "doom",
        crate::apps::activity::AppKind::Tribe => "tribe",
    }
}

fn write_decimal(out: &mut [u8], mut value: u32) -> Option<usize> {
    if out.is_empty() {
        return None;
    }
    if value == 0 {
        out[0] = b'0';
        return Some(1);
    }

    let mut digits = [0u8; 10];
    let mut len = 0usize;
    while value > 0 {
        digits[len] = b'0' + (value % 10) as u8;
        value /= 10;
        len += 1;
    }
    if len > out.len() {
        return None;
    }
    for index in 0..len {
        out[index] = digits[len - 1 - index];
    }
    Some(len)
}
