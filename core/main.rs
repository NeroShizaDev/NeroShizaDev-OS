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
// Мы дополнительно identity-маппим legacy VGA graphics/font window вручную,
// чтобы Mode 13h и plane-2 font uploads оставались доступны.
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
#[inline(always)]
fn debugcon_byte(byte: u8) {
    unsafe {
        let mut port = Port::new(0xE9);
        port.write(byte);
    }
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // ============================================================
    // АБСОЛЮТНО ПЕРВАЯ ИНСТРУКЦИЯ ЯДРА — диагностика на реальном железе.
    // Без serial мы слепые: нужен максимально ранний маркер того, что ядро
    // вообще получило управление, ДО любых других вызовов (init, heap,
    // mapper и т.д.). Полноэкранная заливка в VBox перекрывает весь вывод,
    // поэтому оставляем компактный ярко-зелёный marker в левом верхнем углу.
    //
    // Если marker появился — kernel_main выполняется.
    // Если экран остался как был — ядро не получило управление
    //   (проблема бутлоадера, стек, page-tables, или CR3).
    // ============================================================
    if let bootloader_api::info::Optional::Some(ref mut fb) = boot_info.framebuffer {
        let info = fb.info();
        let bpp = info.bytes_per_pixel;
        let stride_bytes = info.stride * bpp;
        let buf = fb.buffer_mut();
        let marker_w = info.width.min(64);
        let marker_h = info.height.min(32);
        for y in 0..marker_h {
            let row_start = y * stride_bytes;
            let row_end = row_start + marker_w * bpp;
            if row_end > buf.len() {
                break;
            }
            let mut i = row_start;
            while i + bpp <= row_end {
                buf[i] = 0x00; // B
                buf[i + 1] = 0xFF; // G (ярко-зелёный)
                buf[i + 2] = 0x00; // R
                if bpp >= 4 {
                    buf[i + 3] = 0;
                }
                i += bpp;
            }
        }
    }

    debugcon_byte(b'A');
    blog_os::serial_println!("[ЯДРО] kernel_main entered");

    // ============================================================
    // ШАГ 0: GDT → IDT → PICS → FPU — ДО ЛЮБЫХ ОБРАЩЕНИЙ К СТРАНИЦАМ!
    // memory::init() ходит по CR3/page tables → может вызвать исключение.
    // map_vga_memory() модифицирует page tables → тоже.
    // Без IDT любое исключение = double fault = triple fault = ребут.
    // ============================================================
    blog_os::init();
    debugcon_byte(b'B');
    blog_os::serial_println!("[ЯДРО] cpu init done");

    // ============================================================
    // После поднятия IDT можно безопасно трогать framebuffer bootloader'а:
    // если здесь всё ещё будет fault, мы уже увидим page-fault/double-fault
    // отчёт вместо немого зависания сразу после jump to kernel entry point.
    // ============================================================
    {
        if let bootloader_api::info::Optional::Some(ref mut fb) = boot_info.framebuffer {
            let info = fb.info();
            let buf = fb.buffer_mut();
            let addr = buf.as_mut_ptr() as usize;
            unsafe {
                blog_os::fb_buffer::init(
                    addr,
                    info.width,
                    info.height,
                    info.stride,
                    info.bytes_per_pixel,
                    info.byte_len,
                );
            }
            debugcon_byte(b'C');
            blog_os::serial_println!(
                "[ЯДРО] framebuffer init done: {}x{} stride={} bpp={} len={}",
                info.width,
                info.height,
                info.stride,
                info.bytes_per_pixel,
                info.byte_len,
            );
        } else {
            debugcon_byte(b'N');
            blog_os::serial_println!("[ЯДРО] framebuffer absent");
        }
    }

    // ============================================================
    // ДИАГНОСТИКА FRAMEBUFFER — legacy VGA text path на этом железе недоступен
    // (RTX 3060 + CSM: загрузчик поднял VBE framebuffer, не VGA text).
    // Пишем цветные полоски прямо в framebuffer из boot_info.
    // Каждая полоска = 1 шаг — если её видно, всё до неё прошло OK.
    //   Полоска 1 (красная,  пиксели 0..319)   — вошли в kernel_main
    //   Полоска 2 (зелёная,  пиксели 320..639) — heap OK
    //   Полоска 3 (синяя,    пиксели 640..959) — phys_offset прочитан
    //   Полоска 4 (жёлтая,   пиксели 960..1279)— mapper init OK
    //   Полоска 5 (белая,    пиксели 1280..1599)— map_vga_memory OK
    // ============================================================
    // Вспомогательный макрос: рисует ТОЛСТУЮ горизонтальную полосу (STRIPE_H пикселей).
    // Полосы расположены с шагом STRIPE_H*2, чтобы не перекрывались.
    // Поддерживает bpp=3 (BGR) и bpp=4 (BGRx).
    const STRIPE_H: usize = 24; // высота полосы в пикселях — видно на любом мониторе
    macro_rules! fb_stripe {
        ($fb_info:expr, $stripe_idx:expr, $b:expr, $g:expr, $r:expr) => {
            if let bootloader_api::info::Optional::Some(ref mut fb) = $fb_info.framebuffer {
                let info = fb.info();
                let bpp = info.bytes_per_pixel;
                if bpp >= 3 {
                    let stride_bytes = info.stride * bpp;
                    let buf = fb.buffer_mut();
                    let y_start = $stripe_idx * STRIPE_H * 2;
                    for py in y_start..(y_start + STRIPE_H) {
                        let row_start = py * stride_bytes;
                        let row_end = row_start + info.width * bpp;
                        if buf.len() < row_end {
                            break;
                        }
                        let mut i = row_start;
                        while i + bpp <= row_end {
                            buf[i] = $b;
                            buf[i + 1] = $g;
                            buf[i + 2] = $r;
                            if bpp >= 4 {
                                buf[i + 3] = 0xFF;
                            }
                            i += bpp;
                        }
                    }
                }
            }
        };
    }
    fb_stripe!(boot_info, 0, 0x00, 0x00, 0xFF); // полоска 0 (синяя) — kernel_main достигнут

    // ПЕРВЫМ ДЕЛОМ — инициализируем кучу (bump-аллокатор в BSS).
    blog_os::apps::games::doom::stubs::init_heap();

    fb_stripe!(boot_info, 1, 0x00, 0xFF, 0x00); // полоска 1 (зелёная) — heap OK

    fb_stripe!(boot_info, 2, 0xFF, 0x55, 0x00); // полоска 2 (оранжевая) — init() уже был выше
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

    fb_stripe!(boot_info, 3, 0xFF, 0x00, 0x00); // полоска 3 (красная) — phys_offset прочитан

    let phys_offset_addr = VirtAddr::new(phys_offset);
    let _mapper = unsafe { blog_os::memory::init(phys_offset_addr) };

    fb_stripe!(boot_info, 4, 0x00, 0xFF, 0xFF); // полоска 4 (жёлтая) — mapper OK

    let _frame_allocator =
        unsafe { blog_os::memory::BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    fb_stripe!(boot_info, 5, 0xFF, 0xFF, 0xFF); // полоска 5 (белая) — allocator OK

    blog_os::serial_println!("[ЯДРО] Фаза 0: legacy VGA path skipped, framebuffer only");
    blog_os::trace::record("legacy vga path skipped; framebuffer only");

    // ============================================================
    // ФАЗА 1: Ранняя инициализация & Линия жизни
    // serial (макросы serial_print! уже доступны), validator (готов),
    // vga_hw (используется font-загрузчиком), vga_buffer, logo.
    // Цель: видеть вывод даже если следующие фазы упадут.
    // ============================================================
    blog_os::trace::record("phase1 clear_screen start");
    blog_os::fb_buffer::clear_screen();
    blog_os::trace::record("phase1 clear_screen done");
    blog_os::trace::record("phase1 localized header start");
    blog_os::locale::print_localized_line(
        blog_os::kernel_messages::current(blog_os::kernel_messages::UiText::SystemCheckHeader),
        0x0B,
    );
    blog_os::trace::record("phase1 localized header done");
    match blog_os::locale::get_locale() {
        blog_os::kernel_messages::Locale::RuRu => blog_os::locale::print_localized_fmt(
            0x0E,
            format_args!("[Фаза 1] Буфер кадра: активен"),
        ),
        blog_os::kernel_messages::Locale::EnUs => blog_os::locale::print_localized_fmt(
            0x0E,
            format_args!("[Phase 1] framebuffer: active"),
        ),
        blog_os::kernel_messages::Locale::ArEg => blog_os::locale::print_localized_fmt(
            0x0E,
            format_args!("[المرحلة 1] framebuffer: active"),
        ),
    }
    blog_os::serial_println!("[ЯДРО] Фаза 1: framebuffer path active, legacy VGA detect skipped");
    blog_os::trace::record("framebuffer path active");

    // ============================================================
    // ФАЗА 2: Архитектура CPU — уже инициализирована в ШАГ 0
    // GDT + IDT + PICS + FPU подняты ДО memory::init и map_vga_memory.
    // ============================================================
    blog_os::serial_println!("[ЯДРО] Фаза 2: GDT + IDT + PICS + FPU готовы (подняты в шаг 0)");
    blog_os::trace::record("gdt idt pics fpu ready");
    blog_os::locale::print_boot_status(blog_os::kernel_messages::current(
        blog_os::kernel_messages::UiText::Phase2CpuOk,
    ));

    // ============================================================
    // ФАЗА 3: Память
    // Физическая память и куча инициализированы загрузчиком (bootimage).
    // При необходимости здесь будет memory::init(physical_offset).
    // ============================================================
    blog_os::locale::print_boot_status(blog_os::kernel_messages::current(
        blog_os::kernel_messages::UiText::Phase3MemoryOk,
    ));
    blog_os::serial_println!("[ЯДРО] Фаза 3: Память ОК");

    // ============================================================
    // ФАЗА 4: Время и Энтропия
    // Сначала validator::probe_cmos() (уже внутри display_status),
    // затем чтение времени, температура CPU, RNG.
    // ============================================================
    blog_os::apps::rtc::display_status();
    blog_os::apps::rtc::display_thermal();
    let risk = blog_os::validator::probe_pre_freeze_risks();
    blog_os::validator::display_pre_freeze_risks(&risk);
    blog_os::trace::record("rtc validator phase done");

    let rng_ok = blog_os::apps::rng::is_supported();
    blog_os::serial_println!(
        "[ЯДРО] Фаза 4: RDRAND {}",
        if rng_ok { "ВКЛ" } else { "ВЫКЛ" }
    );
    blog_os::locale::print_localized_line(
        blog_os::kernel_messages::current(if rng_ok {
            blog_os::kernel_messages::UiText::Phase4RngOn
        } else {
            blog_os::kernel_messages::UiText::Phase4RngOff
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
        blog_os::locale::print_boot_status(blog_os::kernel_messages::current(
            blog_os::kernel_messages::UiText::Phase5SpeakerOk,
        ));
        boot_beep();
    } else {
        blog_os::serial_println!("[ЯДРО] Фаза 5: Спикер НЕ НАЙДЕН");
        blog_os::locale::print_localized_line(
            blog_os::kernel_messages::current(blog_os::kernel_messages::UiText::Phase5SpeakerFail),
            0x0E,
        );
    }

    // ============================================================
    // ФАЗА 6: Мультиязычность и Рендеринг текста
    // Шрифты → локаль → Unicode-подсистема → баннер OS.
    // ============================================================
    blog_os::locale::draw_locale_badge();
    blog_os::serial_println!("[ЯДРО] Фаза 6: Шрифт + локаль готовы");
    blog_os::trace::record("font + locale ready");
    blog_os::locale::print_phase6_ok();
    print_startup_banner();

    // ============================================================
    // ФАЗА 7: Пространство пользователя — Shell готов
    // Все демо-приложения запускаются через APPS меню.
    // ============================================================
    print_phase7_ready_line();
    blog_os::trace::record("phase7 logo start");
    blog_os::logo::show_boot_logo();
    blog_os::trace::record("phase7 logo done");
    blog_os::shell::show_shell_prompt();
    blog_os::trace::record("shell prompt drawn");

    let unicode_report = blog_os::vga_unicode::init_runtime_after_shell();
    blog_os::trace::record("unicode runtime init after shell");
    blog_os::serial_println!(
        "[ЯДРО] Unicode runtime init after shell complete: mode={:?} ready={}/{} cyr={} ar={}",
        unicode_report.mode,
        unicode_report.ready,
        unicode_report.checked,
        if unicode_report.cyrillic_ready { 1 } else { 0 },
        if unicode_report.arabic_ready { 1 } else { 0 }
    );
    match blog_os::locale::get_locale() {
        blog_os::kernel_messages::Locale::RuRu => {
            blog_os::locale::print_localized_line("[Фаза 8] Юникод прогружен после шелла", 0x0A)
        }
        blog_os::kernel_messages::Locale::EnUs => {
            blog_os::locale::print_localized_line("[Phase 8] Unicode loaded after shell", 0x0A)
        }
        blog_os::kernel_messages::Locale::ArEg => {
            blog_os::locale::print_localized_line("[المرحلة 8] تم تحميل Unicode بعد shell", 0x0A)
        }
    }
    blog_os::shell::show_shell_prompt();

    blog_os::serial_println!("[ЯДРО] Фаза 7: Шелл готов, включаем прерывания");
    // Разрешаем сканирование PS/2 клавиатуры перед включением прерываний.
    // QEMU включает сканирование по умолчанию, VirtualBox оставляет порт
    // в отключённом состоянии. Вызываем здесь — после всей инициализации,
    // перед sti, чтобы избежать раннего тройного сброса в VirtualBox.
    unsafe { blog_os::ps2::init() };
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
        blog_os::apps::beeper::play(880);
        // Задержка ~200 мс: порт 0x80 (POST-диагностический) ≈ 1 мкс/чтение
        let mut p: Port<u8> = Port::new(0x80);
        for _ in 0u32..200_000 {
            core::hint::black_box(p.read());
        }
        blog_os::apps::beeper::stop();
    }
}

// ================================================================
// Баннер ОС с Unicode-статистикой (Фаза 6)
// ================================================================
fn print_startup_banner() {
    blog_os::locale::print_localized_line(
        blog_os::kernel_messages::current(blog_os::kernel_messages::UiText::BootBannerTitle),
        0x0E,
    );
    blog_os::locale::print_localized_line(
        blog_os::kernel_messages::current(blog_os::kernel_messages::UiText::BootBannerUnicode),
        0x0E,
    );
    match blog_os::locale::get_locale() {
        blog_os::kernel_messages::Locale::RuRu => {
            blog_os::locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "Блоки: {} | Скрипты: {} | Символы: {}",
                    blog_os::unicode_blocks::block_count(),
                    blog_os::unicode_scripts::script_count(),
                    blog_os::unicode_categories::total_defined_chars(),
                ),
            );
            blog_os::locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "Словарь: {} интентов ({} байт)",
                    blog_os::unicode::dict_size(),
                    blog_os::unicode::dict_bytes()
                ),
            );
        }
        blog_os::kernel_messages::Locale::EnUs => {
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
        blog_os::kernel_messages::Locale::ArEg => {
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
        }
    }
}

fn print_phase7_ready_line() {
    blog_os::fb_buffer::set_color(blog_os::fb_buffer::ColorCode::new(
        blog_os::fb_buffer::Color::LightGreen,
        blog_os::fb_buffer::Color::Black,
    ));
    blog_os::println!(
        "{}",
        blog_os::kernel_messages::current(blog_os::kernel_messages::UiText::Phase7ShellReady)
    );
    blog_os::fb_buffer::set_color(blog_os::fb_buffer::ColorCode::new(
        blog_os::fb_buffer::Color::Yellow,
        blog_os::fb_buffer::Color::Black,
    ));
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
            blog_os::locale::write_ascii_str_at_vga(loc.file(), 13, 2, 0x0F);
            blog_os::locale::write_ascii_str_at_vga(":", 13, loc.file().len().min(68) + 2, 0x0F);
            blog_os::locale::write_hex32_at_vga(loc.line(), 13, loc.file().len().min(68) + 3, 0x0D);
        }
    }
    blog_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    blog_os::locale::print_localized_line("test_println output", 0x0F);
}
