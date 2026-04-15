// FPU (x87) — инициализация и математика через inline assembly
// Rust компилирует с soft-float, поэтому все float-операции через asm!

use core::arch::asm;

/// Инициализация x87 FPU
/// Настраивает CR0: убирает эмуляцию, включает мониторинг и нативные ошибки
pub fn init() {
    unsafe {
        let mut cr0: u64;
        asm!("mov {}, cr0", out(reg) cr0);
        cr0 &= !(1 << 2); // EM = 0 (не эмулировать FPU)
        cr0 |= 1 << 1;    // MP = 1 (мониторинг сопроцессора)
        cr0 |= 1 << 5;    // NE = 1 (нативные ошибки FPU)
        asm!("mov cr0, {}", in(reg) cr0);
        asm!("fninit");    // Инициализация x87 без проверки ошибок
    }
}

/// Вычисляет log2(x) через x87 инструкцию fyl2x
/// fyl2x: ST(1) * log2(ST(0)) — мы ставим y=1, x=наше значение
pub fn log2(x: u64) -> i64 {
    // x передаём как целое, конвертим внутри FPU
    let result: i64;
    unsafe {
        asm!(
            "fld1",           // ST(0) = 1.0 (это будет y)
            "push {x}",
            "fild qword ptr [rsp]", // ST(0) = x (целое -> float), ST(1) = 1.0
            "add rsp, 8",
            "fyl2x",         // ST(0) = 1.0 * log2(x)
            "push 0",
            "fistp qword ptr [rsp]", // конвертим обратно в целое
            "pop {out}",
            x = in(reg) x,
            out = out(reg) result,
        );
    }
    result
}

/// Вычисляет sqrt(x) через x87
pub fn sqrt(x: u64) -> u64 {
    let result: u64;
    unsafe {
        asm!(
            "push {x}",
            "fild qword ptr [rsp]",  // загружаем x как float
            "fsqrt",                  // ST(0) = sqrt(x)
            "fistp qword ptr [rsp]",  // обратно в целое
            "pop {out}",
            x = in(reg) x,
            out = out(reg) result,
        );
    }
    result
}

/// Простая энтропия Шеннона для блока байтов
/// H = -sum(p_i * log2(p_i)) для каждого уникального байта
/// Возвращает энтропию * 1000 (чтобы показать дробную часть без float в println)
pub fn shannon_entropy(data: &[u8]) -> u64 {
    if data.is_empty() {
        return 0;
    }

    // Считаем частоты каждого байта
    let mut freq: [u32; 256] = [0; 256];
    for &byte in data {
        freq[byte as usize] += 1;
    }

    let len = data.len() as u64;

    // H = -sum(count/len * log2(count/len))
    //   = log2(len) - (1/len) * sum(count * log2(count))
    // Используем x87 для вычисления
    let result: i64;
    unsafe {
        asm!(
            // Начинаем: обнуляем FPU стек
            "fldz",              // ST(0) = 0.0
            "fstp st(0)",        // очищаем
        );

        let mut sum_part: i64 = 0;
        for i in 0..256 {
            let count = freq[i];
            if count > 0 {
                // count * log2(count) через x87
                let part: i64;
                asm!(
                    "fld1",                       // ST(0) = 1.0
                    "push {c}",
                    "fild qword ptr [rsp]",       // ST(0) = count
                    "add rsp, 8",
                    "fyl2x",                      // ST(0) = 1.0 * log2(count)
                    "push {c}",
                    "fild qword ptr [rsp]",       // ST(0) = count, ST(1) = log2(count)
                    "add rsp, 8",
                    "fmulp",                      // ST(0) = count * log2(count)
                    "push 0",
                    "fistp qword ptr [rsp]",
                    "pop {out}",
                    c = in(reg) count as u64,
                    out = out(reg) part,
                );
                sum_part += part;
            }
        }

        // log2(len) * 1000
        let log2_len: i64;
        asm!(
            "fld1",
            "push {n}",
            "fild qword ptr [rsp]",
            "add rsp, 8",
            "fyl2x",                // ST(0) = log2(len)
            // умножаем на 1000
            "push {k}",
            "fild qword ptr [rsp]",
            "add rsp, 8",
            "fmulp",               // ST(0) = log2(len) * 1000
            "push 0",
            "fistp qword ptr [rsp]",
            "pop {out}",
            n = in(reg) len,
            k = in(reg) 1000u64,
            out = out(reg) log2_len,
        );

        // sum_part * 1000 / len
        let scaled_sum = (sum_part * 1000) / (len as i64);

        // H*1000 = log2(len)*1000 - sum_part*1000/len
        let entropy = log2_len - scaled_sum;
        result = if entropy > 0 { entropy } else { 0 };
    }

    result as u64
}

/// Демо: вычисляет и выводит результаты FPU
pub fn demo() {
    crate::locale::print_localized_line(
        crate::user_messages::current(crate::user_messages::UiText::FpuHeader),
        0x0E,
    );

    // sqrt тесты
    crate::user_messages::print_fpu_sqrt(144, sqrt(144));
    crate::user_messages::print_fpu_sqrt(1_000_000, sqrt(1_000_000));

    // log2 тесты  
    crate::user_messages::print_fpu_log(256, log2(256));
    crate::user_messages::print_fpu_log(1024, log2(1024));

    // Энтропия тестовых данных
    let low_entropy = b"AAAAAAAAAAAAAAAA";  // повторяющиеся — низкая энтропия
    let high_entropy = b"abcdefghijklmnop";  // все разные — высокая энтропия

    let h_low = shannon_entropy(low_entropy);
    let h_high = shannon_entropy(high_entropy);

    crate::user_messages::print_fpu_entropy("AAAA...", h_low / 1000, h_low % 1000);
    crate::user_messages::print_fpu_entropy("abcd...", h_high / 1000, h_high % 1000);
    crate::locale::print_localized_line(
        crate::user_messages::current(crate::user_messages::UiText::FpuDone),
        0x0A,
    );
}
