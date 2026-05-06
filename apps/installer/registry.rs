// ============================================================
// APP REGISTRY — «Реестр Windows» для NeroShizaDev-OS
// ============================================================
// HKLM\SOFTWARE\...\Uninstall\  →  REGISTRY[0..15]
//
// Launcher читает реестр и показывает установленные .nhs apps
// в меню наравне со встроенными (Games, Doom, Jackal).
// ============================================================

use super::header::NhsManifest;
use super::slots::MAX_SLOTS;
use crate::apps::activity::AppKind;

pub const FLAG_REGISTRY_BUILTIN: u32 = 1 << 31;

/// Одна запись = одно установленное приложение.
#[derive(Clone, Copy)]
pub struct InstalledApp {
    pub occupied: bool,
    pub slot_index: u8,
    pub name: [u8; 32],
    pub author: [u8; 32],
    pub version: [u8; 4],
    pub icon_char: u8,
    pub category: u8,
    pub installed_size: u32,
    pub flags: u32,
    pub lump_count: u16,
    pub entry_point: u32,
}

impl InstalledApp {
    pub const fn empty() -> Self {
        Self {
            occupied: false,
            slot_index: 0,
            name: [0; 32],
            author: [0; 32],
            version: [0; 4],
            icon_char: b'?',
            category: 0,
            installed_size: 0,
            flags: 0,
            lump_count: 0,
            entry_point: 0,
        }
    }

    pub fn name_str(&self) -> &str {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(32);
        core::str::from_utf8(&self.name[..end]).unwrap_or("???")
    }

    pub fn author_str(&self) -> &str {
        let end = self.author.iter().position(|&b| b == 0).unwrap_or(32);
        core::str::from_utf8(&self.author[..end]).unwrap_or("???")
    }

    pub fn version_tuple(&self) -> (u8, u8, u8) {
        (self.version[0], self.version[1], self.version[2])
    }

    pub fn is_builtin(&self) -> bool {
        self.flags & FLAG_REGISTRY_BUILTIN != 0
    }

    pub fn launch_kind(&self) -> AppKind {
        if self.is_builtin() {
            AppKind::from_registry_id(self.entry_point as u8).unwrap_or(AppKind::Nhs)
        } else {
            AppKind::Nhs
        }
    }
}

// ── Глобальный реестр (static, no heap) ──────────────────────

static mut REGISTRY: [InstalledApp; MAX_SLOTS] = [InstalledApp::empty(); MAX_SLOTS];

// ── API ──────────────────────────────────────────────────────

/// Регистрирует установленный .nhs в реестре.
/// Аналог: Windows пишет ключ в HKLM\...\Uninstall\.
pub fn register(
    slot: usize,
    manifest: &NhsManifest,
    flags: u32,
    size: u32,
    lump_count: u16,
    entry_point: u32,
) -> bool {
    if slot >= MAX_SLOTS {
        return false;
    }
    unsafe {
        if REGISTRY[slot].occupied {
            return false;
        }

        let r = &mut REGISTRY[slot];
        r.occupied = true;
        r.slot_index = slot as u8;
        r.name = manifest.app_name;
        r.author = manifest.author;
        r.version = manifest.version;
        r.icon_char = b'\xFE'; // ■ — по умолчанию
        r.category = 0;
        r.installed_size = size;
        r.flags = flags;
        r.lump_count = lump_count;
        r.entry_point = entry_point;
        true
    }
}

pub fn register_builtin(slot: usize, spec: &super::catalog::BuiltinAppSpec, size: u32) -> bool {
    if slot >= MAX_SLOTS {
        return false;
    }

    unsafe {
        if REGISTRY[slot].occupied {
            return false;
        }

        let r = &mut REGISTRY[slot];
        r.occupied = true;
        r.slot_index = slot as u8;
        copy_str_into(&mut r.name, spec.name);
        copy_str_into(&mut r.author, spec.author);
        r.version = [spec.version[0], spec.version[1], spec.version[2], 0];
        r.icon_char = spec.name.as_bytes().first().copied().unwrap_or(b'?');
        r.category = spec.category;
        r.installed_size = size;
        r.flags = FLAG_REGISTRY_BUILTIN;
        r.lump_count = 0;
        r.entry_point = spec.kind as u32;
        true
    }
}

/// Удаляет запись из реестра.
/// Аналог: Windows удаляет ключ из Uninstall.
pub fn unregister(slot: usize) -> bool {
    if slot >= MAX_SLOTS {
        return false;
    }
    unsafe {
        if !REGISTRY[slot].occupied {
            return false;
        }
        REGISTRY[slot] = InstalledApp::empty();
        true
    }
}

/// Количество установленных приложений.
pub fn installed_count() -> usize {
    unsafe {
        let mut count = 0;
        for i in 0..MAX_SLOTS {
            if REGISTRY[i].occupied {
                count += 1;
            }
        }
        count
    }
}

/// Получить запись по индексу слота.
pub fn get(slot: usize) -> Option<&'static InstalledApp> {
    if slot >= MAX_SLOTS {
        return None;
    }
    unsafe {
        if REGISTRY[slot].occupied {
            Some(&REGISTRY[slot])
        } else {
            None
        }
    }
}

/// Получить только реально установленный NHS-пакет.
/// Встроенные builtin-записи для launcher-секции не возвращаются.
pub fn get_user(slot: usize) -> Option<&'static InstalledApp> {
    match get(slot) {
        Some(app) if !app.is_builtin() => Some(app),
        _ => None,
    }
}

pub fn user_installed_count() -> usize {
    unsafe {
        let mut count = 0;
        for i in 0..MAX_SLOTS {
            if REGISTRY[i].occupied && !REGISTRY[i].is_builtin() {
                count += 1;
            }
        }
        count
    }
}

/// Найти по имени. Возвращает индекс слота.
pub fn find_by_name(name: &[u8]) -> Option<usize> {
    unsafe {
        for i in 0..MAX_SLOTS {
            if !REGISTRY[i].occupied {
                continue;
            }
            let n = &REGISTRY[i].name;
            let nlen = n.iter().position(|&b| b == 0).unwrap_or(32);
            if nlen == name.len() && &n[..nlen] == name {
                return Some(i);
            }
        }
        None
    }
}

/// Итерирует по всем установленным приложениям.
/// Вызывает `f(slot_index, &InstalledApp)` для каждого.
pub fn for_each(mut f: impl FnMut(usize, &InstalledApp)) {
    unsafe {
        for i in 0..MAX_SLOTS {
            if REGISTRY[i].occupied {
                f(i, &REGISTRY[i]);
            }
        }
    }
}

pub fn for_each_user(mut f: impl FnMut(usize, &InstalledApp)) {
    unsafe {
        for i in 0..MAX_SLOTS {
            if REGISTRY[i].occupied && !REGISTRY[i].is_builtin() {
                f(i, &REGISTRY[i]);
            }
        }
    }
}

/// Возвращает все записи как массив + количество.
/// Для использования в launcher.
pub fn list() -> ([Option<(usize, InstalledApp)>; MAX_SLOTS], usize) {
    let mut out: [Option<(usize, InstalledApp)>; MAX_SLOTS] = [None; MAX_SLOTS];
    let mut count = 0;
    unsafe {
        for i in 0..MAX_SLOTS {
            if REGISTRY[i].occupied {
                out[count] = Some((i, REGISTRY[i]));
                count += 1;
            }
        }
    }
    (out, count)
}

fn copy_str_into<const N: usize>(out: &mut [u8; N], value: &str) {
    out.fill(0);
    let bytes = value.as_bytes();
    let len = bytes.len().min(N);
    out[..len].copy_from_slice(&bytes[..len]);
}
