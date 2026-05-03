// ============================================================
// PS/2 — общие примитивы опроса PS/2 контроллера
// ============================================================
// Устраняет дублирование между doom/input.rs, menger.rs и interrupts.rs.
// Все функции unsafe: прямой доступ к аппаратным I/O-портам.
// ============================================================

use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use x86_64::instructions::port::Port;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InputOwner {
    Shell = 0,
    Apps = 1,
}

const SCANCODE_QUEUE_SIZE: usize = 64;
static mut SCANCODE_QUEUE: [u8; SCANCODE_QUEUE_SIZE] = [0; SCANCODE_QUEUE_SIZE];
static SCANCODE_HEAD: AtomicUsize = AtomicUsize::new(0);
static SCANCODE_TAIL: AtomicUsize = AtomicUsize::new(0);
static INPUT_OWNER: AtomicU8 = AtomicU8::new(InputOwner::Shell as u8);

const DEBUG_EVENT_QUEUE_SIZE: usize = 512;
const DEBUG_SRC_IRQ: u8 = 1;
const DEBUG_SRC_READ_QUEUE: u8 = 2;
const DEBUG_SRC_READ_PORT: u8 = 3;
static mut DEBUG_SCANCODE_QUEUE: [u8; DEBUG_EVENT_QUEUE_SIZE] = [0; DEBUG_EVENT_QUEUE_SIZE];
static mut DEBUG_OWNER_QUEUE: [u8; DEBUG_EVENT_QUEUE_SIZE] = [0; DEBUG_EVENT_QUEUE_SIZE];
static mut DEBUG_SRC_QUEUE: [u8; DEBUG_EVENT_QUEUE_SIZE] = [0; DEBUG_EVENT_QUEUE_SIZE];
static mut DEBUG_TSC_QUEUE: [u64; DEBUG_EVENT_QUEUE_SIZE] = [0; DEBUG_EVENT_QUEUE_SIZE];
static DEBUG_HEAD: AtomicUsize = AtomicUsize::new(0);
static DEBUG_TAIL: AtomicUsize = AtomicUsize::new(0);
static DEBUG_DROPPED: AtomicUsize = AtomicUsize::new(0);
static mut DEBUG_EXT_PENDING: bool = false;

#[inline]
fn owner_name(owner: InputOwner) -> &'static str {
    match owner {
        InputOwner::Shell => "Shell",
        InputOwner::Apps => "Apps",
    }
}

#[inline]
fn debug_src_name(src: u8) -> &'static str {
    match src {
        DEBUG_SRC_IRQ => "irq",
        DEBUG_SRC_READ_QUEUE => "read_queue",
        DEBUG_SRC_READ_PORT => "read_port",
        _ => "unknown",
    }
}

#[inline]
fn read_tsc() -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        core::arch::asm!(
            "rdtsc",
            out("eax") lo,
            out("edx") hi,
            options(nostack, nomem)
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

#[inline]
fn debug_key_name(scancode: u8, extended: bool) -> &'static str {
    let code = scancode & 0x7F;
    match (extended, code) {
        (true, 0x1C) => "kp-enter",
        (true, 0x48) => "up",
        (true, 0x50) => "down",
        (true, 0x4B) => "left",
        (true, 0x4D) => "right",
        (false, 0x01) => "esc",
        (false, 0x1C) => "enter",
        (false, 0x39) => "space",
        (false, 0x10) => "q",
        (false, 0x11) => "w",
        (false, 0x12) => "e",
        (false, 0x19) => "p",
        (false, 0x1E) => "a",
        (false, 0x1F) => "s",
        (false, 0x20) => "d",
        (false, 0x48) => "kp8",
        (false, 0x50) => "kp2",
        (false, 0x4B) => "kp4",
        (false, 0x4D) => "kp6",
        _ => "other",
    }
}

#[inline]
fn push_debug_event(scancode: u8, owner: InputOwner, src: u8) {
    let head = DEBUG_HEAD.load(Ordering::Relaxed);
    let next = (head + 1) % DEBUG_EVENT_QUEUE_SIZE;
    let tail = DEBUG_TAIL.load(Ordering::Acquire);
    if next == tail {
        DEBUG_DROPPED.fetch_add(1, Ordering::Relaxed);
        return;
    }

    unsafe {
        DEBUG_SCANCODE_QUEUE[head] = scancode;
        DEBUG_OWNER_QUEUE[head] = owner as u8;
        DEBUG_SRC_QUEUE[head] = src;
        DEBUG_TSC_QUEUE[head] = read_tsc();
    }
    DEBUG_HEAD.store(next, Ordering::Release);
}

#[inline]
fn pop_debug_event() -> Option<(u8, InputOwner, u8, u64)> {
    let tail = DEBUG_TAIL.load(Ordering::Acquire);
    let head = DEBUG_HEAD.load(Ordering::Acquire);
    if tail == head {
        return None;
    }

    let scancode = unsafe { DEBUG_SCANCODE_QUEUE[tail] };
    let owner = match unsafe { DEBUG_OWNER_QUEUE[tail] } {
        1 => InputOwner::Apps,
        _ => InputOwner::Shell,
    };
    let src = unsafe { DEBUG_SRC_QUEUE[tail] };
    let tsc = unsafe { DEBUG_TSC_QUEUE[tail] };
    DEBUG_TAIL.store((tail + 1) % DEBUG_EVENT_QUEUE_SIZE, Ordering::Release);
    Some((scancode, owner, src, tsc))
}

#[inline]
pub fn set_input_owner(owner: InputOwner) {
    INPUT_OWNER.store(owner as u8, Ordering::Release);
    crate::serial_println!("[PS2][OWNER] {}", owner_name(owner));
}

#[inline]
pub fn input_owner() -> InputOwner {
    match INPUT_OWNER.load(Ordering::Acquire) {
        1 => InputOwner::Apps,
        _ => InputOwner::Shell,
    }
}

#[inline]
pub fn clear_scancode_queue() {
    SCANCODE_HEAD.store(0, Ordering::Release);
    SCANCODE_TAIL.store(0, Ordering::Release);
    crate::serial_println!("[PS2][QUEUE] cleared owner={}", owner_name(input_owner()));
}

#[inline]
pub fn trace_irq_scancode(scancode: u8, owner: InputOwner) {
    push_debug_event(scancode, owner, DEBUG_SRC_IRQ);
}

pub fn flush_debug_serial(limit: usize) {
    let dropped = DEBUG_DROPPED.swap(0, Ordering::AcqRel);
    if dropped != 0 {
        crate::serial_println!("[PS2][DROP] debug_events={}", dropped);
    }

    let mut emitted = 0usize;
    while emitted < limit {
        let Some((scancode, owner, src, irq_tsc)) = pop_debug_event() else {
            break;
        };

        if scancode == 0xE0 {
            unsafe {
                DEBUG_EXT_PENDING = true;
            }
            continue;
        }

        let extended = unsafe {
            let pending = DEBUG_EXT_PENDING;
            DEBUG_EXT_PENDING = false;
            pending
        };
        let release = scancode & 0x80 != 0;
        crate::serial_println!(
            "[PS2][SC] irq_tsc={:#x} src={} owner={} raw=0x{:02X} key={} release={} ext={}",
            irq_tsc,
            debug_src_name(src),
            owner_name(owner),
            scancode,
            debug_key_name(scancode, extended),
            if release { 1 } else { 0 },
            if extended { 1 } else { 0 }
        );
        emitted += 1;
    }
}

#[inline]
pub fn push_scancode_from_irq(scancode: u8) {
    let head = SCANCODE_HEAD.load(Ordering::Relaxed);
    let next = (head + 1) % SCANCODE_QUEUE_SIZE;
    let tail = SCANCODE_TAIL.load(Ordering::Acquire);
    if next == tail {
        return;
    }

    unsafe {
        SCANCODE_QUEUE[head] = scancode;
    }
    SCANCODE_HEAD.store(next, Ordering::Release);
}

#[inline]
fn pop_scancode_from_queue() -> Option<u8> {
    let tail = SCANCODE_TAIL.load(Ordering::Acquire);
    let head = SCANCODE_HEAD.load(Ordering::Acquire);
    if tail == head {
        return None;
    }

    let sc = unsafe { SCANCODE_QUEUE[tail] };
    SCANCODE_TAIL.store((tail + 1) % SCANCODE_QUEUE_SIZE, Ordering::Release);
    Some(sc)
}

/// Проверяет бит OBF (Output Buffer Full) регистра статуса 0x64.
/// Возвращает true если в буфере 0x60 есть скан-код.
///
/// # Safety
/// Порт 0x64 — PS/2 Status Register. Чтение — read-only, без побочных эффектов.
/// Bare-metal ядро: единственный потребитель PS/2 контроллера.
#[inline(always)]
pub unsafe fn has_scancode() -> bool {
    if SCANCODE_TAIL.load(Ordering::Acquire) != SCANCODE_HEAD.load(Ordering::Acquire) {
        return true;
    }
    Port::<u8>::new(0x64).read() & 0x01 != 0
}

/// Читает один скан-код из порта 0x60 (без блокировки).
/// Вызывать только после has_scancode() == true.
///
/// # Safety
/// Порт 0x60 — PS/2 Data Register. Чтение сдвигает внутренний FIFO контроллера —
/// аппаратный побочный эффект (x86_64 crate, RFC #2873).
/// Вызывать только когда OBF=1 (проверено has_scancode).
#[inline(always)]
pub unsafe fn read_scancode() -> u8 {
    if let Some(sc) = pop_scancode_from_queue() {
        push_debug_event(sc, input_owner(), DEBUG_SRC_READ_QUEUE);
        return sc;
    }
    let sc = Port::<u8>::new(0x60).read();
    push_debug_event(sc, input_owner(), DEBUG_SRC_READ_PORT);
    sc
}

/// Инициализирует PS/2 контроллер: дренирует буфер и разрешает сканирование клавиатуры.
///
/// QEMU включает сканирование по умолчанию, VirtualBox оставляет PS/2 порт
/// в отключённом состоянии после BIOS (команда 0xAD без последующего 0xAE).
/// Явная команда 0xAE (Enable First PS/2 Port) исправляет ввод в VirtualBox.
///
/// # Safety
/// Прямой доступ к I/O-портам 0x60/0x64. Вызывать до x86_64::instructions::interrupts::enable().
pub unsafe fn init() {
    let mut status_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);

    // Дренаж выходного буфера (OBF = bit 0) — убираем мусор от BIOS
    for _ in 0..16u8 {
        if status_port.read() & 0x01 == 0 {
            break;
        }
        let _ = data_port.read();
    }

    // Ждём освобождения входного буфера (IBF = bit 1) перед отправкой команды
    for _ in 0..0xFFFFu32 {
        if status_port.read() & 0x02 == 0 {
            break;
        }
    }

    // 0xAE = Enable First PS/2 Port (клавиатура)
    status_port.write(0xAEu8);

    crate::serial_println!("[PS2][INIT] keyboard scanning enabled (cmd 0xAE)");
}

/// Ожидает вертикального гашения (VBlank) через VGA Input Status 1 (0x3DA).
///
/// # Safety
/// Порт 0x3DA — VGA Input Status 1 (read-only). Чтение сбрасывает AC flip-flop
/// как побочный эффект — это нормально (AC переинициализируется перед следующей
/// записью в 0x3C0). Используется для frame-sync без разрывов экрана.
#[inline(always)]
pub unsafe fn wait_vblank() {
    let mut p: Port<u8> = Port::new(0x3DA);
    while p.read() & 0x08 != 0 {}
    while p.read() & 0x08 == 0 {}
}
