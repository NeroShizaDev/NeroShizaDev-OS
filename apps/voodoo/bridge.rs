// ============================================================
// VOODOO BRIDGE — мост к большой LLM на хосте через COM1
// ============================================================
// Два мозга — один судья:
//   * voodoo_nn (в ядре)      — быстрое «ухо», работает всегда;
//   * LLM на хосте (llama.cpp) — глубокое «ухо» и голос оракула;
//   * VoodooEngine             — единственный, кто меняет состояние игры.
//
// Протокол — строки UTF-8 с префиксом "@VDO " поверх обычного serial-лога
// (хост всё, что без префикса, просто пишет в лог):
//
//   ядро → хост   @VDO PING <lang>
//                 @VDO REQ <id> <json>         контекст хода, need=voice|meaning
//   хост → ядро   @VDO HELLO <модель>
//                 @VDO ACT <id> answer <v‰>     LLM поняла ответ (шкала да/нет)
//                 @VDO ACT <id> hint <w‰> <p0‰> .. <p4‰>   подсказка + правдоподобия
//                 @VDO ACT <id> none            болтовня, движок не трогаем
//                 @VDO SAY <id> <текст>         реплика оракула (всегда последней)
//                 @VDO CAND <id> <k> <t×100> <смысл‰> <гладкость‰> <новизна‰> <текст>
//                 @VDO PICK <id>                версии кончились — выбирай (вместо SAY)
//                 @VDO ERR <id> <причина>
//   ядро → хост   @VDO PICKED <id> <k>          что выбрал интерпретатор (для новизны)
//
// CAND/PICK — режим интерпретатора (interp.rs): хост шлёт несколько версий
// реплики на разных температурах, движок отсеивает бред и выбирает живую.
//
// COM1 опрашивается из главного цикла (hlt_loop → process_deferred_actions),
// а не из IRQ. Разбор строки и работа с сессией идут под without_interrupts,
// чтобы клавиатурный IRQ не застал SESSION/SHELL посреди изменения.
// ============================================================

use crate::kernel_messages::Locale;
use crate::locale;
use crate::voodoo_engine::CHAR_COUNT;
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

use super::chat;
use super::interp::{self, Candidate, SLOTS};

const COM1_DATA: u16 = 0x3F8;
const COM1_LSR: u16 = 0x3FD;
const LSR_DATA_READY: u8 = 0x01;

const PREFIX: &[u8] = b"@VDO ";
const LINE_MAX: usize = 768;
const PACKET_MAX: usize = 1024;
const MODEL_MAX: usize = 48;
/// Байт за один опрос — чтобы главный цикл не залипал на потоке логов.
const BYTES_PER_POLL: usize = 256;
/// Сколько ждём ответа, прежде чем считать запрос потерянным.
const TIMEOUT_SECS: u32 = 45;
/// Столько потерь подряд — и мост считается упавшим.
const MAX_MISSES: u32 = 2;

pub const C_SPIRIT: u8 = 0x09;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Need {
    /// Ход уже обработан ядром — LLM только комментирует.
    Voice,
    /// Сеть в ядре не поняла фразу — LLM должна её истолковать (ACT), потом сказать.
    Meaning,
}

impl Need {
    fn as_str(self) -> &'static str {
        match self {
            Need::Voice => "voice",
            Need::Meaning => "meaning",
        }
    }
}

/// Что LLM поняла из непонятой ядром фразы.
#[derive(Clone, Copy)]
pub enum Act {
    Answer(f32),
    Hint {
        weight: f32,
        persona: [f32; CHAR_COUNT],
    },
    Nothing,
}

#[derive(Clone, Copy)]
struct Pending {
    id: u32,
    need: Need,
    since: u32,
    acted: bool,
    /// LLM сочла фразу болтовнёй: после её реплики повторить вопрос.
    reprompt_after_say: bool,
}

struct Link {
    online: bool,
    model: [u8; MODEL_MAX],
    model_len: usize,
    next_id: u32,
    pending: Option<Pending>,
    misses: u32,
    sent: u32,
    said: u32,
    acts: u32,
    timeouts: u32,
    errors: u32,
    line: [u8; LINE_MAX],
    line_len: usize,
    overflow: bool,
}

impl Link {
    const fn new() -> Self {
        Self {
            online: false,
            model: [0; MODEL_MAX],
            model_len: 0,
            next_id: 1,
            pending: None,
            misses: 0,
            sent: 0,
            said: 0,
            acts: 0,
            timeouts: 0,
            errors: 0,
            line: [0; LINE_MAX],
            line_len: 0,
            overflow: false,
        }
    }

    fn model(&self) -> &str {
        core::str::from_utf8(&self.model[..self.model_len]).unwrap_or("?")
    }
}

static LINK: Mutex<Link> = Mutex::new(Link::new());

/// Версии реплики текущего запроса (режим интерпретатора).
const CAND_TEXT: usize = 384;

struct Cands {
    id: u32,
    meta: [Option<Candidate>; SLOTS],
    text: [[u8; CAND_TEXT]; SLOTS],
    len: [usize; SLOTS],
}

static CANDS: Mutex<Cands> = Mutex::new(Cands {
    id: 0,
    meta: [None; SLOTS],
    text: [[0; CAND_TEXT]; SLOTS],
    len: [0; SLOTS],
});
/// Опрашивать ли COM1. Включается только на время сеанса с оракулом,
/// чтобы не съесть байты, предназначенные `install serial`.
static LISTENING: AtomicBool = AtomicBool::new(false);

// ============================================================
// Публичный API для chat.rs
// ============================================================

/// Начало/конец сеанса. На старте — PING: хост отвечает HELLO.
pub fn listen(on: bool) {
    LISTENING.store(on, Ordering::Release);
    interrupts::without_interrupts(|| {
        let mut link = LINK.lock();
        link.pending = None;
        link.line_len = 0;
        link.overflow = false;
    });
    if on {
        crate::serial::_print(format_args!("@VDO PING {}\n", lang_code()));
    }
}

pub fn online() -> bool {
    interrupts::without_interrupts(|| LINK.lock().online)
}

/// Ждём ли сейчас, что LLM истолкует фразу.
pub fn awaiting_meaning() -> bool {
    interrupts::without_interrupts(
        || matches!(LINK.lock().pending, Some(p) if p.need == Need::Meaning && !p.acted),
    )
}

/// Отправляет контекст хода. `fill` дописывает поля в JSON-объект.
/// Новый запрос вытесняет старый: опоздавшие ответы игнорируются.
pub fn request(need: Need, fill: impl FnOnce(&mut Packet)) -> bool {
    let mut packet = Packet::new();
    packet.str("lang", lang_code()).str("need", need.as_str());
    fill(&mut packet);
    let body = packet.finish();

    interrupts::without_interrupts(|| {
        let mut link = LINK.lock();
        if !link.online {
            return false;
        }
        let id = link.next_id;
        link.next_id = link.next_id.wrapping_add(1).max(1);
        link.pending = Some(Pending {
            id,
            need,
            since: now_secs(),
            acted: false,
            reprompt_after_say: false,
        });
        link.sent += 1;
        crate::serial::_print(format_args!("@VDO REQ {} {}\n", id, body));
        true
    })
}

/// Опрос COM1 из главного цикла.
pub fn poll() {
    if !LISTENING.load(Ordering::Acquire) {
        return;
    }
    interrupts::without_interrupts(|| {
        let mut lsr: Port<u8> = Port::new(COM1_LSR);
        let mut data: Port<u8> = Port::new(COM1_DATA);
        for _ in 0..BYTES_PER_POLL {
            // SAFETY: COM1 LSR/RBR — стандартные порты UART; чтение без побочных эффектов,
            // кроме снятия байта из FIFO приёмника. 0xFF = порта нет (плавающая шина).
            let status = unsafe { lsr.read() };
            if status == 0xFF || status & LSR_DATA_READY == 0 {
                break;
            }
            let byte = unsafe { data.read() };
            feed(byte);
        }
        check_timeout();
    });
}

/// `voodoo bridge` — состояние моста.
pub fn print_status() {
    interrupts::without_interrupts(|| {
        let link = LINK.lock();
        let listening = LISTENING.load(Ordering::Acquire);
        if link.online {
            match locale::get_locale() {
                Locale::RuRu => locale::print_localized_fmt(
                    C_SPIRIT,
                    format_args!("[МОСТ] на связи: {} (COM1)", link.model()),
                ),
                _ => locale::print_localized_fmt(
                    C_SPIRIT,
                    format_args!("[BRIDGE] online: {} (COM1)", link.model()),
                ),
            }
        } else {
            match locale::get_locale() {
                Locale::RuRu => locale::print_localized_line(
                    "[МОСТ] нет связи. На хосте: python tools/voodoo_bridge/bridge.py --qemu",
                    0x08,
                ),
                _ => locale::print_localized_line(
                    "[BRIDGE] offline. On the host: python tools/voodoo_bridge/bridge.py --qemu",
                    0x08,
                ),
            }
        }
        locale::print_localized_fmt(
            0x0E,
            format_args!(
                "  sent {} | said {} | acts {} | timeouts {} | errors {} | listening {}",
                link.sent, link.said, link.acts, link.timeouts, link.errors, listening
            ),
        );
    });
}

// ============================================================
// Приём
// ============================================================

fn feed(byte: u8) {
    let mut link = LINK.lock();
    if byte == b'\n' {
        let len = link.line_len;
        let overflow = link.overflow;
        link.line_len = 0;
        link.overflow = false;
        if overflow || len < PREFIX.len() || &link.line[..PREFIX.len()] != PREFIX {
            return;
        }
        let mut line = [0u8; LINE_MAX];
        line[..len].copy_from_slice(&link.line[..len]);
        drop(link);
        handle_line(&line[PREFIX.len()..len]);
        return;
    }
    if byte == b'\r' {
        return;
    }
    if link.line_len < LINE_MAX {
        let at = link.line_len;
        link.line[at] = byte;
        link.line_len += 1;
    } else {
        link.overflow = true;
    }
}

fn handle_line(line: &[u8]) {
    let Ok(text) = core::str::from_utf8(line) else {
        return;
    };
    let (verb, rest) = split_word(text);
    match verb {
        "HELLO" => on_hello(rest.trim()),
        "SAY" => {
            let (id, text) = split_word(rest);
            if let Some(id) = parse_u32(id) {
                on_say(id, text.trim());
            }
        }
        "ACT" => {
            let (id, rest) = split_word(rest);
            if let (Some(id), Some(act)) = (parse_u32(id), parse_act(rest)) {
                on_act(id, act);
            }
        }
        "ERR" => {
            let (id, reason) = split_word(rest);
            if let Some(id) = parse_u32(id) {
                on_err(id, reason.trim());
            }
        }
        "CAND" => on_cand(rest),
        "PICK" => {
            if let Some(id) = parse_u32(split_word(rest).0) {
                on_pick(id);
            }
        }
        _ => {}
    }
}

/// `CAND <id> <k> <t×100> <смысл‰> <гладкость‰> <новизна‰> <текст>`
fn on_cand(rest: &str) {
    let mut fields = [0u32; 6];
    let mut rest = rest;
    for f in fields.iter_mut() {
        let (word, tail) = split_word(rest);
        let Some(v) = parse_u32(word) else {
            return;
        };
        *f = v;
        rest = tail;
    }
    let [id, k, t, sense, fluency, novelty] = fields;
    let k = k as usize;
    if k >= SLOTS || take_current(id, false).is_none() {
        return;
    }
    let text = rest.trim();
    let mut n = text.len().min(CAND_TEXT);
    while n > 0 && !text.is_char_boundary(n) {
        n -= 1;
    }
    let mut cands = CANDS.lock();
    if cands.id != id {
        cands.id = id;
        cands.meta = [None; SLOTS];
    }
    cands.text[k][..n].copy_from_slice(&text.as_bytes()[..n]);
    cands.len[k] = n;
    cands.meta[k] = Some(Candidate {
        temp: t as f32 / 100.0,
        sense: sense.min(1000) as f32 / 1000.0,
        fluency: fluency.min(1000) as f32 / 1000.0,
        novelty: novelty.min(1000) as f32 / 1000.0,
    });
}

/// Версии собраны — интерпретатор выбирает, дух говорит выбранную.
fn on_pick(id: u32) {
    let Some(pending) = take_current(id, true) else {
        return;
    };
    let (meta, text, len) = {
        let mut cands = CANDS.lock();
        if cands.id != id {
            return;
        }
        let snapshot = (cands.meta, cands.text, cands.len);
        cands.meta = [None; SLOTS];
        snapshot
    };
    let verdict = interp::decide(&meta);
    LINK.lock().said += 1;
    crate::shell::interject(|| {
        if let Some(v) = &verdict {
            let chosen = core::str::from_utf8(&text[v.pick][..len[v.pick]]).unwrap_or("");
            if !chosen.is_empty() {
                print_spirit(chosen);
            }
            interp::print_verdict(&meta, v);
        }
        if pending.reprompt_after_say {
            chat::reprompt_current();
        }
    });
    if let Some(v) = verdict {
        crate::serial::_print(format_args!("@VDO PICKED {} {}\n", id, v.pick));
    }
}

fn on_hello(model: &str) {
    let first = {
        let mut link = LINK.lock();
        let first = !link.online;
        link.online = true;
        link.misses = 0;
        let bytes = model.as_bytes();
        let mut n = bytes.len().min(MODEL_MAX);
        while n > 0 && !model.is_char_boundary(n) {
            n -= 1;
        }
        link.model[..n].copy_from_slice(&bytes[..n]);
        link.model_len = n;
        first
    };
    if !first || !chat::is_active() {
        return;
    }
    crate::shell::interject(|| {
        match locale::get_locale() {
            Locale::RuRu => locale::print_localized_fmt(
                C_SPIRIT,
                format_args!("[МОСТ] Дух на связи: {}", model),
            ),
            _ => locale::print_localized_fmt(
                C_SPIRIT,
                format_args!("[BRIDGE] Spirit online: {}", model),
            ),
        }
        chat::on_bridge_online();
    });
}

/// Ответ относится к текущему запросу? Опоздавшие (вытесненные) — мимо.
fn take_current(id: u32, finish: bool) -> Option<Pending> {
    let mut link = LINK.lock();
    let pending = link.pending.filter(|p| p.id == id)?;
    link.misses = 0;
    if finish {
        link.pending = None;
    }
    Some(pending)
}

fn on_say(id: u32, text: &str) {
    let Some(pending) = take_current(id, true) else {
        return;
    };
    LINK.lock().said += 1;
    crate::shell::interject(|| {
        if !text.is_empty() {
            print_spirit(text);
        }
        if pending.reprompt_after_say {
            chat::reprompt_current();
        }
    });
}

fn on_act(id: u32, act: Act) {
    let Some(pending) = take_current(id, false) else {
        return;
    };
    if pending.need != Need::Meaning || pending.acted {
        return;
    }
    {
        let mut link = LINK.lock();
        link.acts += 1;
        if let Some(p) = link.pending.as_mut() {
            p.acted = true;
            p.reprompt_after_say = matches!(act, Act::Nothing);
        }
    }
    // Болтовня: движок не трогаем, вопрос повторим после реплики духа
    if !matches!(act, Act::Nothing) {
        crate::shell::interject(|| chat::apply_act(act));
    }
}

fn on_err(id: u32, reason: &str) {
    let Some(pending) = take_current(id, true) else {
        return;
    };
    LINK.lock().errors += 1;
    crate::serial_println!("[VOODOO][BRIDGE] host error for {}: {}", id, reason);
    if pending.need == Need::Meaning && !pending.acted {
        crate::shell::interject(|| chat::apply_act(Act::Nothing));
    }
}

fn check_timeout() {
    let expired = {
        let mut link = LINK.lock();
        let Some(p) = link.pending else {
            return;
        };
        let elapsed = (now_secs() + 86_400 - p.since) % 86_400;
        if elapsed < TIMEOUT_SECS {
            return;
        }
        link.pending = None;
        link.timeouts += 1;
        link.misses += 1;
        let dropped = link.misses >= MAX_MISSES;
        if dropped {
            link.online = false;
        }
        (p, dropped)
    };
    let (pending, dropped) = expired;
    crate::shell::interject(|| {
        if dropped {
            match locale::get_locale() {
                Locale::RuRu => {
                    locale::print_localized_line("[МОСТ] Дух молчит - играю сам.", 0x08)
                }
                _ => locale::print_localized_line(
                    "[BRIDGE] The spirit is silent - going solo.",
                    0x08,
                ),
            }
        }
        if pending.need == Need::Meaning && !pending.acted {
            chat::apply_act(Act::Nothing);
        }
    });
}

// ============================================================
// Вывод реплики оракула с переносом по словам
// ============================================================

fn print_spirit(text: &str) {
    let label = match locale::get_locale() {
        Locale::RuRu => "[ДУХ] ",
        Locale::EnUs => "[SPIRIT] ",
        Locale::ArEg => "[روح] ",
    };
    let indent = label.chars().count();
    let width = crate::fb_buffer::default_text_cols()
        .saturating_sub(indent + 1)
        .max(16);

    let mut rest = text;
    let mut first = true;
    let mut lines = 0;
    while !rest.is_empty() && lines < 6 {
        let cut = wrap_point(rest, width);
        let (line, tail) = rest.split_at(cut);
        locale::print_localized_fmt(
            C_SPIRIT,
            format_args!(
                "{:<w$}{}",
                if first { label } else { "" },
                line.trim_end(),
                w = indent
            ),
        );
        rest = tail.trim_start();
        first = false;
        lines += 1;
    }
}

/// Байтовый индекс, где резать строку не длиннее `width` символов — по пробелу,
/// если он есть во второй половине строки, иначе жёстко.
fn wrap_point(s: &str, width: usize) -> usize {
    let mut last_space = None;
    for (count, (i, ch)) in s.char_indices().enumerate() {
        if count == width {
            return match last_space {
                Some(sp) if sp * 2 >= width => sp,
                _ => i,
            };
        }
        if ch == ' ' {
            last_space = Some(i);
        }
    }
    s.len()
}

// ============================================================
// JSON-пакет запроса (без аллокаций)
// ============================================================

pub struct Packet {
    buf: [u8; PACKET_MAX],
    len: usize,
    fields: usize,
}

impl Packet {
    fn new() -> Self {
        let mut p = Self {
            buf: [0; PACKET_MAX],
            len: 0,
            fields: 0,
        };
        p.push(b"{");
        p
    }

    fn push(&mut self, bytes: &[u8]) {
        // Последний байт держим под закрывающую скобку
        let free = (PACKET_MAX - 1).saturating_sub(self.len);
        let take = bytes.len().min(free);
        self.buf[self.len..self.len + take].copy_from_slice(&bytes[..take]);
        self.len += take;
    }

    fn key(&mut self, key: &str) {
        if self.fields > 0 {
            self.push(b",");
        }
        self.fields += 1;
        self.push(b"\"");
        self.push(key.as_bytes());
        self.push(b"\":");
    }

    fn escaped_char(&mut self, ch: char) {
        match ch {
            '"' => self.push(b"\\\""),
            '\\' => self.push(b"\\\\"),
            c if (c as u32) < 0x20 => self.push(b" "),
            c => {
                let mut tmp = [0u8; 4];
                self.push(c.encode_utf8(&mut tmp).as_bytes());
            }
        }
    }

    pub fn str(&mut self, key: &str, value: &str) -> &mut Self {
        self.key(key);
        self.push(b"\"");
        for ch in value.chars() {
            self.escaped_char(ch);
        }
        self.push(b"\"");
        self
    }

    pub fn cps(&mut self, key: &str, value: &[u32]) -> &mut Self {
        self.key(key);
        self.push(b"\"");
        for &cp in value {
            self.escaped_char(char::from_u32(cp).unwrap_or('?'));
        }
        self.push(b"\"");
        self
    }

    pub fn num(&mut self, key: &str, value: u32) -> &mut Self {
        self.key(key);
        let _ = write!(Raw(self), "{}", value);
        self
    }

    /// Строковое поле из format_args! (экранируется на лету).
    pub fn fmt(&mut self, key: &str, args: fmt::Arguments) -> &mut Self {
        self.key(key);
        self.push(b"\"");
        let _ = Escaped(self).write_fmt(args);
        self.push(b"\"");
        self
    }

    fn finish(&mut self) -> &str {
        self.buf[self.len] = b'}';
        self.len += 1;
        // push() режет по байтам — откатываемся до целого UTF-8
        let mut end = self.len;
        while core::str::from_utf8(&self.buf[..end]).is_err() && end > 0 {
            end -= 1;
        }
        core::str::from_utf8(&self.buf[..end]).unwrap_or("{}")
    }
}

struct Raw<'a>(&'a mut Packet);

impl Write for Raw<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.push(s.as_bytes());
        Ok(())
    }
}

struct Escaped<'a>(&'a mut Packet);

impl Write for Escaped<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() {
            self.0.escaped_char(ch);
        }
        Ok(())
    }
}

// ============================================================
// Мелочи
// ============================================================

fn lang_code() -> &'static str {
    match locale::get_locale() {
        Locale::RuRu => "ru",
        Locale::EnUs => "en",
        Locale::ArEg => "ar",
    }
}

/// Секунды от полуночи по CMOS RTC — для таймаутов (PIT в ядре не откалиброван).
fn now_secs() -> u32 {
    let t = crate::apps::rtc::read_rtc();
    t.hours as u32 * 3600 + t.minutes as u32 * 60 + t.seconds as u32
}

fn split_word(s: &str) -> (&str, &str) {
    let s = s.trim_start();
    match s.find(' ') {
        Some(i) => (&s[..i], &s[i + 1..]),
        None => (s, ""),
    }
}

fn parse_u32(s: &str) -> Option<u32> {
    if s.is_empty() || s.len() > 9 {
        return None;
    }
    let mut v = 0u32;
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        v = v * 10 + (b - b'0') as u32;
    }
    Some(v)
}

/// Промилле → доля [0, 1].
fn permille(s: &str) -> Option<f32> {
    parse_u32(s).map(|v| v.min(1000) as f32 / 1000.0)
}

fn parse_act(s: &str) -> Option<Act> {
    let (kind, mut rest) = split_word(s);
    match kind {
        "answer" => permille(split_word(rest).0).map(Act::Answer),
        "hint" => {
            let (w, tail) = split_word(rest);
            let weight = permille(w)?;
            rest = tail;
            let mut persona = [0.0f32; CHAR_COUNT];
            for slot in persona.iter_mut() {
                let (p, tail) = split_word(rest);
                *slot = permille(p)?;
                rest = tail;
            }
            Some(Act::Hint { weight, persona })
        }
        "none" => Some(Act::Nothing),
        _ => None,
    }
}
