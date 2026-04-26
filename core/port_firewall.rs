/// Port Firewall — белый список безопасных портов ввода-вывода.
///
/// Все прямые обращения к портам должны идти через `safe_outb()` / `safe_inb()`.
/// Запрещённые порты: HDD (0x1F0-0x1F7), IDE (0x170), PCI config (0xCF8-0xCFF).
/// При нарушении — MODULE_ABANDON (лог в serial, продолжить работу).
use x86_64::instructions::port::{Port, PortReadOnly};

// ── Белый список разрешённых диапазонов портов ──────────────────────────────

const ALLOWED: &[(u16, u16, &str)] = &[
    (0x0020, 0x0021, "PIC Master"),
    (0x0040, 0x0043, "PIT 8254"),
    (0x0060, 0x0060, "PS/2 Data"),
    (0x0061, 0x0061, "Speaker/Port B"),
    (0x0064, 0x0064, "PS/2 Command"),
    (0x0070, 0x0071, "CMOS/RTC"),
    (0x0080, 0x0080, "POST Debug Port"),
    (0x0092, 0x0092, "System Control A (A20/reset)"),
    (0x00A0, 0x00A1, "PIC Slave"),
    (0x01CE, 0x01CF, "VBE Bochs"),
    (0x0604, 0x0604, "ACPI Power Off (QEMU)"),
    (0x03C0, 0x03DF, "VGA"),
    (0x03F8, 0x03FF, "Serial COM1"),
];

// ── Чёрный список — порты, которые никогда не должны трогаться ──────────────

const DENIED: &[(u16, u16, &str)] = &[
    (0x01F0, 0x01F7, "ATA Primary HDD"),
    (0x0170, 0x0177, "ATA Secondary HDD"),
    (0x0CF8, 0x0CFF, "PCI Config Space"),
    (0x0778, 0x077A, "Parallel Port"),
];

// ── Проверка ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortAccess {
    Allowed,
    Denied(&'static str),
    Unknown,
}

pub fn check_port(port: u16) -> PortAccess {
    for &(lo, hi, name) in DENIED.iter() {
        if port >= lo && port <= hi {
            return PortAccess::Denied(name);
        }
    }
    for &(lo, hi, _name) in ALLOWED.iter() {
        if port >= lo && port <= hi {
            return PortAccess::Allowed;
        }
    }
    PortAccess::Unknown
}

// ── safe_outb / safe_inb ────────────────────────────────────────────────────

/// Безопасный outb: проверяет порт, логирует запрещённые.
/// Возвращает false если порт запрещён (запись НЕ выполнена).
#[inline]
pub fn safe_outb(port: u16, value: u8) -> bool {
    match check_port(port) {
        PortAccess::Denied(name) => {
            // MODULE_ABANDON: логируем нарушение в serial, продолжаем
            crate::serial_println!(
                "[PORT FIREWALL] DENIED outb(0x{:04X}, 0x{:02X}) — {} [MODULE_ABANDON]",
                port,
                value,
                name
            );
            false
        }
        PortAccess::Allowed | PortAccess::Unknown => {
            unsafe { Port::new(port).write(value) };
            true
        }
    }
}

/// Безопасный inb: проверяет порт, логирует запрещённые.
/// Возвращает None если порт запрещён (чтение НЕ выполнено).
#[inline]
pub fn safe_inb(port: u16) -> Option<u8> {
    match check_port(port) {
        PortAccess::Denied(name) => {
            crate::serial_println!(
                "[PORT FIREWALL] DENIED inb(0x{:04X}) — {} [MODULE_ABANDON]",
                port,
                name
            );
            None
        }
        PortAccess::Allowed | PortAccess::Unknown => {
            Some(unsafe { PortReadOnly::new(port).read() })
        }
    }
}

/// Диагностика: вывести статус порта.
pub fn port_status(port: u16) -> &'static str {
    match check_port(port) {
        PortAccess::Allowed => "ALLOWED",
        PortAccess::Denied(_) => "DENIED",
        PortAccess::Unknown => "UNKNOWN (unlisted)",
    }
}
