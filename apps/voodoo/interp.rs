// ============================================================
// VOODOO INTERP — движок как интерпретатор выхода нейросети
// ============================================================
// Задача: ответы LLM непредсказуемее, но с меньшим количеством бреда.
// Температура сама по себе даёт и то и другое сразу, поэтому иначе:
//
//   хост: одна реплика в SLOTS версиях — слот k = своя температура,
//         от холодной (точно, скучно) к горячей (живо, рискованно),
//         у каждой три оценки:
//           смысл    — LLM-судья: «реплика по делу, без бреда?»
//           гладкость — средняя вероятность токенов (только телеметрия,
//                       см. FLUENCY_WEIGHT)
//           новизна  — непохожесть на уже сказанное и на шаблоны промпта
//   ядро: бред (смысл < SENSE_MIN) и повтор (новизна < NOVELTY_MIN)
//                                            → маска, как вычеркнутый персонаж
//         настроение × свидетельство (Байес) → торможение → клеточный автомат
//         выбор — жребий по апостериори
//
// «Настроение» — апостериори прошлого хода: живёт между репликами, поэтому
// дух сам дрейфует между холодным и горячим. Автомат не даёт залипнуть,
// Байес не даёт уйти в бред. Застой (одна температура STALL раз подряд) —
// автомат меняет правило.
// ============================================================

use crate::kernel_messages::Locale;
use crate::locale;
use crate::voodoo_engine::{AutomatonMode, CHAR_COUNT, VoodooEngine};
use spin::Mutex;

pub const SLOTS: usize = CHAR_COUNT;

/// Ниже — бред, слот вычёркивается на этот ход.
const SENSE_MIN: f32 = 0.35;
/// Ниже — повтор сказанного или шаблона промпта: мёртвая реплика — тоже отсев.
const NOVELTY_MIN: f32 = 0.2;
/// Доля прошлого настроения в новом априори (остальное — равномерное).
const MOOD_MEMORY: f32 = 0.5;
/// Одна и та же температура столько раз подряд — смена правила автомата.
const STALL: u32 = 3;
/// Вес гладкости в свидетельстве. Был 0.5, но замер voodoo-proxy (bench.py,
/// 16 запросов × 3 прогона, YandexGPT-5-Lite-8B) показал: гладкость — это
/// просто «температура наоборот» (у холодных версий она всегда выше), а
/// торможение p² удваивает фору. Интерпретатор уплывал в t=0.35 (54% выборов)
/// и был менее разнообразным, чем обычная генерация. С нулём выбор ровный
/// по всем температурам, разнообразие выше, качество по судье то же.
const FLUENCY_WEIGHT: f32 = 0.0;

const MODES: [AutomatonMode; 6] = [
    AutomatonMode::ConwayB3S23,
    AutomatonMode::Seeds,
    AutomatonMode::Diffusion,
    AutomatonMode::HighLife,
    AutomatonMode::QuantumChaos,
    AutomatonMode::Rule90,
];

#[derive(Clone, Copy)]
pub struct Candidate {
    pub temp: f32,
    pub sense: f32,
    pub fluency: f32,
    pub novelty: f32,
}

pub struct Verdict {
    pub pick: usize,
    /// Правило автомата, по которому сделан этот выбор.
    pub mode: AutomatonMode,
    pub posterior: [f32; SLOTS],
    pub rejected: usize,
    pub new_mode: Option<AutomatonMode>,
}

struct Interp {
    engine: VoodooEngine,
    mode: usize,
    last_pick: usize,
    streak: u32,
    picks: u32,
    rejected_total: u32,
    hist: [u32; SLOTS],
    temps: [f32; SLOTS],
}

static INTERP: Mutex<Option<Interp>> = Mutex::new(None);

fn with_interp<R>(f: impl FnOnce(&mut Interp) -> R) -> R {
    let mut guard = INTERP.lock();
    let it = guard.get_or_insert_with(|| Interp {
        engine: VoodooEngine::new(),
        mode: 0,
        last_pick: usize::MAX,
        streak: 0,
        picks: 0,
        rejected_total: 0,
        hist: [0; SLOTS],
        temps: [0.0; SLOTS],
    });
    f(it)
}

/// Выбор версии реплики. None — ни одной версии не пришло.
pub fn decide(cands: &[Option<Candidate>; SLOTS]) -> Option<Verdict> {
    if cands.iter().all(|c| c.is_none()) {
        return None;
    }

    // Свидетельство: смысл — главный множитель, новизна (и гладкость с весом
    // FLUENCY_WEIGHT) — мягкие (линейные смеси вместо степеней: в no_std нет powf).
    let mut evidence = [0.0f32; SLOTS];
    let mut allowed = [false; SLOTS];
    let mut best_sense = (0usize, -1.0f32);
    for (k, c) in cands.iter().enumerate() {
        let Some(c) = c else {
            continue;
        };
        let fluency = (1.0 - FLUENCY_WEIGHT) + FLUENCY_WEIGHT * c.fluency;
        evidence[k] = c.sense * fluency * (0.25 + 0.75 * c.novelty);
        allowed[k] = c.sense >= SENSE_MIN && c.novelty >= NOVELTY_MIN;
        if c.sense > best_sense.1 {
            best_sense = (k, c.sense);
        }
    }
    let rejected = cands
        .iter()
        .zip(allowed.iter())
        .filter(|(c, a)| c.is_some() && !**a)
        .count();
    if allowed.iter().all(|&a| !a) {
        // Всё в отсеве — берём наименее бредовое, лишь бы дух не молчал
        allowed[best_sense.0] = true;
    }

    Some(with_interp(|it| {
        it.engine.cellular_mode = MODES[it.mode];
        // Априори = смягчённое настроение прошлого хода
        let uniform = 1.0 / SLOTS as f32;
        for p in it.engine.probabilities.iter_mut() {
            *p = MOOD_MEMORY * *p + (1.0 - MOOD_MEMORY) * uniform;
        }
        it.engine.absorb_evidence(&evidence, &allowed);
        let posterior = it.engine.probabilities;
        let pick = sample(&posterior, &allowed);
        let mode = MODES[it.mode];

        it.picks += 1;
        it.hist[pick] += 1;
        it.rejected_total += rejected as u32;
        for (k, c) in cands.iter().enumerate() {
            if let Some(c) = c {
                it.temps[k] = c.temp;
            }
        }
        if pick == it.last_pick {
            it.streak += 1;
        } else {
            it.last_pick = pick;
            it.streak = 1;
        }
        let mut new_mode = None;
        if it.streak >= STALL {
            it.mode = (it.mode + 1) % MODES.len();
            it.streak = 0;
            new_mode = Some(MODES[it.mode]);
        }
        Verdict {
            pick,
            mode,
            posterior,
            rejected,
            new_mode,
        }
    }))
}

/// Жребий по распределению: RDRAND, без него — TSC.
fn sample(p: &[f32; SLOTS], allowed: &[bool; SLOTS]) -> usize {
    let r = match crate::apps::rng::random_u64() {
        Some(v) => v,
        None => unsafe { core::arch::x86_64::_rdtsc() },
    };
    let mut x = (r % 1_000_000) as f32 / 1_000_000.0;
    let mut last = 0;
    for k in 0..SLOTS {
        if !allowed[k] {
            continue;
        }
        last = k;
        if x < p[k] {
            return k;
        }
        x -= p[k];
    }
    last
}

/// Строка телеметрии под репликой духа.
pub fn print_verdict(cands: &[Option<Candidate>; SLOTS], v: &Verdict) {
    let mode = v.mode;
    let temp = cands[v.pick].map(|c| c.temp).unwrap_or(0.0);
    let mut sense = [0u32; SLOTS];
    for (k, c) in cands.iter().enumerate() {
        sense[k] = c.map(|c| pct(c.sense)).unwrap_or(0);
    }
    let chaos = pct(1.0 - v.posterior.iter().cloned().fold(0.0f32, f32::max));
    let [a, b, c, d, e] = sense;
    match locale::get_locale() {
        Locale::RuRu => locale::print_localized_fmt(
            0x08,
            format_args!(
                "  интерп[{}]: смысл {} {} {} {} {} | отсев {} | t={:.2} p={}% | хаос {}%",
                mode.name(),
                a,
                b,
                c,
                d,
                e,
                v.rejected,
                temp,
                pct(v.posterior[v.pick]),
                chaos
            ),
        ),
        _ => locale::print_localized_fmt(
            0x08,
            format_args!(
                "  interp[{}]: sense {} {} {} {} {} | junk {} | t={:.2} p={}% | chaos {}%",
                mode.name(),
                a,
                b,
                c,
                d,
                e,
                v.rejected,
                temp,
                pct(v.posterior[v.pick]),
                chaos
            ),
        ),
    }
    if let Some(m) = v.new_mode {
        match locale::get_locale() {
            Locale::RuRu => locale::print_localized_fmt(
                0x08,
                format_args!(
                    "  интерп: застой на t={:.2} - автомат меняет правило: {}",
                    temp,
                    m.name()
                ),
            ),
            _ => locale::print_localized_fmt(
                0x08,
                format_args!(
                    "  interp: stuck at t={:.2} - automaton switches rule: {}",
                    temp,
                    m.name()
                ),
            ),
        }
    }
}

/// `voodoo interp` — настроение интерпретатора по температурам.
pub fn print_state() {
    with_interp(|it| {
        match locale::get_locale() {
            Locale::RuRu => locale::print_localized_fmt(
                0x0B,
                format_args!(
                    "[ИНТЕРП] настроение духа | автомат {} | выборов {}, отсеяно версий {}",
                    MODES[it.mode].name(),
                    it.picks,
                    it.rejected_total
                ),
            ),
            _ => locale::print_localized_fmt(
                0x0B,
                format_args!(
                    "[INTERP] spirit's mood | automaton {} | picks {}, versions rejected {}",
                    MODES[it.mode].name(),
                    it.picks,
                    it.rejected_total
                ),
            ),
        }
        for k in 0..SLOTS {
            let p = it.engine.probabilities[k];
            let filled = ((p * 20.0 + 0.5) as usize).min(20);
            let mut bar = [b'.'; 20];
            for cell in bar.iter_mut().take(filled) {
                *cell = b'#';
            }
            locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "  t={:.2}  {} {:>3}%  x{}",
                    it.temps[k],
                    core::str::from_utf8(&bar).unwrap_or(""),
                    pct(p),
                    it.hist[k]
                ),
            );
        }
    });
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
