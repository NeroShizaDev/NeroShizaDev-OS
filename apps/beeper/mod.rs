// ============================================================
// PC SPEAKER (БИПЕР) — Голос Предков
// ============================================================
// Programmable Interval Timer (PIT 8253/8254), канал 2.
// Порт 0x42 = данные канала 2, 0x43 = команда, 0x61 = управление.
// Частота генератора: 1193180 Гц.
// ============================================================

use x86_64::instructions::port::Port;

/// Включает бипер на заданной частоте (Гц).
/// Если freq == 0, выключает звук.
/// Сначала probe_speaker() — если порт не отвечает (0xFF), молча выходим.
pub unsafe fn play(freq: u32) {
    // Правило: опрос порта ПЕРЕД записью.
    // probe_speaker() читает 0x61: 0xFF = устройства нет (USB-only PC).
    if !crate::validator::probe_speaker() {
        return;
    }
    unsafe {
        let mut port_61 = Port::<u8>::new(0x61);
        let mut port_43 = Port::<u8>::new(0x43);
        let mut port_42 = Port::<u8>::new(0x42);

        if freq == 0 {
            // Выключаем спикер (сбрасываем биты 0 и 1 порта 0x61)
            let val = port_61.read() & 0xFC;
            port_61.write(val);
            return;
        }

        // Делитель = 1193180 / freq
        let divisor = 1193180u32 / freq;

        // Программируем PIT канал 2: режим 3 (square wave), бинарный
        port_43.write(0xB6);

        // Посылаем делитель (младший байт, потом старший)
        port_42.write((divisor & 0xFF) as u8);
        port_42.write(((divisor >> 8) & 0xFF) as u8);

        // Включаем спикер (устанавливаем биты 0 и 1 порта 0x61)
        let val = port_61.read() | 0x03;
        port_61.write(val);
    }
}

/// Выключает бипер
pub unsafe fn stop() {
    unsafe {
        play(0);
    }
}

/// 16-нотная гексатоническая гамма (16 полутонов на октаву).
/// f(n) = 440 * 2^(n/16), где n = 0..15.
/// Предвычисленные частоты (целые Гц):
pub static HEX_SCALE: [u32; 16] = [
    440, // 0x0: A4 (базовая)
    459, // 0x1
    479, // 0x2
    500, // 0x3
    523, // 0x4
    545, // 0x5
    569, // 0x6
    594, // 0x7
    620, // 0x8
    647, // 0x9
    676, // 0xA
    705, // 0xB
    736, // 0xC
    768, // 0xD
    802, // 0xE
    837, // 0xF
];

/// Играет одну ноту из 16-нотной гаммы.
/// `note` — индекс 0x0..0xF, `octave_shift` — сдвиг октавы (-2..+2).
pub unsafe fn play_hex_note(note: u8, octave_shift: i8) {
    let idx = (note & 0x0F) as usize;
    let mut freq = HEX_SCALE[idx];

    // Сдвиг октавы: умножаем или делим на 2
    if octave_shift > 0 {
        for _ in 0..octave_shift {
            freq *= 2;
        }
    } else if octave_shift < 0 {
        for _ in 0..(-octave_shift) {
            freq /= 2;
        }
    }

    unsafe { play(freq) };
}

/// Демо: играем восходящую 16-нотную гамму.
/// Каждая нота звучит ~80мс (примерная задержка через busy loop).
pub fn demo_hex_scale() {
    crate::locale::print_localized_line(
        crate::kernel_messages::current(crate::kernel_messages::UiText::BeeperStart),
        0x0E,
    );
    for i in 0..16u8 {
        crate::print!("0x{:X} ", i);
        unsafe { play_hex_note(i, 0) };
        // Задержка ~80мс через busy loop (грубая, зависит от CPU)
        for _ in 0..2_000_000u64 {
            core::hint::spin_loop();
        }
    }
    unsafe { stop() };
    crate::locale::print_localized_line("", 0x0E);
    crate::locale::print_localized_line(
        crate::kernel_messages::current(crate::kernel_messages::UiText::BeeperDone),
        0x0A,
    );
}
