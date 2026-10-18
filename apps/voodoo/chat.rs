// ============================================================
// VOODOO CHAT — Акинатор Психотауна поверх нейро-слоя
// ============================================================
// Сессия живёт в шелле: пока она активна, каждая введённая строка
// уходит сюда, а не в обработчик команд.
//
//   строка ─► voodoo_nn::read  → Reading (распределения интентов/персон)
//          ─► интент:
//               да/нет/хз  → answer_question / проверка догадки
//               подсказка  → absorb_neural (правдоподобие от сети)
//               почему     → расклад апостериори
//               заново/стоп/привет → управление
//          ─► следующий вопрос или «Это X?»
//          ─► bridge::request: контекст хода уходит LLM на хосте (если мост жив)
//
// Если сеть в ядре не уверена, а мост жив — фразу толкует LLM (bridge::Act),
// и результат идёт в движок теми же путями, что и ответы/подсказки сети.
//
// Вызывается из IRQ клавиатуры: никаких аллокаций, всё на стеке.
// ============================================================

#![allow(dead_code)]

use crate::kernel_messages::Locale;
use crate::locale;
use crate::voodoo_engine::{self as engine, CHAR_COUNT, Q_COUNT, VoodooEngine};
use crate::voodoo_nn::{self, INTENT_COUNT, Intent, Reading};
use spin::Mutex;

use super::bridge::{self, Act, Need};
use super::golden::GOLDEN;

const C_TITLE: u8 = 0x0B;
const C_TEXT: u8 = 0x0E;
const C_ASK: u8 = 0x0F;
const C_GUESS: u8 = 0x0D;
const C_OK: u8 = 0x0A;
const C_ERR: u8 = 0x0C;
const C_DIM: u8 = 0x08;

/// Ниже этой уверенности топ-интента оракул переспрашивает.
const MIN_CONFIDENCE: f32 = 0.40;
/// Подсказка, где сеть не уверена, о ком она, — при живом мосте её толкует LLM.
/// («в белом халате по ночам» сеть уверенно, но зря тянет к Архитектору: «по ночам».)
const PERSONA_CONFIDENT: f32 = 0.75;

fn needs_spirit(r: &Reading) -> bool {
    let (intent, p) = r.top_intent();
    p < MIN_CONFIDENCE || (intent == Intent::Hint && r.top_persona().1 < PERSONA_CONFIDENT)
}
const BAR_WIDTH: usize = 20;

macro_rules! say {
    ($color:expr, $ru:literal, $en:literal, $ar:literal $(, $arg:expr)* $(,)?) => {
        match locale::get_locale() {
            Locale::RuRu => locale::print_localized_fmt($color, format_args!($ru $(, $arg)*)),
            Locale::EnUs => locale::print_localized_fmt($color, format_args!($en $(, $arg)*)),
            Locale::ArEg => locale::print_localized_fmt($color, format_args!($ar $(, $arg)*)),
        }
    };
}

// ============================================================
// Сессия
// ============================================================
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Asking(usize),
    Guessing(usize),
    Finished,
}

enum Flow {
    Continue,
    Close,
}

/// Что произошло на этом ходу — для LLM-голоса.
#[derive(Clone, Copy)]
enum Event {
    Start,
    Answer(f32),
    Hint,
    Win(usize),
    Miss(usize),
    GiveUp,
    Unsure,
    Explain,
    Greet,
    Restart,
    Again,
    Unclear,
}

impl Event {
    fn name(self) -> &'static str {
        match self {
            Event::Start => "start",
            Event::Answer(_) => "answer",
            Event::Hint => "hint",
            Event::Win(_) => "win",
            Event::Miss(_) => "miss",
            Event::GiveUp => "giveup",
            Event::Unsure => "unsure",
            Event::Explain => "explain",
            Event::Greet => "greet",
            Event::Restart => "restart",
            Event::Again => "again",
            Event::Unclear => "unclear",
        }
    }
}

struct Session {
    engine: VoodooEngine,
    phase: Phase,
    round: u32,
    wins: u32,
    losses: u32,
    event: Event,
}

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

pub fn is_active() -> bool {
    SESSION.lock().is_some()
}

pub fn start() {
    let mut guard = SESSION.lock();
    if let Some(session) = guard.as_mut() {
        say!(
            C_DIM,
            "[VOODOO] Оракул уже на связи.",
            "[VOODOO] The oracle is already listening.",
            "[VOODOO] العراف يستمع بالفعل."
        );
        session.reprompt();
        return;
    }

    print_banner();
    let mut session = Session {
        engine: VoodooEngine::new(),
        phase: Phase::Finished,
        round: 1,
        wins: 0,
        losses: 0,
        event: Event::Start,
    };
    session.advance();
    // listen() сбрасывает ожидающий запрос — поэтому до send().
    bridge::listen(true);
    // Мост уже жив с прошлого сеанса — дух здоровается сразу,
    // иначе поздоровается, когда на PING придёт HELLO.
    session.send(Need::Voice, &[], None);
    *guard = Some(session);
}

pub fn stop() {
    if SESSION.lock().take().is_some() {
        bridge::listen(false);
        print_bye();
    }
}

/// Мост только что ожил (HELLO) — дух здоровается.
pub fn on_bridge_online() {
    if let Some(session) = SESSION.lock().as_mut() {
        session.event = Event::Start;
        session.send(Need::Voice, &[], None);
    }
}

/// LLM истолковала фразу, которую не поняла сеть в ядре.
pub fn apply_act(act: Act) {
    let mut guard = SESSION.lock();
    let Some(session) = guard.as_mut() else {
        return;
    };
    let flow = match act {
        Act::Answer(v) => {
            say!(
                C_DIM,
                "  дух[llm]: ответ -> {:.2}",
                "  spirit[llm]: answer -> {:.2}",
                "  spirit[llm]: answer -> {:.2}",
                v
            );
            session.on_answer(v)
        }
        Act::Hint { weight, persona } => {
            let c = argmax(&persona);
            say!(
                C_DIM,
                "  дух[llm]: подсказка {}% -> {} {}%",
                "  spirit[llm]: hint {}% -> {} {}%",
                "  spirit[llm]: hint {}% -> {} {}%",
                pct(weight),
                persona_short(c),
                pct(persona[c])
            );
            session.on_hint(&persona, weight)
        }
        Act::Nothing => {
            not_understood();
            session.reprompt();
            Flow::Continue
        }
    };
    if let Flow::Close = flow {
        *guard = None;
        bridge::listen(false);
        print_bye();
    }
}

/// Повтор текущего вопроса/догадки (после реплики духа на болтовню).
pub fn reprompt_current() {
    if let Some(session) = SESSION.lock().as_ref() {
        session.reprompt();
    }
}

/// Строка, введённая во время сессии.
pub fn on_line(input: &[u32]) {
    let mut guard = SESSION.lock();
    let Some(session) = guard.as_mut() else {
        return;
    };

    let reading = voodoo_nn::read(input);
    if let Some(r) = &reading {
        print_reading(r);
    }

    let reading = match reading {
        Some(r) if !needs_spirit(&r) => r,
        other => {
            // Сеть в ядре не уверена — спрашиваем большую модель, если она на связи
            session.event = Event::Unclear;
            if session.send(Need::Meaning, input, other.as_ref()) {
                say!(
                    C_DIM,
                    "[VOODOO] Сеть не уверена - спрашиваю духа...",
                    "[VOODOO] The net is unsure - asking the spirit...",
                    "[VOODOO] الشبكة غير متاكدة - اسال الروح..."
                );
                return;
            }
            // Моста нет: неясную персону всё равно берём у сети, мусор — переспрашиваем
            match other {
                Some(r) if r.top_intent().1 >= MIN_CONFIDENCE => r,
                _ => {
                    not_understood();
                    session.reprompt();
                    return;
                }
            }
        }
    };

    let flow = match reading.top_intent().0 {
        Intent::Yes | Intent::No | Intent::Maybe => session.on_answer(reading.answer_value()),
        Intent::Hint => session.on_hint(&reading.persona, reading.intent[Intent::Hint as usize]),
        Intent::Why => {
            session.event = Event::Explain;
            session.explain();
            session.reprompt();
            Flow::Continue
        }
        Intent::Restart => {
            session.new_round();
            Flow::Continue
        }
        Intent::Greet => {
            session.event = Event::Greet;
            say!(
                C_TEXT,
                "[VOODOO] Я Вуду: нейросеть слушает, Байес думает, автомат шумит.",
                "[VOODOO] I am Voodoo: the net listens, Bayes thinks, the automaton hums.",
                "[VOODOO] انا فودو: الشبكة تسمع وبايز يفكر والالة تضج."
            );
            session.reprompt();
            Flow::Continue
        }
        Intent::Stop => Flow::Close,
    };

    if let Flow::Close = flow {
        *guard = None;
        bridge::listen(false);
        print_bye();
        return;
    }
    session.send(Need::Voice, input, Some(&reading));
}

impl Session {
    /// Контекст хода для LLM. false — моста нет.
    fn send(&self, need: Need, input: &[u32], reading: Option<&Reading>) -> bool {
        bridge::request(need, |p| {
            p.str("event", self.event.name()).cps("user", input);
            match self.event {
                Event::Answer(v) => {
                    p.fmt("answer", format_args!("{:.2}", v));
                }
                Event::Win(c) | Event::Miss(c) => {
                    p.str("subject", engine::character_name(c));
                }
                _ => {}
            }
            if let Some(r) = reading {
                let (i, pi) = r.top_intent();
                let (c, pc) = r.top_persona();
                if i == Intent::Hint {
                    p.fmt(
                        "nn",
                        format_args!("hint {}% -> {} {}%", pct(pi), persona_short(c), pct(pc)),
                    );
                } else {
                    p.fmt(
                        "nn",
                        format_args!("{} {}%", INTENT_KEYS[i as usize], pct(pi)),
                    );
                }
            }
            match self.phase {
                Phase::Asking(q) => {
                    p.str("act", "ask")
                        .str("question", engine::question_text(q));
                }
                Phase::Guessing(c) => {
                    p.str("act", "guess")
                        .str("target", engine::character_name(c))
                        .num("confidence", pct(self.engine.probabilities[c]));
                }
                Phase::Finished => {
                    p.str("act", "finished");
                }
            }
            p.fmt("board", format_args!("{}", Board(&self.engine)))
                .num("round", self.round)
                .fmt("score", format_args!("{}:{}", self.wins, self.losses));
        })
    }

    /// Следующий шаг: догадка, если пора, иначе самый информативный вопрос.
    fn advance(&mut self) {
        if self.engine.should_guess() {
            self.make_guess();
            return;
        }
        let drops = self.engine.connection_drops;
        let mut next = self.engine.get_best_question();
        if next.is_none() && self.engine.connection_drops != drops {
            fresh_start();
            next = self.engine.get_best_question();
        }
        match next {
            Some(q) => {
                self.phase = Phase::Asking(q);
                ask(q);
            }
            None => self.make_guess(),
        }
    }

    fn make_guess(&mut self) {
        let (c, _) = self.engine.leader();
        self.phase = Phase::Guessing(c);
        self.say_guess(c);
    }

    fn say_guess(&self, c: usize) {
        let p = pct(self.engine.probabilities[c]);
        say!(
            C_GUESS,
            "[!] Это {}? (уверенность {}%)",
            "[!] Is it {}? (confidence {}%)",
            "[!] هل هو {}? (الثقة {}%)",
            engine::character_name(c),
            p
        );
    }

    fn reprompt(&self) {
        match self.phase {
            Phase::Asking(q) => ask(q),
            Phase::Guessing(c) => self.say_guess(c),
            Phase::Finished => play_again(),
        }
    }

    fn new_round(&mut self) {
        self.engine.reset();
        self.round += 1;
        self.event = Event::Restart;
        say!(
            C_TITLE,
            "[VOODOO] Раунд {}. Загадывай.",
            "[VOODOO] Round {}. Make your pick.",
            "[VOODOO] الجولة {}. اختر شخصية.",
            self.round
        );
        self.advance();
    }

    fn on_answer(&mut self, answer: f32) -> Flow {
        match self.phase {
            Phase::Asking(q) => {
                self.event = Event::Answer(answer);
                if !self.engine.answer_question(q, answer) {
                    fresh_start();
                }
                self.advance();
            }
            Phase::Guessing(c) => {
                if answer >= 0.6 {
                    self.wins += 1;
                    self.event = Event::Win(c);
                    // Без точки после имени: «К.А.Ф.И.Д.Р.А.» уже кончается точкой
                    say!(
                        C_OK,
                        "[VOODOO] Вуду не ошибается - это {}! Счёт {}:{}",
                        "[VOODOO] Voodoo never misses - it's {}! Score {}:{}",
                        "[VOODOO] فودو لا يخطئ - انه {}! النتيجة {}:{}",
                        engine::character_name(c),
                        self.wins,
                        self.losses
                    );
                    self.phase = Phase::Finished;
                    play_again();
                } else if answer <= 0.4 {
                    self.event = Event::Miss(c);
                    say!(
                        C_DIM,
                        "[VOODOO] Мимо. Вычёркиваю: {}",
                        "[VOODOO] Miss. Crossing out: {}",
                        "[VOODOO] خطا. اشطب: {}",
                        engine::character_name(c)
                    );
                    if self.engine.exclude(c) {
                        self.advance();
                    } else {
                        self.losses += 1;
                        self.event = Event::GiveUp;
                        say!(
                            C_OK,
                            "[VOODOO] Матрица пуста - сдаюсь. Счёт {}:{}",
                            "[VOODOO] The matrix is empty - I give up. Score {}:{}",
                            "[VOODOO] المصفوفة فارغة - استسلم. النتيجة {}:{}",
                            self.wins,
                            self.losses
                        );
                        self.phase = Phase::Finished;
                        play_again();
                    }
                } else {
                    self.event = Event::Unsure;
                    say!(
                        C_DIM,
                        "[VOODOO] Так да или нет?",
                        "[VOODOO] So is it yes or no?",
                        "[VOODOO] اذن نعم ام لا?"
                    );
                    self.reprompt();
                }
            }
            Phase::Finished => {
                if answer >= 0.6 {
                    self.new_round();
                } else if answer <= 0.4 {
                    return Flow::Close;
                } else {
                    self.event = Event::Again;
                    play_again();
                }
            }
        }
        Flow::Continue
    }

    /// Подсказка словами: правдоподобия персон (от сети в ядре или от LLM) с весом.
    fn on_hint(&mut self, persona: &[f32; CHAR_COUNT], weight: f32) -> Flow {
        if self.phase == Phase::Finished {
            self.event = Event::Again;
            play_again();
            return Flow::Continue;
        }
        self.event = Event::Hint;
        if self.engine.absorb_neural(persona, weight) {
            let (c, p) = self.engine.leader();
            say!(
                C_TEXT,
                "[VOODOO] Подсказку учёл: склоняюсь к {} ({}%).",
                "[VOODOO] Hint absorbed: leaning towards {} ({}%).",
                "[VOODOO] تم اخذ التلميح: اميل الى {} ({}%).",
                engine::character_name(c),
                pct(p)
            );
        } else {
            fresh_start();
        }

        match self.phase {
            Phase::Asking(q) if !self.engine.should_guess() && !self.engine.asked_questions[q] => {
                ask(q)
            }
            _ => self.advance(),
        }
        Flow::Continue
    }

    fn explain(&self) {
        let e = &self.engine;
        say!(
            C_TITLE,
            "[VOODOO] Расклад: вопросов {}/{}, подсказок {}, хаос {}%, автомат {}",
            "[VOODOO] Matrix: questions {}/{}, hints {}, chaos {}%, automaton {}",
            "[VOODOO] المصفوفة: اسئلة {}/{}, تلميحات {}, فوضى {}%, الة {}",
            e.questions_asked(),
            Q_COUNT,
            e.neural_evidence,
            pct(e.chaos_measure()),
            e.cellular_mode.name()
        );
        for c in 0..CHAR_COUNT {
            let p = e.probabilities[c];
            let mut buf = [0u8; BAR_WIDTH];
            locale::print_localized_fmt(
                if e.excluded[c] { C_DIM } else { C_TEXT },
                format_args!(
                    "  {:<24} {} {:>3}%{}",
                    engine::character_name(c),
                    bar(p, &mut buf),
                    pct(p),
                    if e.excluded[c] { "  x" } else { "" }
                ),
            );
        }
    }
}

// ============================================================
// Реплики
// ============================================================
fn print_banner() {
    say!(
        C_TITLE,
        "=== VOODOO :: нейро-оракул Психотауна ===",
        "=== VOODOO :: neural oracle of Psychotown ===",
        "=== VOODOO :: عراف بسيكوتاون العصبي ==="
    );
    print_model_line();
    say!(
        C_TEXT,
        "Загадай резидента:",
        "Pick a resident:",
        "اختر احد السكان:"
    );
    for c in 0..CHAR_COUNT {
        locale::print_localized_fmt(C_TEXT, format_args!("  - {}", engine::character_name(c)));
    }
    say!(
        C_TEXT,
        "Отвечай как хочешь: да / нет / хз - или подскажи словами.",
        "Answer any way: yes / no / maybe - or drop a hint in words.",
        "اجب كما تشاء: نعم / لا / ربما - او اعط تلميحا بالكلمات."
    );
    say!(
        C_DIM,
        "\"почему\" - расклад, \"заново\" - новый раунд, \"стоп\" или Esc - выход.",
        "\"why\" - show matrix, \"restart\" - new round, \"stop\" or Esc - exit.",
        "\"why\" - المصفوفة, \"restart\" - جولة جديدة, \"stop\" او Esc - خروج."
    );
}

fn print_model_line() {
    match voodoo_nn::net() {
        Ok(net) => say!(
            C_DIM,
            "[NN] {}: GGUF, {}x{} int8, {} КБ",
            "[NN] {}: GGUF, {}x{} int8, {} KB",
            "[NN] {}: GGUF, {}x{} int8, {} KB",
            net.name,
            net.buckets,
            net.hidden,
            net.file_len / 1024
        ),
        Err(e) => say!(
            C_ERR,
            "[NN] веса не загружены ({}) - оракул слушает ключевые слова.",
            "[NN] weights not loaded ({}) - the oracle falls back to keywords.",
            "[NN] الاوزان غير محملة ({}) - كلمات مفتاحية فقط.",
            e.describe()
        ),
    }
}

fn print_bye() {
    say!(
        C_TITLE,
        "[VOODOO] Связь с оракулом закрыта.",
        "[VOODOO] Oracle link closed.",
        "[VOODOO] تم اغلاق الاتصال بالعراف."
    );
}

fn ask(q: usize) {
    say!(
        C_ASK,
        "[?] {}",
        "[?] {}",
        "[?] {}",
        engine::question_text(q)
    );
}

fn play_again() {
    say!(
        C_TEXT,
        "[VOODOO] Ещё раунд? (да / стоп)",
        "[VOODOO] Another round? (yes / stop)",
        "[VOODOO] جولة اخرى? (نعم / توقف)"
    );
}

fn fresh_start() {
    say!(
        C_DIM,
        "[VOODOO] Начинаю с чистого листа.",
        "[VOODOO] Starting from a clean slate.",
        "[VOODOO] ابدا من جديد."
    );
}

fn not_understood() {
    say!(
        C_DIM,
        "[VOODOO] Не разобрал. Скажи да, нет, хз - или подскажи словами.",
        "[VOODOO] Didn't get that. Say yes, no, maybe - or give a hint.",
        "[VOODOO] لم افهم. قل نعم او لا او ربما - او اعط تلميحا."
    );
}

/// Что увидела нейросеть — серой строкой под вводом.
fn print_reading(r: &Reading) {
    let (top, p) = r.top_intent();
    let source = match (r.neural, locale::get_locale()) {
        (true, Locale::RuRu) => "нейро",
        (false, Locale::RuRu) => "правила",
        (true, _) => "nn",
        (false, _) => "rules",
    };

    if top == Intent::Hint {
        let (c, pc) = r.top_persona();
        locale::print_localized_fmt(
            C_DIM,
            format_args!(
                "  {}[{}]: {} {}% -> {} {}%",
                source,
                r.features,
                intent_label(top),
                pct(p),
                persona_short(c),
                pct(pc)
            ),
        );
        return;
    }

    // Топ-3 интента; для ответов — ещё и «шкала да/нет», которую получит движок
    let mut order = [0usize; INTENT_COUNT];
    for (i, slot) in order.iter_mut().enumerate() {
        *slot = i;
    }
    order.sort_unstable_by(|&a, &b| {
        r.intent[b]
            .partial_cmp(&r.intent[a])
            .unwrap_or(core::cmp::Ordering::Equal)
    });
    let [a, b, c, ..] = order;
    let scale = AnswerScale(
        matches!(top, Intent::Yes | Intent::No | Intent::Maybe).then(|| r.answer_value()),
    );
    locale::print_localized_fmt(
        C_DIM,
        format_args!(
            "  {}[{}]: {} {}% | {} {}% | {} {}%{}",
            source,
            r.features,
            intent_label(Intent::ALL[a]),
            pct(r.intent[a]),
            intent_label(Intent::ALL[b]),
            pct(r.intent[b]),
            intent_label(Intent::ALL[c]),
            pct(r.intent[c]),
            scale
        ),
    );
}

/// Хвост «  -> 0.96» — шкала да/нет, которую получит движок (только для ответов).
struct AnswerScale(Option<f32>);

impl core::fmt::Display for AnswerScale {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            Some(v) => write!(f, "  -> {:.2}", v),
            None => Ok(()),
        }
    }
}

/// Имена интентов для хоста (не локализуются).
const INTENT_KEYS: [&str; INTENT_COUNT] = [
    "yes", "no", "maybe", "hint", "why", "restart", "stop", "greet",
];

/// «Dr. Bred 86%, Петрович 5%, ... | crossed out: X;» — топ кандидатов для LLM.
struct Board<'a>(&'a VoodooEngine);

impl core::fmt::Display for Board<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let e = self.0;
        let mut order = [0usize; CHAR_COUNT];
        for (i, slot) in order.iter_mut().enumerate() {
            *slot = i;
        }
        order.sort_unstable_by(|&a, &b| {
            e.probabilities[b]
                .partial_cmp(&e.probabilities[a])
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        let mut first = true;
        for &c in order.iter().filter(|&&c| !e.excluded[c]).take(3) {
            if !first {
                f.write_str(", ")?;
            }
            first = false;
            write!(
                f,
                "{} {}%",
                engine::character_name(c),
                pct(e.probabilities[c])
            )?;
        }
        let mut crossed = order.iter().filter(|&&c| e.excluded[c]).peekable();
        if crossed.peek().is_some() {
            f.write_str(" | crossed out:")?;
            for &c in crossed {
                write!(f, " {};", engine::character_name(c))?;
            }
        }
        Ok(())
    }
}

fn argmax(v: &[f32; CHAR_COUNT]) -> usize {
    let mut best = 0;
    for (i, &x) in v.iter().enumerate() {
        if x > v[best] {
            best = i;
        }
    }
    best
}

fn intent_label(i: Intent) -> &'static str {
    const RU: [&str; INTENT_COUNT] = [
        "да",
        "нет",
        "хз",
        "подсказка",
        "почему",
        "заново",
        "стоп",
        "привет",
    ];
    const EN: [&str; INTENT_COUNT] = [
        "yes", "no", "maybe", "hint", "why", "restart", "stop", "hello",
    ];
    const AR: [&str; INTENT_COUNT] = [
        "نعم",
        "لا",
        "ربما",
        "تلميح",
        "لماذا",
        "من جديد",
        "توقف",
        "مرحبا",
    ];
    match locale::get_locale() {
        Locale::RuRu => RU[i as usize],
        Locale::EnUs => EN[i as usize],
        Locale::ArEg => AR[i as usize],
    }
}

fn persona_short(c: usize) -> &'static str {
    const RU: [&str; CHAR_COUNT] = [
        "Петрович",
        "К.А.Ф.И.Д.Р.А.",
        "Турист",
        "Dr. Bred",
        "Архитектор",
    ];
    const EN: [&str; CHAR_COUNT] = [
        "Petrovich",
        "K.A.F.I.D.R.A.",
        "Tourist",
        "Dr. Bred",
        "Architect",
    ];
    match locale::get_locale() {
        Locale::RuRu => RU[c],
        Locale::EnUs | Locale::ArEg => EN[c],
    }
}

fn pct(p: f32) -> u32 {
    let v = p * 100.0 + 0.5;
    if v <= 0.0 {
        0
    } else if v >= 100.0 {
        100
    } else {
        v as u32
    }
}

fn bar(p: f32, buf: &mut [u8; BAR_WIDTH]) -> &str {
    let filled = ((p * BAR_WIDTH as f32 + 0.5) as usize).min(BAR_WIDTH);
    for (i, b) in buf.iter_mut().enumerate() {
        *b = if i < filled { b'#' } else { b'.' };
    }
    core::str::from_utf8(buf).unwrap_or("")
}

// ============================================================
// Шелл-подкоманды: voodoo nn / model / test
// ============================================================

/// `voodoo nn <текст>` — что нейросеть видит в произвольной фразе.
pub fn probe(input: &[u32]) {
    let net = match voodoo_nn::net() {
        Ok(net) => net,
        Err(_) => {
            print_model_line();
            return;
        }
    };
    let Some(r) = net.read(input) else {
        say!(
            C_DIM,
            "[NN] usage: voodoo nn <фраза>",
            "[NN] usage: voodoo nn <phrase>",
            "[NN] usage: voodoo nn <text>"
        );
        return;
    };
    say!(
        C_TITLE,
        "[NN] признаков {}, шкала да/нет {:.2}",
        "[NN] features {}, yes/no scale {:.2}",
        "[NN] الميزات {}, مقياس نعم/لا {:.2}",
        r.features,
        r.answer_value()
    );
    let mut buf = [0u8; BAR_WIDTH];
    for (i, &p) in r.intent.iter().enumerate() {
        locale::print_localized_fmt(
            C_TEXT,
            format_args!(
                "  {:<10} {} {:>3}%",
                intent_label(Intent::ALL[i]),
                bar(p, &mut buf),
                pct(p)
            ),
        );
    }
    say!(
        C_TITLE,
        "[NN] если это подсказка - про кого:",
        "[NN] if it's a hint - who is it about:",
        "[NN] اذا كان تلميحا - عن من:"
    );
    for (c, &p) in r.persona.iter().enumerate() {
        locale::print_localized_fmt(
            C_TEXT,
            format_args!(
                "  {:<14} {} {:>3}%",
                persona_short(c),
                bar(p, &mut buf),
                pct(p)
            ),
        );
    }
}

/// `voodoo model` — паспорт весов.
pub fn print_model_info() {
    match voodoo_nn::net() {
        Ok(net) => {
            locale::print_localized_fmt(C_TITLE, format_args!("[NN] {}", net.name));
            locale::print_localized_fmt(
                C_TEXT,
                format_args!(
                    "  GGUF v3: {} tensors, {} kv, {} bytes, fnv1a {:08X}",
                    net.tensor_count, net.kv_count, net.file_len, net.checksum
                ),
            );
            locale::print_localized_fmt(
                C_TEXT,
                format_args!(
                    "  embed {}x{} i8 -> relu -> intent {} | persona {}",
                    net.buckets, net.hidden, INTENT_COUNT, CHAR_COUNT
                ),
            );
        }
        Err(_) => print_model_line(),
    }
}

/// `voodoo test` — golden-векторы (бит-в-бит с Python) + сценарии движка.
pub fn selftest() -> bool {
    let net = match voodoo_nn::net() {
        Ok(net) => net,
        Err(e) => {
            print_model_line();
            crate::serial_println!("[VOODOO][TEST] FAIL model: {}", e.describe());
            return false;
        }
    };

    let mut passed = 0usize;
    for g in GOLDEN {
        let mut buf = [0u32; voodoo_nn::MAX_INPUT_CPS];
        let len = str_to_cps(g.text, &mut buf);
        let ok = matches!(
            net.infer_raw(&buf[..len]),
            Some(raw) if raw.features == g.features
                && raw.intent == g.intent_acc
                && raw.persona == g.persona_acc
        );
        if ok {
            passed += 1;
        } else {
            locale::print_localized_fmt(C_ERR, format_args!("  FAIL golden: \"{}\"", g.text));
            crate::serial_println!("[VOODOO][TEST] FAIL golden {:?}", g.text);
        }
    }

    let mut checks = 0usize;
    let mut checks_ok = 0usize;
    let mut check = |name: &str, ok: bool| {
        checks += 1;
        if ok {
            checks_ok += 1;
        } else {
            locale::print_localized_fmt(C_ERR, format_args!("  FAIL {}", name));
            crate::serial_println!("[VOODOO][TEST] FAIL {}", name);
        }
    };

    let e1 = voodoo_nn::exp_approx(1.0);
    check("exp(1)", e1 > 2.718 && e1 < 2.719);
    check("exp(-60)", voodoo_nn::exp_approx(-60.0) > 0.0);

    // «да» на вопрос про рецепты → лидер Dr. Bred, пора угадывать
    let mut eng = VoodooEngine::new();
    let yes = reading_of(net, "да");
    check(
        "nn: да = yes",
        matches!(yes, Some(r) if r.top_intent().0 == Intent::Yes),
    );
    let a = yes.map(|r| r.answer_value()).unwrap_or(0.0);
    eng.answer_question(2, a);
    check(
        "engine: рецепты -> Dr. Bred",
        eng.leader().0 == 3 && eng.should_guess(),
    );

    // Подсказка словами → правдоподобие от persona-головы
    eng.reset();
    if let Some(r) = reading_of(net, "король жижи") {
        eng.absorb_neural(&r.persona, r.intent[Intent::Hint as usize]);
    }
    check("engine: король жижи -> Петрович", eng.leader().0 == 0);

    // Неверная догадка вычёркивается и больше не всплывает
    eng.exclude(0);
    check(
        "engine: exclude",
        eng.leader().0 != 0 && eng.probabilities[0] == 0.0,
    );

    let total = GOLDEN.len();
    let ok = passed == total && checks_ok == checks;
    locale::print_localized_fmt(
        if ok { C_OK } else { C_ERR },
        format_args!(
            "[VOODOO TEST] golden {}/{}, engine {}/{} -> {}",
            passed,
            total,
            checks_ok,
            checks,
            if ok { "PASS" } else { "FAIL" }
        ),
    );
    crate::serial_println!(
        "[VOODOO][TEST] golden {}/{} engine {}/{} {}",
        passed,
        total,
        checks_ok,
        checks,
        if ok { "PASS" } else { "FAIL" }
    );
    ok
}

fn reading_of(net: &voodoo_nn::VoodooNet, text: &str) -> Option<Reading> {
    let mut buf = [0u32; voodoo_nn::MAX_INPUT_CPS];
    let len = str_to_cps(text, &mut buf);
    net.read(&buf[..len])
}

fn str_to_cps(text: &str, out: &mut [u32; voodoo_nn::MAX_INPUT_CPS]) -> usize {
    let mut len = 0;
    for ch in text.chars() {
        if len == out.len() {
            break;
        }
        out[len] = ch as u32;
        len += 1;
    }
    len
}
