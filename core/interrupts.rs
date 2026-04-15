use crate::gdt;
use core::sync::atomic::AtomicBool;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

static FIRST_TIMER_IRQ: AtomicBool = AtomicBool::new(true);

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 { self as u8 }
    fn as_usize(self) -> usize { usize::from(self.as_u8()) }
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

unsafe fn notify_end_of_interrupt_raw(interrupt: InterruptIndex) {
    // IRQ с ведомого PIC требуют EOI сначала на slave (0xA0), затем на master (0x20).
    if interrupt.as_u8() >= PIC_2_OFFSET {
        Port::<u8>::new(0xA0).write(0x20);
    }
    Port::<u8>::new(0x20).write(0x20);
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    crate::serial_println!("========== ТОЧКА ОСТАНОВА ==========");
    crate::serial_println!("Указатель инструкции: {:#x}", _stack_frame.instruction_pointer.as_u64());
    crate::serial_println!("Указатель стека:      {:#x}", _stack_frame.stack_pointer.as_u64());
    crate::serial_println!("=====================================");
    crate::trace::record("breakpoint handler entered");
    unsafe {
        let ip = _stack_frame.instruction_pointer.as_u64();
        crate::locale::render_panic_screen(crate::kernel_messages::KernelEvent::BreakPoint);
        crate::locale::write_crash_address(ip);
        crate::locale::write_registers(
            ip,
            _stack_frame.stack_pointer.as_u64(),
            _stack_frame.cpu_flags.bits(),
        );
    }
    x86_64::instructions::interrupts::disable();
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    err: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    use crate::kernel_messages::KernelEvent;
    crate::trace::record("page fault handler entered");
    let cr2 = Cr2::read_raw();
    // Serial: полный дамп для диагностики
    crate::serial_println!("========== ОШИБКА СТРАНИЦЫ ==========");
    crate::serial_println!("Адрес обращения (CR2): {:#x}", cr2);
    crate::serial_println!("Код ошибки:            {:#x}", err.bits());
    crate::serial_println!("Указатель инструкции:  {:#x}", _stack_frame.instruction_pointer.as_u64());
    crate::serial_println!("Указатель стека:       {:#x}", _stack_frame.stack_pointer.as_u64());
    crate::serial_println!("Сегмент кода:          {:#x}", _stack_frame.code_segment.0);
    crate::serial_println!("Флаги CPU:             {:#x}", _stack_frame.cpu_flags.bits());
    crate::serial_println!("=====================================");
    // SAFETY: page fault handler — прерывания отключены, мьютексы нельзя использовать.
    // render_panic_screen и write_*_at_vga обращаются к VGA напрямую без блокировок.
    // Это единственный безопасный способ вывода информации в момент page fault.
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
    x86_64::instructions::interrupts::disable();
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn general_protection_fault_handler(
    _stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    crate::trace::record("general protection handler entered");
    // Serial: полный дамп для диагностики
    crate::serial_println!("========== НАРУШЕНИЕ ЗАЩИТЫ ==========");
    crate::serial_println!("Код ошибки:           {:#x}", error_code);
    crate::serial_println!("Указатель инструкции: {:#x}", _stack_frame.instruction_pointer.as_u64());
    crate::serial_println!("Указатель стека:      {:#x}", _stack_frame.stack_pointer.as_u64());
    crate::serial_println!("Сегмент кода:         {:#x}", _stack_frame.code_segment.0);
    crate::serial_println!("Флаги CPU:            {:#x}", _stack_frame.cpu_flags.bits());
    crate::serial_println!("=======================================");
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
    x86_64::instructions::interrupts::disable();
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    // Прямая запись в COM1 (0x3F8) без Mutex — безопасно в double fault
    fn serial_byte(b: u8) {
        unsafe {
            let mut p = Port::<u8>::new(0x3F8 + 5);
            while p.read() & 0x20 == 0 {}
            Port::<u8>::new(0x3F8).write(b);
        }
    }
    fn serial_str(s: &str) {
        for &b in s.as_bytes() { serial_byte(b); }
    }
    fn serial_hex(mut v: u64) {
        let mut buf = [0u8; 16];
        for i in (0..16).rev() {
            let d = (v & 0xF) as u8;
            buf[i] = if d < 10 { b'0' + d } else { b'a' + d - 10 };
            v >>= 4;
        }
        serial_str("0x");
        for &b in &buf { serial_byte(b); }
    }

    serial_str("========== ДВОЙНОЙ СБОЙ ==========\r\n");
    serial_str("Инструкция: "); serial_hex(_stack_frame.instruction_pointer.as_u64()); serial_str("\r\n");
    serial_str("Стек:       "); serial_hex(_stack_frame.stack_pointer.as_u64()); serial_str("\r\n");
    serial_str("Сегмент:    "); serial_hex(_stack_frame.code_segment.0 as u64); serial_str("\r\n");
    serial_str("Флаги:      "); serial_hex(_stack_frame.cpu_flags.bits()); serial_str("\r\n");
    serial_str("==================================\r\n");
    // SAFETY: double fault — стек может быть повреждён (мы на IST стеке).
    // render_panic_screen пишет в VGA напрямую без spin::Mutex (дедлок недопустим).
    unsafe {
        crate::locale::render_panic_screen(crate::kernel_messages::KernelEvent::DoubleFault);
        let ip = _stack_frame.instruction_pointer.as_u64();
        crate::locale::write_crash_address(ip);
        crate::locale::write_registers(
            ip,
            _stack_frame.stack_pointer.as_u64(),
            _stack_frame.cpu_flags.bits(),
        );
    }
    x86_64::instructions::interrupts::disable();
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        notify_end_of_interrupt_raw(InterruptIndex::Timer);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
    use spin::Mutex;

    lazy_static! {
        // ТИП: Keyboard<Layout, Set>
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),    // Аргумент 1: S (Set)
                layouts::Us104Key,      // Аргумент 2: L (Layout)
                HandleControl::Ignore
            ));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    // SAFETY: порт 0x60 — PS/2 Data Register. Чтение в ISR обязательно:
    // PIC ожидает чтения до отправки следующего прерывания.
    // ISR-контекст: нет конкурентного доступа к порту.
    let scancode: u8 = unsafe { port.read() };

    // SAFETY (ALT/CTRL/SHIFT): static mut bool — запись из ISR-контекста.
    // Прерывания отключены пока мы в ISR (CPU отключает IF при входе).
    // Единственный writer — этот ISR; читатели в lib.rs не конкурируют.

    // Alt (Left Alt = 0x38): треким состояние для Alt+Fn хоткеев
    if scancode == 0x38 {
        unsafe { crate::shell::ALT_HELD = true; }
    }
    if scancode == 0xB8 {
        unsafe { crate::shell::ALT_HELD = false; }
    }

    // Ctrl (Left Ctrl = 0x1D): треким для Ctrl+C/V/X/A
    if scancode == 0x1D {
        unsafe { crate::shell::CTRL_HELD = true; }
    }
    if scancode == 0x9D {
        unsafe { crate::shell::CTRL_HELD = false; }
    }

    // Shift (Left = 0x2A, Right = 0x36): треким для Shift+стрелки
    if scancode == 0x2A || scancode == 0x36 {
        unsafe { crate::shell::SHIFT_HELD = true; }
    }
    if scancode == 0xAA || scancode == 0xB6 {
        unsafe { crate::shell::SHIFT_HELD = false; }
    }

    // CapsLock (0x3A) — перехватываем ДО pc-keyboard, чтобы не менял регистр
    // Только нажатие (без бита 0x80), отпускание игнорируем
    if scancode == 0x3A {
        crate::shell::handle_raw_key(pc_keyboard::KeyCode::CapsLock);
        // SAFETY: IRQ1 приходит с master PIC, достаточно прямого EOI.
        unsafe {
            notify_end_of_interrupt_raw(InterruptIndex::Keyboard);
        }
        return;
    }
    if scancode == 0xBA { // 0x3A | 0x80 = отпускание CapsLock
        // SAFETY: EOI — аналогично выше.
        unsafe {
            notify_end_of_interrupt_raw(InterruptIndex::Keyboard);
        }
        return;
    }

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => crate::shell::handle_keyboard_input(character),
                DecodedKey::RawKey(key) => crate::shell::handle_raw_key(key),
            }
        }
    }

    // SAFETY: EOI для IRQ1; без этого PIC не отправит следующее прерывание клавиатуры.
    unsafe {
        notify_end_of_interrupt_raw(InterruptIndex::Keyboard);
    }
}
