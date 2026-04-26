// ============================================================
// DOOM MOD — Публичный API: init() + run()
// ============================================================
// Архитектура:
//   init(wad)  → инициализирует кучу, загружает WAD
//   run()      → Mode 13h + Doom Fire эффект + ожидание ESC
//
// Вызов из обработчика прерывания клавиатуры (lib.rs handle_keyboard_input).
// Прерывания отключены во время выполнения — используем прямой опрос PS/2.
// При выходе: восстанавливаем текстовый режим + шрифт + экран.
// ============================================================

pub mod chaos;
pub mod input;
pub mod stubs;
pub mod vga_graphics;
pub mod wad;
pub mod watchdog;

// ============================================================
// Публичный API
// ============================================================

/// Инициализирует Doom-подсистему.
/// * `wad_data` — содержимое doom1.wad (include_bytes! или None = demo mode)
///
/// Вызывать до run(). Безопасно вызывать повторно — init_heap() идемпотентна,
/// повторный вызов не трогает существующие аллокации.
pub fn init(wad_data: &'static [u8]) {
    crate::serial_println!("[DOOM] Инициализация: размер WAD = {}", wad_data.len());
    stubs::init_heap();

    if wad_data.len() >= 12 {
        wad::load(wad_data);
        crate::serial_println!("[DOOM] WAD загружен");
    } else {
        crate::serial_println!("[DOOM] WAD нет (демо-режим)");
    }
}

/// Запускает Doom-сессию.
/// Переключает VGA в Mode 13h, запускает Doom Fire,
/// ждёт ESC → восстанавливает текстовый режим → возвращается.
///
/// # Safety
/// Вызывается из контекста прерывания клавиатуры (ISR). Прерывания CPU
/// заблокированы, PIC EOI ещё не отправлен — используем прямой опрос PS/2
/// (порт 0x60). VGA-регистры и 0xA0000 должны быть identity-mapped.
pub fn run() {
    crate::serial_println!("[DOOM] Запуск");
    unsafe {
        // Сохраняем ВСЕ 256 слотов шрифта plane 2 перед Mode 13h.
        // chain-4 в Mode 13h перезапишет plane 2 данными огня — без этого
        // box-drawing символы (0xBA..0xCD и т.п.) будут испорчены после выхода.
        vga_graphics::save_font_plane();
        crate::serial_println!("[DOOM] Переключение в Mode 13h...");
        vga_graphics::set_mode_13h();
        crate::serial_println!("[DOOM] Палитра огня...");
        vga_graphics::set_fire_palette();
        vga_graphics::clear_fb(0);
        vga_graphics::fire_set_mode(vga_graphics::FireMode::Classic);
        vga_graphics::fire_init();
        crate::serial_println!(
            "[DOOM] Fire {}",
            vga_graphics::fire_mode_name(vga_graphics::fire_mode())
        );
        watchdog::init();
        input::set_active(true);
        crate::serial_println!("[DOOM] Главный цикл запущен");

        if let Some(_reader) = wad::reader() {
            for x in 0..vga_graphics::SCREEN_W {
                vga_graphics::fire_ignite_bottom(x, (vga_graphics::FIRE_LEVELS - 1) as u8);
            }
        }

        loop {
            if input::poll() {
                break;
            }

            if input::take_mode_1() {
                vga_graphics::fire_set_mode(vga_graphics::FireMode::Classic);
                crate::serial_println!(
                    "[DOOM] Fire {}",
                    vga_graphics::fire_mode_name(vga_graphics::fire_mode())
                );
                watchdog::log_profile("mode-1");
            }
            if input::take_mode_2() {
                vga_graphics::fire_set_mode(vga_graphics::FireMode::Dual);
                crate::serial_println!(
                    "[DOOM] Fire {}",
                    vga_graphics::fire_mode_name(vga_graphics::fire_mode())
                );
                watchdog::log_profile("mode-2");
            }
            if input::take_mode_3() {
                vga_graphics::fire_set_mode(vga_graphics::FireMode::Inferno);
                crate::serial_println!(
                    "[DOOM] Fire {}",
                    vga_graphics::fire_mode_name(vga_graphics::fire_mode())
                );
                watchdog::log_profile("mode-3");
            }
            if input::take_mode_4() {
                vga_graphics::fire_set_mode(vga_graphics::FireMode::FpuNoise);
                crate::serial_println!(
                    "[DOOM] Fire {}",
                    vga_graphics::fire_mode_name(vga_graphics::fire_mode())
                );
                watchdog::log_profile("mode-4");
            }
            if input::take_mode_5() {
                vga_graphics::fire_set_mode(vga_graphics::FireMode::WindLeft);
                crate::serial_println!(
                    "[DOOM] Fire {}",
                    vga_graphics::fire_mode_name(vga_graphics::fire_mode())
                );
                watchdog::log_profile("mode-5");
            }
            if input::take_mode_6() {
                vga_graphics::fire_set_mode(vga_graphics::FireMode::WindRight);
                crate::serial_println!(
                    "[DOOM] Fire {}",
                    vga_graphics::fire_mode_name(vga_graphics::fire_mode())
                );
                watchdog::log_profile("mode-6");
            }
            if input::take_fpu_toggle() {
                vga_graphics::fire_toggle_fpu();
                crate::serial_println!(
                    "[DOOM] FPU {}",
                    if vga_graphics::fire_fpu_enabled() {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
                watchdog::log_profile("toggle-fpu");
            }

            if input::is_fire() {
                for x in 0..vga_graphics::SCREEN_W {
                    chaos_ignite(x);
                }
            }

            vga_graphics::fire_update();
            vga_graphics::fire_render();

            watchdog::check();
            watchdog::pet();
            watchdog::wait_frame();
        }

        crate::serial_println!("[DOOM] Выход, восстановление текстового режима...");
        input::reset();
        input::set_active(false);
        stubs::reset_heap();
        crate::serial_println!("[DOOM] Выход завершён (cleanup в on_destroy)");
    }
}

/// Вызывается ActivityManager через lifecycle Destroy после выхода из run().
/// Выполняет 4 шага Ритуала Восстановления Mode 3 + кириллица + badge.
pub fn on_destroy() {
    crate::serial_println!("[DOOM] on_destroy: восстановление текстового режима (4 шага)");
    unsafe {
        // Шаг 1-4: регистры + DAC-палитра + ASCII 0-127 + очистка 0xB8000
        crate::vga_hw::restore_text_mode();
        // Восстанавливаем ВСЕ 256 слотов шрифта из бэкапа:
        // ASCII 0-127 (наш кастомный), кириллица 128-191, box-drawing 192-255.
        // Без этого символы ╠═╚╝║ (>191) остаются испорченными огнём doom.
        vga_graphics::restore_font_plane();
        // Восстанавливаем locale badge (флаг локали в правом углу)
        crate::locale::draw_locale_badge();
    }
    crate::serial_println!("[DOOM] on_destroy завершён");
}

// ============================================================
// Вспомогательные функции
// ============================================================

/// Усиливает огонь в колонке `x` нижней строки FIRE_BUF.
/// Небольшой шум от RDRAND делает огонь живее при удержании FIRE.
#[inline]
fn chaos_ignite(x: usize) {
    // Случайный шум: ±0..1 от максимальной яркости
    let noise = chaos::rand_u8(2) as usize;
    let bright = (vga_graphics::FIRE_LEVELS - 1).saturating_sub(noise);
    // fire_ignite_bottom пишет в FIRE_BUF (не в VGA напрямую),
    // чтобы запись не затиралась fire_render() в этом же кадре.
    vga_graphics::fire_ignite_bottom(x, bright as u8);
}

/// Перезагрузка из контекста Doom (вызывается watchdog при зависании).
/// Восстанавливает текстовый режим перед перезагрузкой.
pub fn reboot_from_doom() -> ! {
    crate::serial_println!("[DOOM] Аварийная перезагрузка: восстановление текстового режима");
    unsafe {
        vga_graphics::restore_text_mode();
        crate::vga_unicode::load_static_glyphs();
        let mut port = x86_64::instructions::port::Port::<u8>::new(0x64);
        port.write(0xFE);
    }
    loop {
        x86_64::instructions::hlt();
    }
}
