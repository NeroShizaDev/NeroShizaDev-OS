// Аппаратный генератор случайных чисел из теплового шума CPU
// Использует инструкцию RDRAND (Intel Ivy Bridge+ / AMD Zen+)

use core::arch::asm;

/// Проверяет поддержку RDRAND через CPUID (bit 30 of ECX, leaf 1)
pub fn is_supported() -> bool {
    let ecx: u32;
    // SAFETY: CPUID — сериализующая инструкция, доступна без привилегий на x86_64.
    // push/pop rbx нужен т.к. компилятор использует rbx для PIC; CPUID его клоббает.
    // out("eax")/out("edx") — явно объявляем клоббер, чтобы компилятор не
    // переиспользовал эти регистры. Инструкция безопасна на любом x86_64 CPU.
    unsafe {
        asm!(
            "push rbx",
            "mov eax, 1",
            "cpuid",
            "pop rbx",
            out("ecx") ecx,
            out("eax") _,
            out("edx") _,
        );
    }
    ecx & (1 << 30) != 0
}

/// Генерирует 64-битное случайное число из теплового шума CPU
/// Возвращает None если RDRAND не сработал после нескольких попыток
pub fn random_u64() -> Option<u64> {
    let mut val: u64;
    let mut ok: u8;

    // Пробуем 10 раз (RDRAND может временно не иметь энтропии)
    for _ in 0..10 {
        // SAFETY: RDRAND — аппаратный RNG (Intel Ivy Bridge+, AMD Zen+).
        // Доступность проверена is_supported() (CPUID.01H:ECX[30]).
        // CF=1 → успех; CF=0 → энтропии нет, повторяем.
        // nomem/nostack: инструкция не читает/пишет память и не трогает RSP.
        // Intel SDM Vol. 1 §7.3.17 — retry loop рекомендован производителем.
        unsafe {
            asm!(
                "rdrand {val}",
                "setc {ok}",
                val = out(reg) val,
                ok = out(reg_byte) ok,
            );
        }
        if ok != 0 {
            return Some(val);
        }
    }
    None
}

/// Генерирует число в диапазоне [0, max)
pub fn random_range(max: u64) -> u64 {
    match random_u64() {
        Some(val) => val % max,
        None => 0,
    }
}

/// Демо: показывает случайные числа
pub fn demo() {
    crate::locale::print_localized_line(
        crate::kernel_messages::current(crate::kernel_messages::UiText::RngHeader),
        0x0E,
    );

    if !is_supported() {
        crate::locale::print_localized_line(
            crate::kernel_messages::current(crate::kernel_messages::UiText::RngNotSupported),
            0x0C,
        );
        return;
    }

    crate::locale::print_localized_line(
        crate::kernel_messages::current(crate::kernel_messages::UiText::RngSupported),
        0x0A,
    );

    for i in 0..4 {
        match random_u64() {
            Some(val) => crate::kernel_messages::print_rng_value(i, val),
            None => crate::kernel_messages::print_rng_error(i),
        }
    }

    // Бросаем кубик
    let dice = random_range(6) + 1;
    crate::kernel_messages::print_rng_dice(dice);

    crate::locale::print_localized_line(
        crate::kernel_messages::current(crate::kernel_messages::UiText::RngDone),
        0x0A,
    );
}
