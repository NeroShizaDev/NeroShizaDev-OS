// Аппаратный генератор случайных чисел из теплового шума CPU
// Использует инструкцию RDRAND (Intel Ivy Bridge+ / AMD Zen+)

use core::arch::asm;

/// Проверяет поддержку RDRAND через CPUID (bit 30 of ECX, leaf 1)
pub fn is_supported() -> bool {
    let ecx: u32;
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
    crate::println!("--- RNG (тепловой шум CPU) ---");

    if !is_supported() {
        crate::println!("RDRAND не поддерживается!");
        return;
    }

    crate::println!("RDRAND: поддерживается");

    for i in 0..4 {
        match random_u64() {
            Some(val) => crate::println!("  rand[{}] = {:#018X}", i, val),
            None => crate::println!("  rand[{}] = ОШИБКА", i),
        }
    }

    // Бросаем кубик
    let dice = random_range(6) + 1;
    crate::println!("  Кубик: {}", dice);

    crate::println!("--- RNG OK ---");
}
