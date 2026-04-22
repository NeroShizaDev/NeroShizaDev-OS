// ============================================================
// NHS SERIAL RECEIVER — принимает .nhs пакеты через COM1
// ============================================================
//
// Протокол:
//   host  -> guest : "NHS_SYNC"
//   guest -> host  : "NHS_READY"
//   host  -> guest : [4 байта LE: total_size] [total_size байт: NHS данные]
//   guest -> host  : "NHS_OK" | "NHS_ERR"
//
// COM1 опрашивается в цикле (polling), без IRQ.
//
// Хост-сторона (Python):
//   python tools/send_nhs.py app.nhs --tcp localhost:4321
//
// ВАЖНО: run.bat использует -serial file:serial.log (только запись).
// Для приёма данных замените на:
//   -serial tcp:127.0.0.1:4321,server,nowait
// Тогда хост подключается через netcat или tools/send_nhs.py.
// ============================================================

use x86_64::instructions::port::Port;

const COM1_DATA: u16 = 0x3F8;
const COM1_LSR:  u16 = 0x3FD;
const LSR_DR:    u8  = 0x01;   // Data Ready bit
const LSR_THRE:  u8  = 0x20;   // Transmitter Holding Register Empty

const MAX_NHS: usize = 64 * 1024;
const SYNC_MAGIC: &[u8] = b"NHS_SYNC";
const READY_MAGIC: &[u8] = b"NHS_READY";
const OK_MAGIC: &[u8] = b"NHS_OK";
const ERR_MAGIC: &[u8] = b"NHS_ERR";

/// Статический буфер приёма — место для одного входящего пакета.
/// Перезаписывается при каждом вызове receive().
static mut STAGE: [u8; MAX_NHS] = [0u8; MAX_NHS];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RecvError {
    Timeout,        // нет данных за ~2 сек
    TooLarge,       // размер > 64 KB
    InvalidMagic,   // первые 4 байта не NHS magic
}

impl RecvError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Timeout      => "Timeout: no data on COM1 within 2 sec.",
            Self::TooLarge     => "Package too large (> 64 KB).",
            Self::InvalidMagic => "Invalid magic bytes — not a .nhs file.",
        }
    }
}

/// Polling-чтение одного байта с COM1.
/// Таймаут ~2 сек (100 MIPS CPU → ~200 000 000 итераций).
fn recv_byte() -> Option<u8> {
    const TIMEOUT: u64 = 200_000_000;
    let mut lsr: Port<u8> = Port::new(COM1_LSR);
    let mut dat: Port<u8> = Port::new(COM1_DATA);
    for _ in 0..TIMEOUT {
        let status = unsafe { lsr.read() };
        if status & LSR_DR != 0 {
            return Some(unsafe { dat.read() });
        }
        core::hint::spin_loop();
    }
    None
}

fn send_byte(byte: u8) {
    let mut lsr: Port<u8> = Port::new(COM1_LSR);
    let mut dat: Port<u8> = Port::new(COM1_DATA);
    loop {
        let status = unsafe { lsr.read() };
        if status & LSR_THRE != 0 {
            unsafe { dat.write(byte); }
            return;
        }
        core::hint::spin_loop();
    }
}

fn send_bytes(bytes: &[u8]) {
    for &byte in bytes {
        send_byte(byte);
    }
}

fn wait_for_sync() -> Result<(), RecvError> {
    let mut match_idx = 0usize;
    while match_idx < SYNC_MAGIC.len() {
        let byte = recv_byte().ok_or(RecvError::Timeout)?;
        if byte == SYNC_MAGIC[match_idx] {
            match_idx += 1;
        } else {
            match_idx = if byte == SYNC_MAGIC[0] { 1 } else { 0 };
        }
    }
    Ok(())
}

/// Блокирующий приём NHS-пакета по COM1.
/// Возвращает срез в статическом буфере STAGE.
///
/// Протокол: NHS_SYNC -> NHS_READY -> [u32 LE: размер][<размер> байт NHS-данных]
pub fn receive() -> Result<&'static [u8], RecvError> {
    wait_for_sync()?;
    send_bytes(READY_MAGIC);

    // 1. Читаем 4-байтовый заголовок с размером
    let mut hdr = [0u8; 4];
    for b in hdr.iter_mut() {
        *b = recv_byte().ok_or(RecvError::Timeout)?;
    }
    let size = u32::from_le_bytes(hdr) as usize;
    if size > MAX_NHS {
        send_bytes(ERR_MAGIC);
        return Err(RecvError::TooLarge);
    }

    // 2. Читаем payload
    for i in 0..size {
        let b = recv_byte().ok_or(RecvError::Timeout)?;
        unsafe { STAGE[i] = b; }
    }

    // 3. Проверяем NHS magic: 'N' 'H' 'S' 0x1A
    if size < 4 {
        send_bytes(ERR_MAGIC);
        return Err(RecvError::InvalidMagic);
    }
    let magic_ok = unsafe {
        STAGE[0] == b'N' && STAGE[1] == b'H' && STAGE[2] == b'S' && STAGE[3] == 0x1A
    };
    if !magic_ok {
        send_bytes(ERR_MAGIC);
        return Err(RecvError::InvalidMagic);
    }

    send_bytes(OK_MAGIC);
    Ok(unsafe { &STAGE[..size] })
}
