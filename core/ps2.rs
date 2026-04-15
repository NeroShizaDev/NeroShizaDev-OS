// ============================================================
// PS/2 — общие примитивы опроса PS/2 контроллера
// ============================================================
// Устраняет дублирование между doom/input.rs, menger.rs и interrupts.rs.
// Все функции unsafe: прямой доступ к аппаратным I/O-портам.
// ============================================================

use x86_64::instructions::port::Port;

/// Проверяет бит OBF (Output Buffer Full) регистра статуса 0x64.
/// Возвращает true если в буфере 0x60 есть скан-код.
///
/// # Safety
/// Порт 0x64 — PS/2 Status Register. Чтение — read-only, без побочных эффектов.
/// Bare-metal ядро: единственный потребитель PS/2 контроллера.
#[inline(always)]
pub unsafe fn has_scancode() -> bool {
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
    Port::<u8>::new(0x60).read()
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
