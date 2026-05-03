// ============================================================
// NHS RUNTIME — интерпретатор NeroShizaScript
// ============================================================
//
// Запускает CODE-секцию из полного .nhs blob, сохранённого в слоте.
// CODE — ASCII NeroShizaScript, одна инструкция на строку.
//
// Стек: i32[32]
// Инструкции:
//   PUSH <n>         — положить i32 на стек
//   POP              — убрать вершину
//   DUP              — дублировать вершину
//   ADD/SUB/MUL/DIV  — арифметика: pop b, pop a, push a OP b
//   PRINT            — вывести вершину стека как число
//   PRINTS <text>    — вывести строку (остаток строки)
//   HALT             — завершить скрипт
//   # ...            — комментарий (игнорируется)
// ============================================================

use super::{header, slots};
use x86_64::instructions::hlt;

const STACK_DEPTH: usize = 32;

pub enum RunResult {
    Ok,
    Halted,
    StackOverflow,
    StackUnderflow,
    DivByZero,
    NotScript, // FLAG_HAS_SCRIPT не установлен или плохой заголовок
    NativeUnsupported,
    EmptySlot,
}

impl RunResult {
    pub fn message(&self) -> &'static str {
        match self {
            Self::Ok => "Script completed.",
            Self::Halted => "Script halted normally.",
            Self::StackOverflow => "Error: stack overflow.",
            Self::StackUnderflow => "Error: stack underflow.",
            Self::DivByZero => "Error: division by zero.",
            Self::NotScript => "Error: not a script package.",
            Self::NativeUnsupported => "Error: native NHS packages are not supported yet.",
            Self::EmptySlot => "Error: slot is empty.",
        }
    }
}

/// Запустить NeroShizaScript из указанного слота.
pub fn run_slot(slot: usize) -> RunResult {
    // Читаем полный NHS blob из слота
    let data = match slots::read_slot(slot) {
        Some(d) => d,
        None => return RunResult::EmptySlot,
    };

    // Парсим заголовок
    let hdr = match header::parse_header(data) {
        Some(h) => h,
        None => return RunResult::NotScript,
    };
    if hdr.flags & header::FLAG_HAS_SCRIPT == 0 {
        if hdr.flags & header::FLAG_HAS_NATIVE != 0 {
            return RunResult::NativeUnsupported;
        }
        return RunResult::NotScript;
    }

    // Получаем CODE-секцию
    let code_start = hdr.code_offset as usize;
    let code_end = code_start.saturating_add(hdr.code_size as usize);
    if code_end > data.len() || code_start >= code_end {
        return RunResult::NotScript;
    }
    let code = &data[code_start..code_end];

    // Получаем имя приложения из манифеста
    let manifest = header::parse_manifest(data, hdr.manifest_offset);

    // Очищаем экран и выводим баннер
    crate::fb_buffer::clear_screen();
    crate::println!("╔══════════════════════════════════════════════════╗");
    if let Some(ref m) = manifest {
        let ne = m.app_name.iter().position(|&b| b == 0).unwrap_or(32);
        let name = core::str::from_utf8(&m.app_name[..ne]).unwrap_or("?");
        let v = &m.version;
        crate::println!("  NHS: {}  v{}.{}.{}", name, v[0], v[1], v[2]);
    } else {
        crate::println!("  NHS: slot {}", slot);
    }
    crate::println!("╚══════════════════════════════════════════════════╝");
    crate::println!();

    // Запускаем интерпретатор
    let result = interpret(code);

    // Выводим итог
    crate::println!();
    crate::println!("── {} ──", result.message());
    crate::println!("Press any key to return...");
    wait_key();

    result
}

fn interpret(code: &[u8]) -> RunResult {
    let mut stack = [0i32; STACK_DEPTH];
    let mut sp: usize = 0;
    let mut pos: usize = 0;

    while pos < code.len() {
        // Читаем одну строку
        let ls = pos;
        while pos < code.len() && code[pos] != b'\n' {
            pos += 1;
        }
        let mut line = &code[ls..pos];
        if pos < code.len() {
            pos += 1;
        } // пропускаем '\n'

        // Убираем '\r' в конце
        if line.last() == Some(&b'\r') {
            line = &line[..line.len() - 1];
        }

        // Пропускаем пустые строки и комментарии
        if line.is_empty() || line[0] == b'#' {
            continue;
        }

        let (op, arg) = split_word(line);

        match op {
            b"PUSH" => {
                if sp >= STACK_DEPTH {
                    return RunResult::StackOverflow;
                }
                stack[sp] = parse_i32(arg);
                sp += 1;
            }
            b"POP" => {
                if sp == 0 {
                    return RunResult::StackUnderflow;
                }
                sp -= 1;
            }
            b"DUP" => {
                if sp == 0 {
                    return RunResult::StackUnderflow;
                }
                if sp >= STACK_DEPTH {
                    return RunResult::StackOverflow;
                }
                stack[sp] = stack[sp - 1];
                sp += 1;
            }
            b"ADD" => {
                if sp < 2 {
                    return RunResult::StackUnderflow;
                }
                let b = stack[sp - 1];
                let a = stack[sp - 2];
                stack[sp - 2] = a.wrapping_add(b);
                sp -= 1;
            }
            b"SUB" => {
                if sp < 2 {
                    return RunResult::StackUnderflow;
                }
                let b = stack[sp - 1];
                let a = stack[sp - 2];
                stack[sp - 2] = a.wrapping_sub(b);
                sp -= 1;
            }
            b"MUL" => {
                if sp < 2 {
                    return RunResult::StackUnderflow;
                }
                let b = stack[sp - 1];
                let a = stack[sp - 2];
                stack[sp - 2] = a.wrapping_mul(b);
                sp -= 1;
            }
            b"DIV" => {
                if sp < 2 {
                    return RunResult::StackUnderflow;
                }
                let b = stack[sp - 1];
                let a = stack[sp - 2];
                if b == 0 {
                    return RunResult::DivByZero;
                }
                stack[sp - 2] = a / b;
                sp -= 1;
            }
            b"PRINT" => {
                if sp == 0 {
                    return RunResult::StackUnderflow;
                }
                crate::println!("{}", stack[sp - 1]);
            }
            b"PRINTS" => {
                // arg — остаток строки после PRINTS (пропускаем ведущий пробел)
                let s = if arg.first() == Some(&b' ') {
                    &arg[1..]
                } else {
                    arg
                };
                if let Ok(text) = core::str::from_utf8(s) {
                    crate::println!("{}", text);
                }
            }
            b"HALT" => {
                return RunResult::Halted;
            }
            _ => { /* неизвестная инструкция — пропускаем */ }
        }
    }

    RunResult::Ok
}

/// Разбивает строку на первое слово и остаток.
fn split_word(line: &[u8]) -> (&[u8], &[u8]) {
    let mut i = 0;
    while i < line.len() && line[i] != b' ' && line[i] != b'\t' {
        i += 1;
    }
    let word = &line[..i];
    while i < line.len() && (line[i] == b' ' || line[i] == b'\t') {
        i += 1;
    }
    (word, &line[i..])
}

/// Парсит i32 из байтов ASCII (нет std::str::parse).
fn parse_i32(s: &[u8]) -> i32 {
    let (neg, s) = if s.first() == Some(&b'-') {
        (true, &s[1..])
    } else {
        (false, s)
    };
    let mut n: i32 = 0;
    for &b in s {
        if b < b'0' || b > b'9' {
            break;
        }
        n = n.wrapping_mul(10).wrapping_add((b - b'0') as i32);
    }
    if neg { -n } else { n }
}

fn wait_key() {
    unsafe {
        loop {
            if crate::ps2::has_scancode() {
                if crate::ps2::read_scancode() & 0x80 == 0 {
                    return;
                }
            }
            hlt();
        }
    }
}
