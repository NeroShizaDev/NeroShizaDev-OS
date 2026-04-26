// ============================================================
// APP SLOTS — «Program Files» для NeroShizaDev-OS
// ============================================================
// 16 слотов по 64KB каждый = 1MB в BSS.
// Каждый слот — область памяти куда установщик копирует
// полный .nhs blob без распаковки.
//
// Аналогия с Windows:
//   C:\Program Files\App1\  →  APP_MEMORY[0][0..65535]
//   C:\Program Files\App2\  →  APP_MEMORY[1][0..65535]
//   ...
//
// BSS: компоновщик выделяет 1MB нулей, не раздувая бинарник.
// ============================================================

/// Размер одного слота (64 КБ)
pub const SLOT_SIZE: usize = 64 * 1024;

/// Максимум установленных приложений
pub const MAX_SLOTS: usize = 16;

/// Хранилище приложений: 16 × 64KB = 1MB в BSS.
///
/// SAFETY: доступ только через install/uninstall/read,
/// которые проверяют slot_index < MAX_SLOTS.
/// Bare-metal однопоточное ядро — гонок нет.
static mut APP_MEMORY: [[u8; SLOT_SIZE]; MAX_SLOTS] = [[0u8; SLOT_SIZE]; MAX_SLOTS];

/// Флаги занятости слотов (true = занят)
static mut SLOT_OCCUPIED: [bool; MAX_SLOTS] = [false; MAX_SLOTS];

/// Размер реально записанных данных в каждом слоте
static mut SLOT_USED: [usize; MAX_SLOTS] = [0; MAX_SLOTS];

// ============================================================
// Публичный API
// ============================================================

/// Находит первый свободный слот. Возвращает индекс или None.
///
/// Аналог: Windows ищет папку в Program Files.
pub fn find_free_slot() -> Option<usize> {
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

    unsafe {
        if SLOT_OCCUPIED[slot] {
            return false;
        }

        // Очищаем слот (BSS-секция)
        for b in APP_MEMORY[slot].iter_mut() {
            *b = 0;
        }

        // Копируем данные
        APP_MEMORY[slot][..data.len()].copy_from_slice(data);
        SLOT_USED[slot] = data.len();
        SLOT_OCCUPIED[slot] = true;
    }

    true
}

/// Удаляет приложение из слота: обнуляет память, освобождает.
///
/// Аналог: Windows Uninstall → удаление папки из Program Files.
pub fn uninstall_slot(slot: usize) -> bool {
    if slot >= MAX_SLOTS {
        return false;
    }

    unsafe {
        if !SLOT_OCCUPIED[slot] {
            return false;
        }

        // Обнуляем (секьюрное удаление — не оставляем данные)
        for b in APP_MEMORY[slot].iter_mut() {
            *b = 0;
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
    unsafe {
        if !SLOT_OCCUPIED[slot] {
            return None;
        }
        Some(&APP_MEMORY[slot][..SLOT_USED[slot]])
    }
}

/// Возвращает размер данных в слоте.
pub fn slot_used_bytes(slot: usize) -> usize {
    if slot >= MAX_SLOTS {
        return 0;
    }
    unsafe { SLOT_USED[slot] }
}
