// validator.rs — Аппаратный валидатор портов
//
// ПРАВИЛО: ПЕРЕД ЛЮБЫМ ЧТЕНИЕМ ПОРТА — вызвать probe_port() / port_alive().
// 0xFF на шине = нет устройства (floating bus) → не читаем дальше.
//
// Архитектура:
//   probe_port(addr) → u8          — сырое чтение (база всего)
//   port_alive(addr) → bool        — ≠ 0xFF
//   probe_cmos()     → CmosReport  — трёхшаговый опрос (нужна полная логика)
//   probe_ps2()      → bool        — обёртка port_alive(0x64)
//   probe_speaker()  → bool        — обёртка port_alive(0x61)
//   probe_serial()   → bool        — обёртка port_alive(0x3FD)  COM1 LSR
//   probe_vga()      → bool        — обёртка port_alive(0x3DA)  VGA Status 1

// ============================================================
// БАЗОВЫЙ ПРИМИТИВ — единственное место где мы читаем порты
// ============================================================

/// Читает один байт из произвольного x86 порта.
/// Это единственное место в ядре где разрешено читать порт «вслепую».
/// Все остальные probe_*() обязаны использовать эту функцию.
///
/// SAFETY: ring-0; caller обязан убедиться, что addr — стандартный порт.
pub fn probe_port(addr: u16) -> u8 {
    // MODULE_ABANDON: если порт запрещён — логируем и возвращаем 0xFF (floating)
    if let crate::port_firewall::PortAccess::Denied(name) = crate::port_firewall::check_port(addr) {
        crate::serial_println!(
            "[VALIDATOR] MODULE_ABANDON: probe_port(0x{:04X}) denied — {}", addr, name
        );
        return 0xFF;
    }
    unsafe {
        x86_64::instructions::port::Port::<u8>::new(addr).read()
    }
}

/// true = устройство отвечает (шина не floating).
/// 0xFF = нет устройства → не читаем дальше.
pub fn port_alive(addr: u16) -> bool {
    probe_port(addr) != 0xFF
}

// ============================================================
// КОНКРЕТНЫЕ ПРОБЫ — обёртки над port_alive()
// ============================================================

/// PC Speaker / System Control Port B (0x61).
/// Бит 1 = Speaker Data Enable, бит 0 = Timer 2 Gate.
pub fn probe_speaker() -> bool { port_alive(0x61) }

/// PS/2 Status Register (0x64).
/// 0xFF = нет PS/2 контроллера (USB-only машина).
pub fn probe_ps2() -> bool { port_alive(0x64) }

/// COM1 Line Status Register (0x3F8 + 5 = 0x3FD).
/// Idle = 0x60. 0xFF = нет UART / порт не существует.
/// Некоторые UART могут возвращать 0x00 при отключенном питании.
pub fn probe_serial() -> bool { port_alive(0x3FD) }

/// VGA Input Status 1 (0x3DA, read-only, safe to probe).
/// 0xFF = нет VGA контроллера.
pub fn probe_vga() -> bool { port_alive(0x3DA) }

/// Выводит результат опроса PS/2 контроллера.
pub fn display_ps2_probe() {
    if probe_ps2() {
        crate::locale::print_boot_status(
            crate::user_messages::current(crate::user_messages::UiText::ValidatorPs2Ok),
        );
    } else {
        crate::locale::print_localized_line(
            crate::user_messages::current(crate::user_messages::UiText::ValidatorPs2NoResp),
            0x0E,
        );
    }
}

/// Выводит runtime-статус IRQ guard (защита от тяжелых операций из ISR).
pub fn display_irq_guard_status() {
    let guard = crate::irq_guard::guard_enabled();
    let in_irq = crate::irq_guard::is_in_irq();
    let hits = crate::irq_guard::violation_count();

    crate::locale::print_localized_fmt(
        0x0B,
        format_args!(
            "[IRQGUARD] enabled={} in_irq={} violations={}",
            guard,
            in_irq,
            hits,
        ),
    );
}

// ============================================================
// CMOS/RTC — трёхшаговый опрос (сложнее port_alive)
// ============================================================

/// Результат полного опроса CMOS
pub struct CmosReport {
    /// Чип отвечает (порт не возвращает 0xFF)
    pub chip_alive: bool,
    /// Батарейка CMOS жива (регистр 0x0D, бит 7)
    pub battery_ok: bool,
    /// RTC не завис в цикле обновления (UIP сбросился за таймаут)
    pub rtc_ready: bool,
    /// Сырое значение Status A (для диагностики)
    pub status_a: u8,
}

impl CmosReport {
    /// Время достоверно только если все три флага OK
    pub fn time_valid(&self) -> bool {
        self.chip_alive && self.battery_ok && self.rtc_ready
    }
}

/// Читает CMOS регистр через port_alive/probe_port с задержкой.
/// SAFETY: порты 0x70/0x71 — стандартные CMOS, ring-0.
/// Caller должен гарантировать, что индексный регистр не модифицируется другими потоками.
/// Задержка ~2 мкс через POST-порт 0x80 (стандарт для CMOS timing).
pub unsafe fn cmos_probe_reg(reg: u8) -> u8 {
    use x86_64::instructions::interrupts;
    use x86_64::instructions::port::Port;

    interrupts::without_interrupts(|| {
        let mut index: Port<u8> = Port::new(0x70);
        let mut data: Port<u8> = Port::new(0x71);
        index.write(0x80 | (reg & 0x7F));
        // Задержка ~2 мкс через POST-порт 0x80 (стандарт для CMOS timing).
        let mut dummy: Port<u8> = Port::new(0x80);
        let _ = dummy.read();
        let _ = dummy.read();
        data.read()
    })
}

/// Общая функция чтения CMOS регистра с задержкой.
/// Используется другими модулями для устранения дублирования кода.
///
/// # Safety
/// Порты 0x70/0x71 — стандартные CMOS, ring-0.
/// Caller должен гарантировать, что индексный регистр не модифицируется другими потоками.
/// Задержка ~2 мкс через POST-порт 0x80 (стандарт для CMOS timing).
pub unsafe fn cmos_read(reg: u8) -> u8 {
    cmos_probe_reg(reg)
}

/// Читает CMOS регистр БЕЗ задержки (только для специальных случаев).
///
/// # Safety
/// Только для использования в циклах ожидания, где задержка не нужна
/// или может вызвать проблемы с timing.
pub unsafe fn cmos_read_no_delay(reg: u8) -> u8 {
    use x86_64::instructions::interrupts;
    use x86_64::instructions::port::Port;

    interrupts::without_interrupts(|| {
        let mut index: Port<u8> = Port::new(0x70);
        let mut data: Port<u8> = Port::new(0x71);
        index.write(0x80 | (reg & 0x7F));
        data.read()
    })
}

/// Шаг 1: жив ли чип (Status A, 0x0A).
pub fn probe_chip() -> (bool, u8) {
    let val = unsafe { cmos_probe_reg(0x0A) };
    if val == 0xFF {
        return (false, 0xFF);
    }
    (true, val)
}

/// Шаг 2: батарейка (Register D, бит 7).
pub fn probe_battery() -> bool {
    unsafe { cmos_probe_reg(0x0D) & 0x80 != 0 }
}

/// Шаг 3: UIP-флаг — чип не завис в обновлении.
pub fn probe_rtc_ready() -> bool {
    let mut tries: u32 = 0;
    loop {
        let sta = unsafe { cmos_probe_reg(0x0A) };
        if sta == 0xFF {
            crate::serial_println!(
                "[VALIDATOR][RTC] probe_rtc_ready: StatusA=0xFF during UIP wait (chip disappeared)"
            );
            return false;
        }
        if sta & 0x80 == 0 { return true; }  // UIP = 0, готов
        
        // Добавляем задержку для ожидания сброса UIP-флага
        if sta & 0x80 != 0 {
            for _ in 0..200 { core::hint::spin_loop(); }
        }
        
        tries += 1;
        if tries >= 65_000 {
            crate::serial_println!(
                "[VALIDATOR][RTC] probe_rtc_ready: UIP stuck (tries={}, StatusA=0x{:02X})",
                tries,
                sta
            );
            return false;
        }
    }
}

/// Полный трёхшаговый опрос CMOS/RTC.
/// Вызывать ДО любого чтения времени/даты.
pub fn probe_cmos() -> CmosReport {
    let (chip_alive, status_a) = probe_chip();
    if !chip_alive {
        crate::serial_println!("[VALIDATOR][CMOS] chip dead (StatusA=0xFF)");
        return CmosReport { chip_alive: false, battery_ok: false, rtc_ready: false, status_a };
    }
    let battery_ok = probe_battery();
    let rtc_ready  = probe_rtc_ready();
    if !battery_ok {
        crate::serial_println!("[VALIDATOR][CMOS] battery low/dead (RegD bit7=0)");
    }
    if !rtc_ready {
        crate::serial_println!("[VALIDATOR][CMOS] RTC is not ready (UIP stuck or chip error)");
    }
    CmosReport { chip_alive, battery_ok, rtc_ready, status_a }
}

/// Выводит результат опроса CMOS.
pub fn display_cmos_probe(report: &CmosReport) {
    if !report.chip_alive {
        crate::locale::print_localized_line(
            crate::user_messages::current(crate::user_messages::UiText::ValidatorCmosDead),
            0x0C,
        );
        return;
    }
    let bat_str = if report.battery_ok { "ОК (жива)" } else { "СДОХЛА! Время недостоверно" };
    let rtc_str = if report.rtc_ready  { "ОК"        } else { "UIP завис — чип завис?" };
    crate::user_messages::print_validator_cmos(report.status_a, bat_str, rtc_str);
}

// ============================================================
// ПОЛИТИКА ОШИБОК
// ============================================================

/// Уровень критичности аппаратной ошибки.
pub enum ErrorLevel {
    /// Логируем в serial, продолжаем работу — никакого reboot.
    Recoverable,
    /// Закрываем приложение, возвращаемся в shell — никакого reboot.
    ExitToShell,
    /// Нет иного выхода — только тогда перезагрузка.
    Fatal,
}

/// Обрабатывает аппаратную ошибку согласно политике:
/// Recoverable → log + continue
/// ExitToShell → сообщение (вызывающий код возвращается в shell)
/// Fatal → reboot (единственная точка принудительного reboot в ядре)
pub fn handle_hw_error(level: ErrorLevel, msg: &str) {
    match level {
        ErrorLevel::Recoverable => {
            crate::serial_println!("[VALIDATOR][WARN] {}", msg);
            // Минимальная VGA-строка: основной разбор идёт через serial.log.
            crate::user_messages::print_validator_warn(msg);
        }
        ErrorLevel::ExitToShell => {
            crate::serial_println!("[VALIDATOR][ERR] return to shell: {}", msg);
            crate::user_messages::print_validator_err(msg);
        }
        ErrorLevel::Fatal => {
            crate::serial_println!("[VALIDATOR][FATAL] {}", msg);
            crate::serial_println!("[VALIDATOR][FATAL] hard reboot disabled; system will halt");
            crate::user_messages::print_validator_fatal(msg);
            x86_64::instructions::interrupts::disable();
            loop { x86_64::instructions::hlt(); }
        }
    }
}
