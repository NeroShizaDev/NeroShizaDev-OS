// ============================================================
// DOOM WATCHDOG — RDTSC-таймер кадров
// ============================================================
// Если кадр не обновлялся >500 мс — перезагрузка системы.
// RDTSC: монотонные тики TSC без системных прерываний.
//
// Используется для:
//   1. Обнаружения зависания игрового цикла
//   2. Синхронизации FPS (~35 кадров/сек, как оригинальный Doom)
// ============================================================

use core::arch::asm;

// Грубая оценка тиков TSC за ~35 мс кадр (при ~1 ГГц: 35_000_000)
// При более быстром CPU это значение меньше реального — безопасно.
const FRAME_TICKS:    u64 = 35_000_000;  // ~35 FPS @ 1 GHz
const TIMEOUT_TICKS:  u64 = 500_000_000; // ~500 мс до watchdog reboot

static mut LAST_FRAME_TSC: u64 = 0;
static mut FRAME_COUNT:    u64 = 0;

// ============================================================
// RDTSC
// ============================================================

/// Читает счётчик TSC (Time Stamp Counter).
/// Доступен без системных вызовов, без прерываний.
#[inline(always)]
pub fn rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    // SAFETY: RDTSC — непривилегированная инструкция x86_64 (CR4.TSD=0 по умолчанию).
    // НЕ является сериализующей: CPU вправе переставить соседние инструкции
    // относительно RDTSC. Для приближённого тайминга кадров (35 FPS watchdog)
    // это допустимо; для точных замеров нужен LFENCE;RDTSC или RDTSCP.
    unsafe {
        asm!(
            "rdtsc",
            out("eax") lo,
            out("edx") hi,
            options(nomem, nostack),
        );
    }
    ((hi as u64) << 32) | lo as u64
}

// ============================================================
// Публичный API
// ============================================================

/// Инициализирует watchdog — вызывать в начале doom::run().
pub fn init() {
    // SAFETY: LAST_FRAME_TSC/FRAME_COUNT — static mut u64; вызывается один раз
    // до старта game loop. Единственный поток — гонок нет.
    unsafe {
        LAST_FRAME_TSC = rdtsc();
        FRAME_COUNT    = 0;
    }
}

/// «Кормит» watchdog — вызывать каждый кадр ПОСЛЕ check().
pub fn pet() {
    // SAFETY: аналогично init() — однопоточный доступ из game loop.
    // pet() должен вызываться ПОСЛЕ check(), чтобы check() мерял elapsed
    // с предыдущего кадра, а не с момента pet() в текущем кадре.
    unsafe {
        LAST_FRAME_TSC = rdtsc();
        FRAME_COUNT   += 1;
    }
}

/// Проверяет watchdog. Если кадры зависли — перезагрузка.
/// Вызывать ПЕРЕД pet() чтобы elapsed = время с предыдущего кадра.
pub fn check() {
    unsafe {
        let now     = rdtsc();
        let elapsed = now.wrapping_sub(LAST_FRAME_TSC);
        if elapsed > TIMEOUT_TICKS {
            crate::serial_println!("[DOOM] Таймаут вотчдога! elapsed={} > {}", elapsed, TIMEOUT_TICKS);
            crate::doom::reboot_from_doom();
        }
    }
}

/// Блокирует до конца текущего кадра (синхронизация ~35 FPS).
/// Возвращает управление когда прошло FRAME_TICKS с момента pet().
pub fn wait_frame() {
    // SAFETY: читаем LAST_FRAME_TSC — записывается только pet() в том же потоке.
    // hlt() допустим т.к. прерывания таймера активны (IRQ0 от PIC); без них
    // hlt зависнет — в этом ядре IRQ0 настроен через blog_os PIC.
    unsafe {
        let frame_start = LAST_FRAME_TSC;
        loop {
            let now = rdtsc();
            if now.wrapping_sub(frame_start) >= FRAME_TICKS {
                break;
            }
            x86_64::instructions::hlt();
        }
    }
}

/// Количество кадров с начала сессии (для статистики)
pub fn frame_count() -> u64 {
    // SAFETY: чтение u64 — атомарно на x86_64 (выровненный доступ).
    unsafe { FRAME_COUNT }
}
