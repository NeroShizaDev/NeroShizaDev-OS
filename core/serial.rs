use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::{Config, Uart16550Tty, backend::PioBackend};
use x86_64::instructions::port::Port;

static SERIAL_EVENT_SEQ: AtomicU64 = AtomicU64::new(0);
static LOG_MIN_LEVEL: AtomicU64 = AtomicU64::new(LogLevel::Trace as u64);
static LOG_SUBSYS_MASK: AtomicU64 = AtomicU64::new(u64::MAX);
static LOG_FORMAT: AtomicU64 = AtomicU64::new(LogFormat::Canonical as u64);
static LOG_DEDUP_ENABLED: AtomicU64 = AtomicU64::new(1);
static LAST_EVENT_HASH: AtomicU64 = AtomicU64::new(0);
static LAST_EVENT_SUPPRESSED: AtomicU64 = AtomicU64::new(0);

#[inline(always)]
fn raw_serial_probe(tag: u8) {
    unsafe {
        let mut lsr = Port::<u8>::new(0x3F8 + 5);
        while lsr.read() & 0x20 == 0 {}
        Port::<u8>::new(0x3F8).write(tag);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Trace = 1,
    Debug = 5,
    Info = 9,
    Warn = 13,
    Error = 17,
    Fatal = 21,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LogFormat {
    Canonical = 1,
    Compact = 2,
    LogFmt = 3,
}

impl LogFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            LogFormat::Canonical => "canonical",
            LogFormat::Compact => "compact",
            LogFormat::LogFmt => "logfmt",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("canonical") || s.eq_ignore_ascii_case("std") {
            Some(LogFormat::Canonical)
        } else if s.eq_ignore_ascii_case("compact") || s.eq_ignore_ascii_case("short") {
            Some(LogFormat::Compact)
        } else if s.eq_ignore_ascii_case("logfmt") || s.eq_ignore_ascii_case("structured") {
            Some(LogFormat::LogFmt)
        } else {
            None
        }
    }
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("trace") {
            Some(LogLevel::Trace)
        } else if s.eq_ignore_ascii_case("debug") {
            Some(LogLevel::Debug)
        } else if s.eq_ignore_ascii_case("info") {
            Some(LogLevel::Info)
        } else if s.eq_ignore_ascii_case("warn") || s.eq_ignore_ascii_case("warning") {
            Some(LogLevel::Warn)
        } else if s.eq_ignore_ascii_case("error") || s.eq_ignore_ascii_case("err") {
            Some(LogLevel::Error)
        } else if s.eq_ignore_ascii_case("fatal") || s.eq_ignore_ascii_case("crit") {
            Some(LogLevel::Fatal)
        } else {
            None
        }
    }
}

struct FixedBuf<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> FixedBuf<N> {
    fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

impl<const N: usize> core::fmt::Write for FixedBuf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let free = N.saturating_sub(self.len);
        if free == 0 {
            return Ok(());
        }
        let take = s.len().min(free);
        self.bytes[self.len..self.len + take].copy_from_slice(&s.as_bytes()[..take]);
        self.len += take;
        Ok(())
    }
}

lazy_static! {
    pub static ref SERIAL1: Mutex<Uart16550Tty<PioBackend>> = {
        // SAFETY: 0x3F8 — стандартный базовый адрес COM1.
        // Вызывается единственный раз при первом обращении к SERIAL1.
        let serial = unsafe {
            Uart16550Tty::new_port(0x3F8, Config::default())
                .expect("serial: не могу инициализировать COM1")
        };
        Mutex::new(serial)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

#[inline(always)]
fn input_owner_name() -> &'static str {
    match crate::ps2::input_owner() {
        crate::ps2::InputOwner::Shell => "Shell",
        crate::ps2::InputOwner::Apps => "Apps",
    }
}

#[inline(always)]
fn severity_from_tag(tag: &str) -> Option<LogLevel> {
    if tag.eq_ignore_ascii_case("TRACE") {
        Some(LogLevel::Trace)
    } else if tag.eq_ignore_ascii_case("DEBUG") || tag.eq_ignore_ascii_case("DBG") {
        Some(LogLevel::Debug)
    } else if tag.eq_ignore_ascii_case("INFO") {
        Some(LogLevel::Info)
    } else if tag.eq_ignore_ascii_case("WARN") || tag.eq_ignore_ascii_case("WARNING") {
        Some(LogLevel::Warn)
    } else if tag.eq_ignore_ascii_case("ERR")
        || tag.eq_ignore_ascii_case("ERROR")
        || tag.eq_ignore_ascii_case("OOM")
    {
        Some(LogLevel::Error)
    } else if tag.eq_ignore_ascii_case("FATAL")
        || tag.eq_ignore_ascii_case("BITE")
        || tag.eq_ignore_ascii_case("PANIC")
    {
        Some(LogLevel::Fatal)
    } else {
        None
    }
}

#[inline(always)]
fn next_tag<'a>(s: &'a str, start: usize) -> Option<(&'a str, usize)> {
    let bytes = s.as_bytes();
    if start >= bytes.len() || bytes[start] != b'[' {
        return None;
    }
    let mut end = start + 1;
    while end < bytes.len() && bytes[end] != b']' {
        end += 1;
    }
    if end >= bytes.len() {
        return None;
    }
    Some((&s[start + 1..end], end + 1))
}

fn classify_body<'a>(body: &'a str) -> (LogLevel, &'a str, &'a str) {
    let mut tags = [""; 3];
    let mut count = 0usize;
    let mut pos = 0usize;

    while count < tags.len() {
        let Some((tag, next)) = next_tag(body, pos) else {
            break;
        };
        tags[count] = tag;
        count += 1;
        pos = next;
    }

    let subsys = if count > 0 && !tags[0].is_empty() {
        tags[0]
    } else {
        "GEN"
    };
    let mut level = None;
    let mut event = "log";
    for tag in tags.iter().take(count).skip(1) {
        if tag.is_empty() {
            continue;
        }
        if level.is_none() {
            level = severity_from_tag(tag);
        }
        if event == "log" && severity_from_tag(tag).is_none() {
            event = tag;
        }
    }

    let level = level.unwrap_or_else(|| {
        if subsys.eq_ignore_ascii_case("BITE") {
            LogLevel::Fatal
        } else if subsys.eq_ignore_ascii_case("PS2") && event.eq_ignore_ascii_case("SC") {
            LogLevel::Debug
        } else if subsys.eq_ignore_ascii_case("MEM") {
            LogLevel::Debug
        } else {
            LogLevel::Info
        }
    });

    (level, subsys, event)
}

#[inline(always)]
fn subsys_bit(subsys: &str) -> u64 {
    if subsys.eq_ignore_ascii_case("PS2") {
        1 << 0
    } else if subsys.eq_ignore_ascii_case("APP") {
        1 << 1
    } else if subsys.eq_ignore_ascii_case("APPS") {
        1 << 2
    } else if subsys.eq_ignore_ascii_case("GAMES") {
        1 << 3
    } else if subsys.eq_ignore_ascii_case("DOOM") {
        1 << 4
    } else if subsys.eq_ignore_ascii_case("BITE") {
        1 << 5
    } else if subsys.eq_ignore_ascii_case("MEM") {
        1 << 6
    } else if subsys.eq_ignore_ascii_case("SYS") {
        1 << 7
    } else if subsys.eq_ignore_ascii_case("IRQGUARD") {
        1 << 8
    } else if subsys.eq_ignore_ascii_case("VALIDATOR") {
        1 << 9
    } else if subsys.eq_ignore_ascii_case("TRIBE") {
        1 << 10
    } else {
        1 << 63
    }
}

pub fn set_min_level(level: LogLevel) {
    LOG_MIN_LEVEL.store(level as u64, Ordering::Release);
}

pub fn min_level() -> LogLevel {
    match LOG_MIN_LEVEL.load(Ordering::Acquire) as u8 {
        1 => LogLevel::Trace,
        5 => LogLevel::Debug,
        9 => LogLevel::Info,
        13 => LogLevel::Warn,
        17 => LogLevel::Error,
        21 => LogLevel::Fatal,
        _ => LogLevel::Info,
    }
}

pub fn set_subsys_enabled(subsys: &str, enabled: bool) {
    let bit = subsys_bit(subsys);
    if enabled {
        LOG_SUBSYS_MASK.fetch_or(bit, Ordering::AcqRel);
    } else {
        LOG_SUBSYS_MASK.fetch_and(!bit, Ordering::AcqRel);
    }
}

pub fn subsys_enabled(subsys: &str) -> bool {
    let bit = subsys_bit(subsys);
    LOG_SUBSYS_MASK.load(Ordering::Acquire) & bit != 0
}

pub fn set_format(format: LogFormat) {
    LOG_FORMAT.store(format as u64, Ordering::Release);
}

pub fn format() -> LogFormat {
    match LOG_FORMAT.load(Ordering::Acquire) as u8 {
        1 => LogFormat::Canonical,
        2 => LogFormat::Compact,
        3 => LogFormat::LogFmt,
        _ => LogFormat::Canonical,
    }
}

pub fn set_dedup_enabled(enabled: bool) {
    LOG_DEDUP_ENABLED.store(if enabled { 1 } else { 0 }, Ordering::Release);
    if !enabled {
        LAST_EVENT_HASH.store(0, Ordering::Release);
        LAST_EVENT_SUPPRESSED.store(0, Ordering::Release);
    }
}

pub fn dedup_enabled() -> bool {
    LOG_DEDUP_ENABLED.load(Ordering::Acquire) != 0
}

#[inline(always)]
fn read_tsc() -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        core::arch::asm!(
            "rdtsc",
            out("eax") lo,
            out("edx") hi,
            options(nostack, nomem)
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

#[inline(always)]
fn fnv1a_extend(mut hash: u64, bytes: &[u8]) -> u64 {
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn event_hash(
    module: &str,
    line: u32,
    level: LogLevel,
    subsys: &str,
    event: &str,
    body: &str,
) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    hash = fnv1a_extend(hash, module.as_bytes());
    hash = fnv1a_extend(hash, &line.to_le_bytes());
    hash = fnv1a_extend(hash, &[level as u8]);
    hash = fnv1a_extend(hash, subsys.as_bytes());
    hash = fnv1a_extend(hash, event.as_bytes());
    fnv1a_extend(hash, body.as_bytes())
}

fn write_logfmt_value<W: core::fmt::Write>(out: &mut W, value: &str) -> core::fmt::Result {
    let needs_quotes = value
        .bytes()
        .any(|b| matches!(b, b' ' | b'\t' | b'=' | b'"'));
    if !needs_quotes {
        return out.write_str(value);
    }

    out.write_str("\"")?;
    for ch in value.chars() {
        match ch {
            '"' => out.write_str("\\\"")?,
            '\\' => out.write_str("\\\\")?,
            _ => out.write_char(ch)?,
        }
    }
    out.write_str("\"")
}

fn flush_suppressed_summary(serial: &mut Uart16550Tty<PioBackend>) {
    use core::fmt::Write;

    let suppressed = LAST_EVENT_SUPPRESSED.swap(0, Ordering::AcqRel);
    if suppressed == 0 {
        return;
    }
    LAST_EVENT_HASH.store(0, Ordering::Release);

    let seq = SERIAL_EVENT_SEQ.fetch_add(1, Ordering::Relaxed) + 1;
    let tsc = read_tsc();
    let format = format();
    match format {
        LogFormat::Canonical => {
            write!(
                serial,
                "[#{} tsc={:#x} irq=0 owner=Shell src=neroshiza_dev_os::serial:0 level=DEBUG subsys=LOG event=DEDUP] [LOG][DEDUP] suppressed={}\n",
                seq,
                tsc,
                suppressed
            )
            .expect("Printing dedup summary failed");
        }
        LogFormat::Compact => {
            write!(
                serial,
                "#{}/DEBUG LOG DEDUP suppressed={} @ {:#x}\n",
                seq, suppressed, tsc
            )
            .expect("Printing dedup summary failed");
        }
        LogFormat::LogFmt => {
            write!(
                serial,
                "seq={} tsc={:#x} level=DEBUG subsys=LOG event=DEDUP suppressed={} msg=\"repeated trace/debug lines collapsed\"\n",
                seq,
                tsc,
                suppressed
            )
            .expect("Printing dedup summary failed");
        }
    }
}

fn write_event_line(
    serial: &mut Uart16550Tty<PioBackend>,
    seq: u64,
    tsc: u64,
    irq: u8,
    owner: &str,
    module: &str,
    line: u32,
    level: LogLevel,
    subsys: &str,
    event: &str,
    body: &str,
) {
    use core::fmt::Write;

    match format() {
        LogFormat::Canonical => {
            write!(
                serial,
                "[#{} tsc={:#x} irq={} owner={} src={}:{} level={} subsys={} event={}] {}\n",
                seq,
                tsc,
                irq,
                owner,
                module,
                line,
                level.as_str(),
                subsys,
                event,
                body
            )
            .expect("Printing canonical serial line failed");
        }
        LogFormat::Compact => {
            write!(
                serial,
                "#{}/{} {}/{} {}:{} {}\n",
                seq,
                level.as_str(),
                subsys,
                event,
                module,
                line,
                body
            )
            .expect("Printing compact serial line failed");
        }
        LogFormat::LogFmt => {
            write!(serial, "seq={} tsc={:#x} irq={} owner=", seq, tsc, irq)
                .expect("Printing logfmt prefix failed");
            write_logfmt_value(serial, owner).expect("Printing logfmt owner failed");
            write!(serial, " src=").expect("Printing logfmt src key failed");
            let mut src = FixedBuf::<192>::new();
            use core::fmt::Write as _;
            write!(src, "{}:{}", module, line).expect("Formatting logfmt src failed");
            write_logfmt_value(serial, src.as_str()).expect("Printing logfmt src failed");
            write!(serial, " level={} subsys=", level.as_str())
                .expect("Printing logfmt level failed");
            write_logfmt_value(serial, subsys).expect("Printing logfmt subsys failed");
            write!(serial, " event=").expect("Printing logfmt event key failed");
            write_logfmt_value(serial, event).expect("Printing logfmt event failed");
            write!(serial, " msg=").expect("Printing logfmt msg key failed");
            write_logfmt_value(serial, body).expect("Printing logfmt msg failed");
            serial
                .write_str("\n")
                .expect("Printing logfmt newline failed");
        }
    }
}

#[doc(hidden)]
pub fn _println_ctx(module: &str, line: u32, args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        raw_serial_probe(b'0');
        let mut body = FixedBuf::<512>::new();
        body.write_fmt(args).expect("Formatting serial body failed");
        let body = body.as_str();
        raw_serial_probe(b'1');
        let (level, subsys, event) = classify_body(body);
        if (level as u64) < LOG_MIN_LEVEL.load(Ordering::Acquire) || !subsys_enabled(subsys) {
            raw_serial_probe(b'X');
            return;
        }
        raw_serial_probe(b'2');
        let hash = event_hash(module, line, level, subsys, event, body);
        if dedup_enabled() && level <= LogLevel::Debug {
            let last_hash = LAST_EVENT_HASH.load(Ordering::Acquire);
            if last_hash == hash {
                LAST_EVENT_SUPPRESSED.fetch_add(1, Ordering::AcqRel);
                raw_serial_probe(b'D');
                return;
            }
        }
        raw_serial_probe(b'3');
        let seq = SERIAL_EVENT_SEQ.fetch_add(1, Ordering::Relaxed) + 1;
        let tsc = read_tsc();
        let irq = if crate::irq_guard::is_in_irq() { 1 } else { 0 };
        raw_serial_probe(b'4');
        let owner = input_owner_name();
        raw_serial_probe(b'5');
        let mut serial = SERIAL1.lock();
        raw_serial_probe(b'6');
        flush_suppressed_summary(&mut serial);
        raw_serial_probe(b'7');
        write_event_line(
            &mut serial,
            seq,
            tsc,
            irq,
            owner,
            module,
            line,
            level,
            subsys,
            event,
            body,
        );
        raw_serial_probe(b'8');
        LAST_EVENT_HASH.store(hash, Ordering::Release);
    });
}

/// Выводит в хост через serial (без переноса строки).
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Выводит в хост через serial (с переносом строки).
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial::_println_ctx(module_path!(), line!(), format_args!("")));
    ($fmt:expr) => ($crate::serial::_println_ctx(module_path!(), line!(), format_args!($fmt)));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial::_println_ctx(module_path!(), line!(), format_args!($fmt, $($arg)*)));
}
