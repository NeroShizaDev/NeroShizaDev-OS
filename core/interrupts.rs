use crate::gdt;
use core::sync::atomic::Ordering;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS: spin::Mutex<ChainedPics> =
    // SAFETY: PIC_1_OFFSET=32, PIC_2_OFFSET=40 — стандартные смещения,
    // не пересекаются с CPU исключениями (0-31). pic8259 требует valid offsets.
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        // SAFETY: gdt::DOUBLE_FAULT_IST_INDEX — валидный индекс IST (1..7),
        // инициализированный в gdt::init() перед вызовом init_idt().
        // Отдельный стек обязателен: double fault может возникнуть при
        // переполнении стека (страничная ошибка на стеке), нужен IST.
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

#[inline(always)]
fn halt_forever() -> ! {
    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}

unsafe fn notify_end_of_interrupt_raw(interrupt: InterruptIndex) {
    // IRQ с ведомого PIC требуют EOI сначала на slave (0xA0), затем на master (0x20).
    if interrupt.as_u8() >= PIC_2_OFFSET {
        Port::<u8>::new(0xA0).write(0x20);
    }
    Port::<u8>::new(0x20).write(0x20);
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    crate::serial_println!("========== ТОЧКА ОСТАНОВА ==========");
    crate::serial_println!(
        "Указатель инструкции: {:#x}",
        _stack_frame.instruction_pointer.as_u64()
    );
    crate::serial_println!(
        "Указатель стека:      {:#x}",
        _stack_frame.stack_pointer.as_u64()
    );
    crate::serial_println!("=====================================");
    // Breakpoint (INT3) — recoverable исключение. Логируем и возвращаемся.
    // НЕ halt_forever() — иначе система зависает на первом int3 при отладке.
    crate::trace::record("breakpoint hit");
}

extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    err: PageFaultErrorCode,
) {
    use crate::kernel_messages::KernelEvent;
    use x86_64::registers::control::Cr2;
    crate::trace::record_fatal("page fault handler entered");
    let cr2 = Cr2::read_raw();
    // Serial: полный дамп для диагностики
    crate::serial_println!("========== ОШИБКА СТРАНИЦЫ ==========");
    crate::serial_println!("Адрес обращения (CR2): {:#x}", cr2);
    crate::serial_println!("Код ошибки:            {:#x}", err.bits());
    crate::serial_println!(
        "Указатель инструкции:  {:#x}",
        _stack_frame.instruction_pointer.as_u64()
    );
    crate::serial_println!(
        "Указатель стека:       {:#x}",
        _stack_frame.stack_pointer.as_u64()
    );
    crate::serial_println!("Сегмент кода:          {:#x}", _stack_frame.code_segment.0);
    crate::serial_println!(
        "Флаги CPU:             {:#x}",
        _stack_frame.cpu_flags.bits()
    );
    crate::serial_println!("=====================================");
    // В режиме framebuffer locale-функции сами маршрутизируют в fb_buffer без mutex.
    // render_panic_screen возвращается рано если fb активен, исключая рекурсию фолтов.
    unsafe {
        crate::locale::render_panic_screen(KernelEvent::PageFault);
        crate::locale::write_str_at_vga("CR2=0x", 8, 2, 0x0F);
        crate::locale::write_hex_at_vga(cr2, 8, 8, 0x0D);
        crate::locale::write_str_at_vga("  ERR=0x", 8, 24, 0x0F);
        crate::locale::write_hex32_at_vga(err.bits() as u32, 8, 32, 0x0D);
        crate::locale::write_crash_address(cr2);
        crate::locale::write_registers(
            _stack_frame.instruction_pointer.as_u64(),
            _stack_frame.stack_pointer.as_u64(),
            _stack_frame.cpu_flags.bits(),
        );
    }
    halt_forever()
}

extern "x86-interrupt" fn general_protection_fault_handler(
    _stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    crate::trace::record_fatal("general protection handler entered");
    // Serial: полный дамп для диагностики
    crate::serial_println!("========== НАРУШЕНИЕ ЗАЩИТЫ ==========");
    crate::serial_println!("Код ошибки:           {:#x}", error_code);
    crate::serial_println!(
        "Указатель инструкции: {:#x}",
        _stack_frame.instruction_pointer.as_u64()
    );
    crate::serial_println!(
        "Указатель стека:      {:#x}",
        _stack_frame.stack_pointer.as_u64()
    );
    crate::serial_println!("Сегмент кода:         {:#x}", _stack_frame.code_segment.0);
    crate::serial_println!("Флаги CPU:            {:#x}", _stack_frame.cpu_flags.bits());
    crate::serial_println!("=======================================");
    // В режиме framebuffer locale-функции маршрутизируют в fb_buffer без mutex.
    // render_panic_screen возвращается рано если fb активен, исключая рекурсию фолтов.
    unsafe {
        let ip = _stack_frame.instruction_pointer.as_u64();
        crate::locale::render_panic_screen(crate::kernel_messages::KernelEvent::GeneralProtection);
        crate::locale::write_str_at_vga("ERR=0x", 8, 2, 0x0F);
        crate::locale::write_hex_at_vga(error_code, 8, 8, 0x0D);
        crate::locale::write_crash_address(ip);
        crate::locale::write_registers(
            ip,
            _stack_frame.stack_pointer.as_u64(),
            _stack_frame.cpu_flags.bits(),
        );
    }
    halt_forever()
}

extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    crate::trace::freeze();
    // Прямая запись в COM1 (0x3F8) без Mutex — безопасно в double fault
    fn serial_byte(b: u8) {
        unsafe {
            let mut p = Port::<u8>::new(0x3F8 + 5);
            while p.read() & 0x20 == 0 {}
            Port::<u8>::new(0x3F8).write(b);
        }
    }
    fn serial_str(s: &str) {
        for &b in s.as_bytes() {
            serial_byte(b);
        }
    }
    fn serial_hex(mut v: u64) {
        let mut buf = [0u8; 16];
        for i in (0..16).rev() {
            let d = (v & 0xF) as u8;
            buf[i] = if d < 10 { b'0' + d } else { b'a' + d - 10 };
            v >>= 4;
        }
        serial_str("0x");
        for &b in &buf {
            serial_byte(b);
        }
    }

    serial_str("========== ДВОЙНОЙ СБОЙ ==========\r\n");
    serial_str("Инструкция: ");
    serial_hex(_stack_frame.instruction_pointer.as_u64());
    serial_str("\r\n");
    serial_str("Стек:       ");
    serial_hex(_stack_frame.stack_pointer.as_u64());
    serial_str("\r\n");
    serial_str("Сегмент:    ");
    serial_hex(_stack_frame.code_segment.0 as u64);
    serial_str("\r\n");
    serial_str("Флаги:      ");
    serial_hex(_stack_frame.cpu_flags.bits());
    serial_str("\r\n");
    serial_str("==================================\r\n");

    // Дамп последних trace-записей — показывает, какой модуль вызвал сбой
    fn serial_u8_dec(v: u8) {
        if v >= 10 {
            serial_byte(b'0' + v / 10);
        }
        serial_byte(b'0' + v % 10);
    }
    serial_str("Последние действия (trace):\r\n");
    let trace_len = crate::trace::len();
    if trace_len == 0 {
        serial_str("  (нет)\r\n");
    } else {
        for i in 0..trace_len {
            if let Some(entry) = crate::trace::get_recent(i) {
                serial_str("  [");
                serial_u8_dec(i as u8);
                serial_str("] ");
                serial_str(entry);
                serial_str("\r\n");
            }
        }
    }
    serial_str("==================================\r\n");

    // В double fault нельзя трогать Mutex/println, но write_char_at не берёт lock.
    #[inline(always)]
    fn screen_put(row: usize, col: usize, ch: u8, attr: u8) {
        crate::fb_buffer::write_char_at(col, row, ch, attr);
    }
    #[inline(always)]
    fn screen_write(row: usize, col: usize, s: &str, attr: u8) {
        let mut current_col = col;
        for &b in s.as_bytes() {
            if current_col >= 80 {
                break;
            }
            screen_put(row, current_col, b, attr);
            current_col += 1;
        }
    }
    fn screen_hex64(row: usize, col: usize, mut value: u64, attr: u8) {
        let mut buf = [0u8; 16];
        for i in (0..16).rev() {
            let digit = (value & 0xF) as u8;
            buf[i] = if digit < 10 {
                b'0' + digit
            } else {
                b'a' + digit - 10
            };
            value >>= 4;
        }
        for (i, &b) in buf.iter().enumerate() {
            if col + i >= 80 {
                break;
            }
            screen_put(row, col + i, b, attr);
        }
    }

    const ATTR_BG: u8 = 0x0F; // white on black
    const ATTR_FRAME: u8 = 0xDF; // white on magenta
    const ATTR_HI: u8 = 0x0D; // bright magenta on black

    if crate::fb_buffer::is_initialized() {
        for row in 0..25 {
            for col in 0..80 {
                screen_put(row, col, b' ', ATTR_BG);
            }
        }
        for col in 0..80 {
            screen_put(0, col, b'=', ATTR_FRAME);
            screen_put(24, col, b'=', ATTR_FRAME);
        }
        for row in 1..24 {
            screen_put(row, 0, b'|', ATTR_FRAME);
            screen_put(row, 79, b'|', ATTR_FRAME);
        }

        screen_write(0, 22, "NeroShizaOS KERNEL EVENT", ATTR_FRAME);
        screen_write(2, 3, "DF   [SAFE DOUBLE FAULT SCREEN]", ATTR_BG);
        screen_write(4, 3, "DOUBLE FAULT. System halted.", ATTR_HI);

        screen_write(7, 3, "RIP: 0x", ATTR_BG);
        screen_hex64(7, 10, _stack_frame.instruction_pointer.as_u64(), ATTR_HI);
        screen_write(8, 3, "RSP: 0x", ATTR_BG);
        screen_hex64(8, 10, _stack_frame.stack_pointer.as_u64(), ATTR_HI);
        screen_write(9, 3, "FLG: 0x", ATTR_BG);
        screen_hex64(9, 10, _stack_frame.cpu_flags.bits(), ATTR_HI);

        screen_write(11, 3, "Last actions:", ATTR_BG);
        let mut row = 12usize;
        if trace_len == 0 {
            screen_write(row, 5, "- (none)", ATTR_HI);
        } else {
            let first = trace_len.saturating_sub(6);
            for i in first..trace_len {
                if row >= 23 {
                    break;
                }
                if let Some(entry) = crate::trace::get_recent(i) {
                    screen_write(row, 5, "- ", ATTR_BG);
                    screen_write(row, 7, entry, ATTR_BG);
                    row += 1;
                }
            }
        }

        screen_write(23, 20, "System halted. Check serial.log", ATTR_BG);
    }

    // Единственный выход — halt навсегда. БЕЗ render_panic_screen!
    // (render_panic_screen → spin::Mutex → triple fault → reboot loop)
    halt_forever()
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        notify_end_of_interrupt_raw(InterruptIndex::Timer);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let _irq_scope = crate::irq_guard::enter_irq();
    use pc_keyboard::{DecodedKey, HandleControl, ScancodeSet1};
    use spin::Mutex;

    lazy_static! {
        // ТИП: PS2Keyboard<Layout, Set>
        static ref KEYBOARD: Mutex<pc_keyboard::PS2Keyboard<pc_keyboard::layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(pc_keyboard::PS2Keyboard::new(
                ScancodeSet1::new(),    // Аргумент 1: S (Set)
                pc_keyboard::layouts::Us104Key, // Аргумент 2: L (Layout)
                HandleControl::Ignore
            ));
    }

    let mut port = Port::new(0x60);

    // SAFETY: порт 0x60 — PS/2 Data Register. Чтение в ISR обязательно:
    // PIC ожидает чтения до отправки следующего прерывания.
    // ISR-контекст: нет конкурентного доступа к порту.
    let scancode: u8 = unsafe { port.read() };
    let owner = crate::ps2::input_owner();
    crate::ps2::trace_irq_scancode(scancode, owner);

    // --- Обновление модификаторов ВСЕГДА, независимо от input_owner ---
    // BUG FIX: раньше модификаторы не обновлялись в режиме Apps → глюки хоткеев.
    // Используем AtomicBool (Ordering::Relaxed: ISR — единственный writer, CPU не кеширует).
    match scancode {
        0x38 => crate::shell::ALT_HELD.store(true, Ordering::Relaxed),
        0xB8 => crate::shell::ALT_HELD.store(false, Ordering::Relaxed),
        0x1D => crate::shell::CTRL_HELD.store(true, Ordering::Relaxed),
        0x9D => crate::shell::CTRL_HELD.store(false, Ordering::Relaxed),
        0x2A | 0x36 => crate::shell::SHIFT_HELD.store(true, Ordering::Relaxed),
        0xAA | 0xB6 => crate::shell::SHIFT_HELD.store(false, Ordering::Relaxed),
        _ => {}
    }

    // --- Если ввод принадлежит приложениям — пушим в очередь и выходим ---
    if owner == crate::ps2::InputOwner::Apps {
        // CapsLock в режиме Apps: не шлём в shell, пушим как обычный scancode
        crate::ps2::push_scancode_from_irq(scancode);
        unsafe {
            notify_end_of_interrupt_raw(InterruptIndex::Keyboard);
        }
        return;
    }

    // --- Режим Shell ---

    // CapsLock (0x3A) — перехватываем ДО pc-keyboard, чтобы не менял регистр
    if scancode == 0x3A {
        crate::shell::handle_raw_key(pc_keyboard::KeyCode::CapsLock);
        unsafe {
            notify_end_of_interrupt_raw(InterruptIndex::Keyboard);
        }
        return;
    }
    if scancode == 0xBA {
        // 0x3A | 0x80 = отпускание CapsLock
        unsafe {
            notify_end_of_interrupt_raw(InterruptIndex::Keyboard);
        }
        return;
    }

    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => crate::shell::handle_keyboard_input(character),
                DecodedKey::RawKey(key) => crate::shell::handle_raw_key(key),
            }
        }
    }
    drop(keyboard); // явный drop до EOI

    // SAFETY: EOI для IRQ1; без этого PIC не отправит следующее прерывание клавиатуры.
    unsafe {
        notify_end_of_interrupt_raw(InterruptIndex::Keyboard);
    }
}
