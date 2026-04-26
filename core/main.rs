#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use x86_64::VirtAddr;
use x86_64::instructions::port::Port;

use bootloader_api::BootInfo;
use bootloader_api::config::{BootloaderConfig, Mapping};

// ================================================================
// КОНФИГУРАЦИЯ ЗАГРУЗЧИКА
// ================================================================
// physical_memory = Dynamic → bootloader маппит всю физическую память
// по произвольному виртуальному смещению (не конфликтует с другими маппингами).
// Смещение доступно через boot_info.physical_memory_offset.
// Мы дополнительно identity-маппим VGA регион (0xA0000-0xBFFFF) вручную,
// чтобы прямые записи в 0xB8000 работали как раньше.
// ================================================================
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

// Регистрируем нашу функцию как точку входа
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

// ================================================================
// ТОЧКА ВХОДА
// ================================================================
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // ПЕРВЫМ ДЕЛОМ — инициализируем кучу (bump-аллокатор в BSS).
    // Без этого любой вызов alloc (Vec, Box, format!) возвращает null →
    // page fault → double fault → triple fault → QEMU [Paused].
    blog_os::apps::games::doom::stubs::init_heap();
    blog_os::serial_println!("[ЯДРО] Куча инициализирована");
    blog_os::trace::record("heap initialized");

    // ============================================================
    // ФАЗА 0: Маппинг VGA-памяти и сброс в текстовый режим
    // ============================================================
    // Bootloader 0.11 с Dynamic-маппингом физической памяти НЕ маппит
    // низкие адреса (0xA0000-0xBFFFF) напрямую. Мы:
    //   1. Читаем physical_memory_offset из boot_info
    //   2. Инициализируем page table mapper
    //   3. Identity-маппим VGA-регион (0xA0000-0xBFFFF)
    //   4. Сбрасываем VGA в текстовый Mode 3 (bootloader оставил VBE)
    // ============================================================
    let phys_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("bootloader did not map physical memory");
    let phys_offset_addr = VirtAddr::new(phys_offset);
    let mut mapper = unsafe { blog_os::memory::init(phys_offset_addr) };
    let mut frame_allocator =
        unsafe { blog_os::memory::BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    blog_os::memory::map_vga_memory(&mut mapper, &mut frame_allocator);
    unsafe {
        blog_os::vga_hw::force_text_mode_3();
    }
    blog_os::serial_println!("[ЯДРО] Фаза 0: VGA замаплена + текстовый режим 3");
    blog_os::trace::record("vga mapped + text mode");

    // ============================================================
    // ФАЗА 1: Ранняя инициализация & Линия жизни
    // serial (макросы serial_print! уже доступны), validator (готов),
    // vga_hw (используется font-загрузчиком), vga_buffer, logo.
    // Цель: видеть вывод даже если следующие фазы упадут.
    // ============================================================
    blog_os::vga_buffer::clear_screen();
    blog_os::logo::show_boot_logo();
    blog_os::locale::print_localized_line(
        blog_os::user_messages::current(blog_os::user_messages::UiText::SystemCheckHeader),
        0x0B,
    );
    // Детектируем VGA и пробуем расширенный режим (90×30).
    // probe_vga() внутри проверяет порт 0x3DA перед записью в CRTC.
    let vga_mode = blog_os::vga_hw::detect_and_switch();
    match blog_os::locale::get_locale() {
        blog_os::kernel_messages::Locale::RuRu => {
            blog_os::locale::print_localized_fmt(0x0E, format_args!("[Фаза 1] VGA: {:?}", vga_mode))
        }
        blog_os::kernel_messages::Locale::EnUs => blog_os::locale::print_localized_fmt(
            0x0E,
            format_args!("[Phase 1] VGA: {:?}", vga_mode),
        ),
        blog_os::kernel_messages::Locale::ArEg => blog_os::locale::print_localized_fmt(
            0x0E,
            format_args!("[المرحلة 1] VGA: {:?}", vga_mode),
        ),
    }
    blog_os::serial_println!("[ЯДРО] Фаза 1: VGA детект завершён, режим: {:?}", vga_mode);
    blog_os::trace::record("vga detect complete");

    // ============================================================
    // ФАЗА 2: Архитектура CPU
    // GDT → IDT → PICS → FPU → прерывания включены.
    // ============================================================
    blog_os::init();
    blog_os::serial_println!("[ЯДРО] Фаза 2: GDT + IDT + PICS + FPU готовы");
    blog_os::trace::record("gdt idt pics fpu ready");
    blog_os::locale::print_boot_status(blog_os::user_messages::current(
        blog_os::user_messages::UiText::Phase2CpuOk,
    ));

    // ============================================================
    // ФАЗА 3: Память
    // Физическая память и куча инициализированы загрузчиком (bootimage).
    // При необходимости здесь будет memory::init(physical_offset).
    // ============================================================
    blog_os::locale::print_boot_status(blog_os::user_messages::current(
        blog_os::user_messages::UiText::Phase3MemoryOk,
    ));
    blog_os::serial_println!("[ЯДРО] Фаза 3: Память ОК");

    // ============================================================
    // ФАЗА 4: Время и Энтропия
    // Сначала validator::probe_cmos() (уже внутри display_status),
    // затем чтение времени, температура CPU, RNG.
    // ============================================================
    blog_os::rtc::display_status();
    blog_os::rtc::display_thermal();
    let risk = blog_os::validator::probe_pre_freeze_risks();
    blog_os::validator::display_pre_freeze_risks(&risk);
    blog_os::trace::record("rtc validator phase done");

    let rng_ok = blog_os::rng::is_supported();
    blog_os::serial_println!(
        "[ЯДРО] Фаза 4: RDRAND {}",
        if rng_ok { "ВКЛ" } else { "ВЫКЛ" }
    );
    blog_os::locale::print_localized_line(
        blog_os::user_messages::current(if rng_ok {
            blog_os::user_messages::UiText::Phase4RngOn
        } else {
            blog_os::user_messages::UiText::Phase4RngOff
        }),
        0x0E,
    );

    // ============================================================
    // ФАЗА 5: Железо и Ввод
    // PS/2 probe → speaker probe → OK-сигнал.
    // ============================================================
    let ps2_ok = blog_os::validator::probe_ps2();
    blog_os::serial_println!("[ЯДРО] Фаза 5: PS/2 {}", if ps2_ok { "OK" } else { "NO" });
    blog_os::trace::record("ps2 probe complete");
    blog_os::validator::display_ps2_probe();

    if blog_os::validator::probe_speaker() {
        blog_os::serial_println!("[ЯДРО] Фаза 5: Спикер ОК");
        blog_os::locale::print_boot_status(blog_os::user_messages::current(
            blog_os::user_messages::UiText::Phase5SpeakerOk,
        ));
        boot_beep();
    } else {
        blog_os::serial_println!("[ЯДРО] Фаза 5: Спикер НЕ НАЙДЕН");
        blog_os::locale::print_localized_line(
            blog_os::user_messages::current(blog_os::user_messages::UiText::Phase5SpeakerFail),
            0x0E,
        );
    }

    // ============================================================
    // ФАЗА 6: Мультиязычность и Рендеринг текста
    // Шрифты → локаль → Unicode-подсистема → баннер OS.
    // ============================================================
    unsafe {
        blog_os::vga_unicode::load_static_glyphs();
    }
    blog_os::locale::draw_locale_badge();
    blog_os::serial_println!("[ЯДРО] Фаза 6: Шрифт + локаль готовы");
    blog_os::trace::record("font + locale ready");
    blog_os::locale::print_phase6_ok();
    print_startup_banner();

    // ============================================================
    // ФАЗА 7: Пространство пользователя — Shell готов
    // Все демо-приложения запускаются через APPS меню.
    // ============================================================
    print_ready_message();
    blog_os::print!("> ");

    blog_os::serial_println!("[ЯДРО] Фаза 7: Шелл готов, включаем прерывания");
    blog_os::trace::record("shell prompt drawn");
    x86_64::instructions::interrupts::enable();
    blog_os::trace::record("interrupts enabled");
    blog_os::hlt_loop()
}

// ================================================================
// Короткий звуковой сигнал — железо инициализировано успешно.
// 880 Гц (A5), ~200 мс через busy-wait на порту 0x80.
// ================================================================
fn boot_beep() {
    unsafe {
        blog_os::beeper::play(880);
        // Задержка ~200 мс: порт 0x80 (POST-диагностический) ≈ 1 мкс/чтение
        let mut p: Port<u8> = Port::new(0x80);
        for _ in 0u32..200_000 {
            core::hint::black_box(p.read());
        }
        blog_os::beeper::stop();
    }
}

// ================================================================
// Баннер ОС с Unicode-статистикой (Фаза 6)
// ================================================================
#[inline]
fn is_arabic_locale() -> bool {
    blog_os::locale::get_locale() == blog_os::kernel_messages::Locale::ArEg
}

fn print_startup_banner() {
    blog_os::locale::print_localized_line(
        blog_os::user_messages::current(blog_os::user_messages::UiText::BootBannerTitle),
        0x0E,
    );
    blog_os::locale::print_localized_line(
        blog_os::user_messages::current(blog_os::user_messages::UiText::BootBannerUnicode),
        0x0E,
    );
    if is_arabic_locale() {
        blog_os::locale::print_localized_fmt(
            0x0E,
            format_args!(
                "الكتل: {} | الخطوط: {} | الرموز: {}",
                blog_os::unicode_blocks::block_count(),
                blog_os::unicode_scripts::script_count(),
                blog_os::unicode_categories::total_defined_chars(),
            ),
        );
        blog_os::locale::print_localized_fmt(
            0x0E,
            format_args!(
                "القاموس: {} أوامر ({} بايت)",
                blog_os::unicode::dict_size(),
                blog_os::unicode::dict_bytes()
            ),
        );
    } else {
        blog_os::locale::print_localized_fmt(
            0x0E,
            format_args!(
                "Blocks: {} | Scripts: {} | Chars: {}",
                blog_os::unicode_blocks::block_count(),
                blog_os::unicode_scripts::script_count(),
                blog_os::unicode_categories::total_defined_chars()
            ),
        );
        blog_os::locale::print_localized_fmt(
            0x0E,
            format_args!(
                "Dictionary: {} intents ({} bytes)",
                blog_os::unicode::dict_size(),
                blog_os::unicode::dict_bytes()
            ),
        );
    }
}

// ================================================================
// Сообщение готовности — Shell активен
// ================================================================
fn print_ready_message() {
    blog_os::locale::print_localized_line(
        blog_os::user_messages::current(blog_os::user_messages::UiText::Phase7ShellReady),
        0x0A,
    );
}

// ================================================================
// Panic handler
// ================================================================
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::serial_println!("[ЯДРО] ПАНИКА: {}", info);
    blog_os::trace::record_fatal("panic handler entered");
    // Рисуем аварийный экран на VGA
    unsafe {
        blog_os::locale::render_panic_screen(blog_os::kernel_messages::KernelEvent::Panic);
        // Текст паники на строке 14 (truncate до 72 символов — ширина рамки)
        {
            struct PanicBuf {
                data: [u8; 72],
                len: usize,
            }
            impl core::fmt::Write for PanicBuf {
                fn write_str(&mut self, s: &str) -> core::fmt::Result {
                    for &b in s.as_bytes() {
                        if self.len >= self.data.len() {
                            break;
                        }
                        self.data[self.len] = b;
                        self.len += 1;
                    }
                    Ok(())
                }
            }
            let mut buf = PanicBuf {
                data: [0u8; 72],
                len: 0,
            };
            let _ = core::fmt::write(&mut buf, format_args!("{}", info.message()));
            if buf.len > 0 {
                if let Ok(msg) = core::str::from_utf8(&buf.data[..buf.len]) {
                    blog_os::locale::write_panic_message(msg);
                }
            }
        }
        // Если есть location — показать файл:строку на строке 13 (внутри рамки)
        if let Some(loc) = info.location() {
            blog_os::locale::write_str_at_vga(loc.file(), 13, 2, 0x0F);
            blog_os::locale::write_str_at_vga(":", 13, loc.file().len().min(68) + 2, 0x0F);
            blog_os::locale::write_hex32_at_vga(loc.line(), 13, loc.file().len().min(68) + 3, 0x0D);
        }
    }
    blog_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    blog_os::locale::print_localized_line("test_println output", 0x0F);
}
