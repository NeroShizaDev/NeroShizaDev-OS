// ============================================================
// VOODOO ENGINE — байесовская прослойка между нейросетью и диалогом
// ============================================================
// Нейро-слой (voodoo_nn) читает текст и отдаёт распределения:
// «насколько это да», «про кого подсказка». Сам он ничего не решает.
// Движок — единственное место, где копится уверенность:
//
//   ответ на вопрос  → answer_question(q, a∈[0,1])   (мягкий Байес)
//   нейро-подсказка  → absorb_neural(L[persona], w)   (правдоподобие)
//   неверная догадка → exclude(c)
//        каждое свидетельство → латеральное торможение
//                             → клеточный автомат (подмешивается как хаос)
//                             → проверка энтропийного коллапса
//
// Диалог поверх движка — apps/voodoo/chat.rs.
// ============================================================

#![allow(dead_code)]

use crate::kernel_messages::Locale;
use core::arch::asm;

pub const CHAR_COUNT: usize = 5;
pub const Q_COUNT: usize = 7;

pub const CHARACTERS: [&str; CHAR_COUNT] = [
    "Петрович (Король жижи)",
    "Группа К.А.Ф.И.Д.Р.А.",
    "Банановый Турист",
    "Dr. Bred",
    "Архитектор Хаоса",
];

const CHARACTERS_EN: [&str; CHAR_COUNT] = [
    "Petrovich (King of Goo)",
    "K.A.F.I.D.R.A. Group",
    "Banana Tourist",
    "Dr. Bred",
    "Architect of Chaos",
];

/// P(вопрос_j отвечает «да» | персонаж_i)
const KNOWLEDGE_BASE: [[f32; Q_COUNT]; CHAR_COUNT] = [
    [0.1, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
    [1.0, 0.5, 0.0, 0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    [0.8, 0.2, 1.0, 0.0, 0.0, 0.0, 0.0],
    [0.9, 0.8, 0.0, 1.0, 0.0, 0.0, 0.0],
];

const QUESTIONS_RU: [&str; Q_COUNT] = [
    "Он учёный или исследователь?",
    "Он у власти - правит, командует?",
    "Он выписывает рецепты?",
    "Он строит миры из кода?",
    "Он путешествует в поисках истины?",
    "Это целая группа, а не один персонаж?",
    "Он связан с жижей?",
];

const QUESTIONS_EN: [&str; Q_COUNT] = [
    "Is it a scientist or a researcher?",
    "Does it hold power - rule, command?",
    "Does it write prescriptions?",
    "Does it build worlds out of code?",
    "Does it travel in search of truth?",
    "Is it a whole group, not one character?",
    "Is it connected to goo?",
];

const QUESTIONS_AR: [&str; Q_COUNT] = [
    "هل هو عالم او باحث?",
    "هل يملك السلطة ويحكم?",
    "هل يكتب الوصفات الطبية?",
    "هل يبني عوالم من الكود?",
    "هل يسافر بحثا عن الحقيقة?",
    "هل هي مجموعة كاملة وليست شخصية واحدة?",
    "هل له علاقة بالسائل اللزج?",
];

/// Вероятность, что человек ответил наоборот (опечатка, не понял вопрос).
/// Без неё один неверный ответ навсегда обнуляет правильного персонажа.
const ANSWER_NOISE: f32 = 0.05;

/// Насколько движок верит persona-голове нейросети. Остаток — равномерное
/// распределение: сеть свидетель, а не судья. Без этого одна фраза при
/// латеральном торможении (p²) сразу даёт «уверенность 100%».
const NEURAL_TRUST: f32 = 0.5;

/// С какой уверенностью лидера движок готов назвать имя.
pub const GUESS_THRESHOLD: f32 = 0.80;

pub fn character_name(idx: usize) -> &'static str {
    match crate::locale::get_locale() {
        Locale::RuRu => CHARACTERS[idx],
        Locale::EnUs | Locale::ArEg => CHARACTERS_EN[idx],
    }
}

pub fn question_text(q: usize) -> &'static str {
    match crate::locale::get_locale() {
        Locale::RuRu => QUESTIONS_RU[q],
        Locale::EnUs => QUESTIONS_EN[q],
        Locale::ArEg => QUESTIONS_AR[q],
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum AutomatonMode {
    ConwayB3S23,
    HighLife,
    Seeds,
    Rule90,
    Diffusion,
    QuantumChaos,
}

impl AutomatonMode {
    #[inline]
    fn is_deterministic(self) -> bool {
        matches!(self, AutomatonMode::Rule90)
    }

    /// Доля автомата, подмешиваемая к байесовскому апостериори.
    /// 1.0 = автомат полностью переписывает вероятности (как в демо).
    #[inline]
    fn chaos_blend(self) -> f32 {
        match self {
            AutomatonMode::ConwayB3S23 | AutomatonMode::HighLife => 0.15,
            AutomatonMode::Seeds => 0.20,
            AutomatonMode::Rule90 => 0.10,
            AutomatonMode::Diffusion => 0.25,
            AutomatonMode::QuantumChaos => 0.35,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            AutomatonMode::ConwayB3S23 => "Conway",
            AutomatonMode::HighLife => "HighLife",
            AutomatonMode::Seeds => "Seeds",
            AutomatonMode::Rule90 => "Rule90",
            AutomatonMode::Diffusion => "Diffusion",
            AutomatonMode::QuantumChaos => "QuantumChaos",
        }
    }
}

pub struct VoodooEngine {
    pub probabilities: [f32; CHAR_COUNT],
    pub asked_questions: [bool; Q_COUNT],
    /// Вычеркнутые после неверной догадки — до конца раунда
    pub excluded: [bool; CHAR_COUNT],
    pub connection_drops: u32,
    /// Сколько нейро-подсказок впитано в этом раунде
    pub neural_evidence: u32,
    question_loop_counter: u32,
    answer_count: u32,
    pub cellular_mode: AutomatonMode,
}

impl VoodooEngine {
    pub fn new() -> Self {
        let init_prob = 1.0 / (CHAR_COUNT as f32);
        Self {
            probabilities: [init_prob; CHAR_COUNT],
            asked_questions: [false; Q_COUNT],
            excluded: [false; CHAR_COUNT],
            connection_drops: 0,
            neural_evidence: 0,
            question_loop_counter: 0,
            answer_count: 0,
            cellular_mode: AutomatonMode::ConwayB3S23,
        }
    }

    /// Новый раунд: равномерное распределение, все вопросы снова доступны.
    pub fn reset(&mut self) {
        self.reset_matrix();
    }

    pub fn questions_asked(&self) -> usize {
        self.asked_questions.iter().filter(|&&q| q).count()
    }

    pub fn candidates_left(&self) -> usize {
        self.excluded.iter().filter(|&&x| !x).count()
    }

    /// Самый вероятный из невычеркнутых.
    pub fn leader(&self) -> (usize, f32) {
        let mut best_idx = 0;
        let mut best_val = -1.0f32;
        for (i, &p) in self.probabilities.iter().enumerate() {
            if !self.excluded[i] && p > best_val {
                best_val = p;
                best_idx = i;
            }
        }
        (best_idx, best_val.max(0.0))
    }

    /// Пора называть имя: лидер уверен, вопросы кончились или выбора нет.
    pub fn should_guess(&self) -> bool {
        let (_, p) = self.leader();
        p >= GUESS_THRESHOLD
            || self.candidates_left() <= 1
            || self.asked_questions.iter().all(|&q| q)
    }

    /// Нейро-подсказка как функция правдоподобия.
    ///
    /// `likelihood` — softmax persona-головы (сумма 1), `weight` ∈ [0,1] —
    /// насколько сеть уверена, что это вообще подсказка. L смягчается
    /// к равномерному (NEURAL_TRUST), затем смесь (1-w) + w·N·L не даёт
    /// одной фразе обнулить персонажа: при w=0 свидетельство нейтрально.
    pub fn absorb_neural(&mut self, likelihood: &[f32; CHAR_COUNT], weight: f32) -> bool {
        let w = clamp_f32(weight, 0.0, 1.0);
        let n = CHAR_COUNT as f32;
        let mut sum = 0.0f32;
        for c in 0..CHAR_COUNT {
            if self.excluded[c] {
                self.probabilities[c] = 0.0;
                continue;
            }
            let l = NEURAL_TRUST * clamp_f32(likelihood[c], 0.0, 1.0) + (1.0 - NEURAL_TRUST) / n;
            self.probabilities[c] *= (1.0 - w) + w * n * l;
            sum += self.probabilities[c];
        }
        if sum <= 1e-6 {
            self.reset_matrix();
            return false;
        }
        for p in &mut self.probabilities {
            *p /= sum;
        }
        self.neural_evidence += 1;
        self.integrate();
        !self.detect_entropy_collapse()
    }

    /// Свидетельство над слотами без игровой семантики — так движок работает
    /// интерпретатором выхода LLM (apps/voodoo/interp.rs): слоты = версии
    /// реплики. `allowed[k] == false` — слот вне игры на этот шаг (бред).
    /// Дальше тот же хвост, что у вопросов и подсказок: торможение → автомат.
    pub fn absorb_evidence(&mut self, evidence: &[f32; CHAR_COUNT], allowed: &[bool; CHAR_COUNT]) {
        for c in 0..CHAR_COUNT {
            self.excluded[c] = !allowed[c];
            self.probabilities[c] *= clamp_f32(evidence[c], 0.0, 1.0);
        }
        self.normalize_probabilities();
        self.integrate();
    }

    /// Неверная догадка: персонаж вычёркивается до конца раунда.
    /// false — вычеркнуть больше некого.
    pub fn exclude(&mut self, idx: usize) -> bool {
        if idx >= CHAR_COUNT {
            return self.candidates_left() > 0;
        }
        self.excluded[idx] = true;
        self.normalize_probabilities();
        self.candidates_left() > 0
    }

    pub fn set_cellular_mode(&mut self, mode: AutomatonMode) {
        self.cellular_mode = mode;
        let mode_str = match mode {
            AutomatonMode::ConwayB3S23 => "Conway 1D-адаптация B2/S12",
            AutomatonMode::HighLife => "HighLife 1D (аналог Conway; 2D вне этой сборки)",
            AutomatonMode::Seeds => "Seeds B2/S (хаос)",
            AutomatonMode::Rule90 => "Rule90 (треугольник Серпинского, детерминирован)",
            AutomatonMode::Diffusion => "Diffusion (сглаживание)",
            AutomatonMode::QuantumChaos => "QuantumChaos (аппаратный шум + смешение)",
        };
        crate::kernel_messages::print_voodoo_mode(mode_str);
    }

    pub fn get_best_question(&mut self) -> Option<usize> {
        if self.asked_questions.iter().all(|&q| q) {
            return None;
        }

        self.question_loop_counter += 1;
        if self.question_loop_counter > 15 {
            self.connection_drops += 1;
            crate::locale::print_localized_line(
                crate::kernel_messages::current(crate::kernel_messages::UiText::VoodooLoopBreak),
                0x0C,
            );
            self.reset_matrix();
            return None;
        }

        let mut best_q: Option<usize> = None;
        let mut best_score = 1.0f32;

        for q in 0..Q_COUNT {
            if self.asked_questions[q] {
                continue;
            }
            let mut expected = 0.0f32;
            for c in 0..CHAR_COUNT {
                expected += self.probabilities[c] * KNOWLEDGE_BASE[c][q];
            }
            expected += unsafe { hw_noise() } - 0.05;
            let score = unsafe { fpu_abs(expected - 0.5) };
            if score < best_score {
                best_score = score;
                best_q = Some(q);
            }
        }
        best_q
    }

    pub fn answer_question(&mut self, q_index: usize, user_answer: f32) -> bool {
        self.answer_count += 1;
        if self.answer_count > (Q_COUNT as u32) + 2 {
            self.connection_drops += 1;
            crate::locale::print_localized_line(
                crate::kernel_messages::current(
                    crate::kernel_messages::UiText::VoodooTooManyAnswers,
                ),
                0x0C,
            );
            self.reset_matrix();
            return false;
        }

        if q_index >= Q_COUNT {
            return false;
        }
        let user_answer = clamp_f32(user_answer, 0.0, 1.0);

        self.asked_questions[q_index] = true;

        let mut sum = 0.0f32;
        for c in 0..CHAR_COUNT {
            if self.excluded[c] {
                self.probabilities[c] = 0.0;
                continue;
            }
            let db = KNOWLEDGE_BASE[c][q_index];
            let agreement = (1.0 - unsafe { fpu_abs(db - user_answer) }).max(0.0);
            // Модель шума: с вероятностью ANSWER_NOISE человек ошибся
            let factor = ANSWER_NOISE + (1.0 - 2.0 * ANSWER_NOISE) * agreement;
            self.probabilities[c] *= factor;
            sum += self.probabilities[c];
        }

        if sum > 1e-6 {
            for p in &mut self.probabilities {
                *p /= sum;
            }
        } else {
            self.reset_matrix();
            return false;
        }

        self.integrate();

        if self.detect_entropy_collapse() {
            return false;
        }

        true
    }

    pub fn guess(&self) -> (&'static str, f32) {
        let (idx, p) = self.leader();
        (CHARACTERS[idx], p)
    }

    pub fn chaos_measure(&self) -> f32 {
        let mut max_p = 0.0f32;
        for &p in &self.probabilities {
            if p > max_p {
                max_p = p;
            }
        }
        1.0 - max_p
    }

    /// Общий хвост любого свидетельства: торможение → автомат как
    /// примесь хаоса → маска вычеркнутых → нормализация.
    fn integrate(&mut self) {
        self.apply_lateral_inhibition();
        let bayes = self.probabilities;
        self.apply_cellular_tick();
        let a = self.cellular_mode.chaos_blend();
        for i in 0..CHAR_COUNT {
            self.probabilities[i] = (1.0 - a) * bayes[i] + a * self.probabilities[i];
        }
        self.normalize_probabilities();
    }

    fn apply_lateral_inhibition(&mut self) {
        let mut sum_sq = 0.0f32;
        for i in 0..CHAR_COUNT {
            let sq = unsafe { fpu_square(self.probabilities[i]) };
            self.probabilities[i] = sq;
            sum_sq += sq;
        }
        if sum_sq > 0.0 {
            for p in &mut self.probabilities {
                *p /= sum_sq;
            }
        }
    }

    fn apply_cellular_tick(&mut self) {
        let len = CHAR_COUNT;
        let mut new = self.probabilities;

        match self.cellular_mode {
            AutomatonMode::ConwayB3S23 | AutomatonMode::HighLife => {
                for i in 0..len {
                    let left = self.probabilities[(i + len - 1) % len];
                    let right = self.probabilities[(i + 1) % len];
                    let alive = self.probabilities[i] > 0.5;
                    let neighbors = usize::from(left > 0.5) + usize::from(right > 0.5);
                    new[i] = if alive && (neighbors == 1 || neighbors == 2) {
                        self.probabilities[i]
                    } else if !alive && neighbors == 2 {
                        0.8
                    } else if alive {
                        0.1
                    } else {
                        self.probabilities[i]
                    };
                }
            }
            AutomatonMode::Seeds => {
                for i in 0..len {
                    let left = self.probabilities[(i + len - 1) % len];
                    let right = self.probabilities[(i + 1) % len];
                    let alive = self.probabilities[i] > 0.5;
                    let neighbors = usize::from(left > 0.5) + usize::from(right > 0.5);
                    new[i] = if !alive && neighbors == 2 {
                        0.8
                    } else if alive {
                        0.1
                    } else {
                        self.probabilities[i]
                    };
                }
            }
            AutomatonMode::Rule90 => {
                for i in 0..len {
                    let left = self.probabilities[(i + len - 1) % len] > 0.5;
                    let right = self.probabilities[(i + 1) % len] > 0.5;
                    new[i] = if left ^ right { 1.0 } else { 0.0 };
                }
            }
            AutomatonMode::Diffusion => {
                for i in 0..len {
                    let left = self.probabilities[(i + len - 1) % len];
                    let right = self.probabilities[(i + 1) % len];
                    new[i] = (left + self.probabilities[i] + right) / 3.0;
                }
            }
            AutomatonMode::QuantumChaos => {
                for i in 0..len {
                    let left = self.probabilities[(i + len - 1) % len];
                    let right = self.probabilities[(i + 1) % len];
                    let mix = (left + self.probabilities[i] + right) / 3.0;
                    let chaos = unsafe { hw_noise() };
                    new[i] = (mix * (1.0 - chaos) + chaos).min(1.0);
                }
            }
        }

        if !self.cellular_mode.is_deterministic() {
            for cell in &mut new {
                *cell = (*cell + unsafe { hw_noise() } * 0.05).min(1.0);
            }
        }

        self.probabilities = new;
    }

    fn normalize_probabilities(&mut self) {
        const EPS: f32 = 1e-9;
        let mut sum = 0.0f32;
        for (i, p) in self.probabilities.iter_mut().enumerate() {
            if self.excluded[i] {
                *p = 0.0;
                continue;
            }
            if *p < EPS {
                *p = EPS;
            }
            sum += *p;
        }
        if sum > 1e-6 {
            for p in &mut self.probabilities {
                *p /= sum;
            }
        } else {
            let left = self.candidates_left().max(1);
            let init = 1.0 / (left as f32);
            for (i, p) in self.probabilities.iter_mut().enumerate() {
                *p = if self.excluded[i] { 0.0 } else { init };
            }
        }
    }

    fn detect_entropy_collapse(&mut self) -> bool {
        let mut max_p = 0.0f32;
        let mut sum = 0.0f32;
        for &p in &self.probabilities {
            if p > max_p {
                max_p = p;
            }
            sum += p;
        }
        let avg = sum / (CHAR_COUNT as f32);
        let asked = self.asked_questions.iter().filter(|&&q| q).count();

        if asked >= 3 && (max_p - avg) < 0.15 {
            crate::locale::print_localized_line(
                crate::kernel_messages::current(
                    crate::kernel_messages::UiText::VoodooEntropyCollapse,
                ),
                0x0C,
            );
            self.reset_matrix();
            return true;
        }
        false
    }

    fn reset_matrix(&mut self) {
        let init = 1.0 / (CHAR_COUNT as f32);
        for p in &mut self.probabilities {
            *p = init;
        }
        for q in &mut self.asked_questions {
            *q = false;
        }
        self.excluded = [false; CHAR_COUNT];
        self.neural_evidence = 0;
        self.question_loop_counter = 0;
        self.answer_count = 0;
    }
}

#[inline]
fn clamp_f32(val: f32, min: f32, max: f32) -> f32 {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

/// Вычисляет |val| через x87 FPU (FABS).
///
/// # Safety
/// x87 FPU доступен в long mode (CPUID: always present). options(nostack) —
/// asm не трогает RSP. fld/fstp балансируют x87-стек (push/pop ST0).
/// read_volatile на локальную переменную нужен чтобы компилятор не
/// оптимизировал fstp-запись как мёртвый код.
#[inline(always)]
unsafe fn fpu_abs(mut val: f32) -> f32 {
    asm!(
        "fld dword ptr [{0}]",
        "fabs",
        "fstp dword ptr [{0}]",
        in(reg) &mut val,
        options(nostack)
    );
    core::ptr::read_volatile(&val)
}

/// Вычисляет val² через x87 FPU (FMUL ST(0),ST(0)).
///
/// # Safety
/// Аналогично fpu_abs: fld/fstp балансируют x87-стек.
/// options(nostack) гарантирует что RSP не изменяется.
#[inline(always)]
unsafe fn fpu_square(mut val: f32) -> f32 {
    asm!(
        "fld dword ptr [{0}]",
        "fmul st(0), st(0)",
        "fstp dword ptr [{0}]",
        in(reg) &mut val,
        options(nostack)
    );
    core::ptr::read_volatile(&val)
}

/// Возвращает шумовое значение [0.0, 0.1) из RDRAND (через rng::random_u64)
/// или RDTSC-fallback.
///
/// # Safety
/// RDTSC fallback: непривилегированная инструкция (CR4.TSD=0), НЕ сериализующая —
/// для визуального шума погрешность незначительна.
/// options(nomem, nostack) — asm не касается памяти и RSP.
#[inline(always)]
unsafe fn hw_noise() -> f32 {
    let noise: u32 = match crate::apps::rng::random_u64() {
        Some(val) => val as u32,
        None => {
            // SAFETY: RDTSC — непривилегированная инструкция (CR4.TSD=0).
            // НЕ сериализующая; для визуального шума погрешность допустима.
            let low: u32;
            asm!("rdtsc", out("eax") low, out("edx") _, options(nomem, nostack));
            low
        }
    };
    (noise % 1000) as f32 / 10000.0
}

pub fn demo_cellular_automaton() {
    let mut engine = VoodooEngine::new();
    crate::locale::print_localized_line(
        crate::kernel_messages::current(crate::kernel_messages::UiText::VoodooDemoHeader),
        0x0E,
    );

    engine.probabilities = [0.0, 0.0, 1.0, 0.0, 0.0];
    engine.set_cellular_mode(AutomatonMode::Rule90);

    for step in 0..8 {
        engine.apply_cellular_tick();
        crate::kernel_messages::print_voodoo_step(
            step,
            engine.probabilities[0],
            engine.probabilities[1],
            engine.probabilities[2],
            engine.probabilities[3],
            engine.probabilities[4],
        );
    }
}
