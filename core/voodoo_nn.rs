// ============================================================
// VOODOO NN — нейро-слой перед байесовским движком
// ============================================================
// Задача: понять свободный текст человека и отдать VoodooEngine
// не «бит», а вероятности. Дальше движок сам решает, что с ними
// делать (Байес, латеральное торможение, клеточный автомат).
//
//   текст (codepoints) ─► featurize: хэши n-грамм → бакеты
//                      ─► EmbeddingBag(avg) + bias ─► ReLU      (int8 · i32)
//                      ─┬► intent  head: да/нет/хз/подсказка/…  (i64 акк.)
//                       └► persona head: про кого подсказка
//                      ─► softmax (f32, soft-float)
//
// Веса: apps/voodoo/weights/oracle.gguf (GGUF v3, int8 + i32).
// Обучение и контракт: tools/voodoo_nn/{train,vnn}.py — featurizer
// и целочисленная арифметика здесь ОБЯЗАНЫ совпадать с vnn.py.
// Проверка бит-в-бит: команда `voodoo test` (golden-векторы).
//
// Всё без аллокаций и без SSE: вызывается прямо из IRQ клавиатуры.
// ============================================================

#![allow(dead_code)]

use crate::gguf::{self, Gguf, Value};

pub const INTENT_COUNT: usize = 8;
pub const PERSONA_COUNT: usize = crate::voodoo_engine::CHAR_COUNT;
pub const MAX_HIDDEN: usize = 64;

const MODEL_VERSION: u32 = 1;
const FRAC_BITS: u32 = 8;
const FEATURIZER_ID: &str = "fnv1a32/cp/word+char2-3+wordbigram/v1";
const ARCHITECTURE: &str = "voodoo-bag";

pub const MAX_INPUT_CPS: usize = 64;
const MAX_WORDS: usize = 16;
const MAX_WORD_LEN: usize = 24;
pub const MAX_FEATURES: usize = 256;

const FNV_OFFSET: u32 = 0x811C_9DC5;
const FNV_PRIME: u32 = 0x0100_0193;
const TAG_CHAR: u32 = 1;
const TAG_WORD: u32 = 2;
const TAG_BIGRAM: u32 = 3;
const PAD_LEFT: u32 = 0x5E; // '^'
const PAD_RIGHT: u32 = 0x24; // '$'
const SEP: u32 = 0x20;

const INTENT_LABELS: [&str; INTENT_COUNT] = [
    "yes", "no", "maybe", "hint", "why", "restart", "stop", "greet",
];
const PERSONA_LABELS: [&str; PERSONA_COUNT] =
    ["petrovich", "kafidra", "tourist", "bred", "architect"];

static ORACLE_GGUF: &[u8] = include_bytes!("../apps/voodoo/weights/oracle.gguf");

// ============================================================
// Интенты
// ============================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Intent {
    Yes = 0,
    No = 1,
    Maybe = 2,
    Hint = 3,
    Why = 4,
    Restart = 5,
    Stop = 6,
    Greet = 7,
}

impl Intent {
    pub const ALL: [Intent; INTENT_COUNT] = [
        Intent::Yes,
        Intent::No,
        Intent::Maybe,
        Intent::Hint,
        Intent::Why,
        Intent::Restart,
        Intent::Stop,
        Intent::Greet,
    ];
}

// ============================================================
// Ошибки загрузки
// ============================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadError {
    Gguf(gguf::GgufError),
    WrongArchitecture,
    WrongVersion,
    WrongFeaturizer,
    MissingKey(&'static str),
    MissingTensor(&'static str),
    BadShape(&'static str),
    BadLabels,
    Checksum,
}

impl LoadError {
    pub fn describe(self) -> &'static str {
        match self {
            LoadError::Gguf(e) => e.describe(),
            LoadError::WrongArchitecture => "not a voodoo-bag model",
            LoadError::WrongVersion => "model version mismatch",
            LoadError::WrongFeaturizer => "featurizer mismatch",
            LoadError::MissingKey(k) => k,
            LoadError::MissingTensor(t) => t,
            LoadError::BadShape(t) => t,
            LoadError::BadLabels => "label set mismatch",
            LoadError::Checksum => "checksum mismatch",
        }
    }
}

// ============================================================
// Модель
// ============================================================
pub struct VoodooNet {
    pub name: &'static str,
    pub file_len: usize,
    pub tensor_count: u64,
    pub kv_count: u64,
    pub buckets: u32,
    pub hidden: usize,
    pub checksum: u32,
    embed: &'static [u8],
    embed_bias: &'static [u8],
    intent_w: &'static [u8],
    intent_b: &'static [u8],
    persona_w: &'static [u8],
    persona_b: &'static [u8],
    /// s_embed · s_head / 2^FRAC_BITS — переводит i64-аккумулятор в логит
    intent_scale: f32,
    persona_scale: f32,
}

/// Сырые целочисленные выходы — их сверяет `voodoo test`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RawLogits {
    pub features: u16,
    pub intent: [i64; INTENT_COUNT],
    pub persona: [i64; PERSONA_COUNT],
}

/// То, что нейро-слой отдаёт наверх: распределения, а не решения.
#[derive(Clone, Copy)]
pub struct Reading {
    pub intent: [f32; INTENT_COUNT],
    pub persona: [f32; PERSONA_COUNT],
    pub features: u16,
    /// false — сеть недоступна, ответ собран по ключевым словам
    pub neural: bool,
}

impl Reading {
    pub fn top_intent(&self) -> (Intent, f32) {
        let i = argmax(&self.intent);
        (Intent::ALL[i], self.intent[i])
    }

    pub fn top_persona(&self) -> (usize, f32) {
        let i = argmax(&self.persona);
        (i, self.persona[i])
    }

    /// Мягкий ответ на вопрос да/нет ∈ [0,1]: P(да) + ½·P(хз),
    /// нормированные на массу «ответных» интентов.
    pub fn answer_value(&self) -> f32 {
        let yes = self.intent[Intent::Yes as usize];
        let no = self.intent[Intent::No as usize];
        let maybe = self.intent[Intent::Maybe as usize];
        let mass = yes + no + maybe;
        if mass < 1e-6 {
            return 0.5;
        }
        (yes + 0.5 * maybe) / mass
    }

    /// Ответ по ключевым словам — запасной путь, если веса не загрузились.
    pub fn from_keywords(input: &[u32]) -> Option<Reading> {
        let feats = featurize(input, 1);
        if feats.len == 0 {
            return None;
        }
        let mut word = [0u32; MAX_WORD_LEN];
        let len = first_word(input, &mut word);
        let w = &word[..len];
        let intent = if matches_any(w, &["да", "ага", "угу", "конечно", "yes", "y", "yep", "ok"])
        {
            Intent::Yes
        } else if matches_any(w, &["нет", "неа", "no", "n", "nope"]) {
            Intent::No
        } else if matches_any(w, &["хз", "незнаю", "может", "maybe", "idk"]) {
            Intent::Maybe
        } else if matches_any(w, &["почему", "why"]) {
            Intent::Why
        } else if matches_any(w, &["заново", "restart"]) {
            Intent::Restart
        } else if matches_any(w, &["стоп", "выход", "stop", "exit", "quit"]) {
            Intent::Stop
        } else if matches_any(w, &["привет", "hello", "hi"]) {
            Intent::Greet
        } else {
            return None;
        };
        let mut reading = Reading {
            intent: [0.0; INTENT_COUNT],
            persona: [1.0 / PERSONA_COUNT as f32; PERSONA_COUNT],
            features: feats.len as u16,
            neural: false,
        };
        reading.intent[intent as usize] = 1.0;
        Some(reading)
    }
}

impl VoodooNet {
    fn load(bytes: &'static [u8]) -> Result<VoodooNet, LoadError> {
        let file = Gguf::parse(bytes).map_err(LoadError::Gguf)?;

        if file.get_str("general.architecture") != Some(ARCHITECTURE) {
            return Err(LoadError::WrongArchitecture);
        }
        if file.get_u32("voodoo.version") != Some(MODEL_VERSION)
            || file.get_u32("voodoo.frac_bits") != Some(FRAC_BITS)
        {
            return Err(LoadError::WrongVersion);
        }
        if file.get_str("voodoo.featurizer") != Some(FEATURIZER_ID) {
            return Err(LoadError::WrongFeaturizer);
        }
        if !labels_match(&file, "voodoo.intent.labels", &INTENT_LABELS)
            || !labels_match(&file, "voodoo.persona.labels", &PERSONA_LABELS)
        {
            return Err(LoadError::BadLabels);
        }

        let buckets = file
            .get_u32("voodoo.buckets")
            .ok_or(LoadError::MissingKey("voodoo.buckets"))?;
        let hidden = file
            .get_u32("voodoo.hidden")
            .ok_or(LoadError::MissingKey("voodoo.hidden"))? as usize;
        if buckets == 0 || buckets > u16::MAX as u32 + 1 || hidden == 0 || hidden > MAX_HIDDEN {
            return Err(LoadError::BadShape("voodoo.hidden/buckets"));
        }
        let s_embed = scale(&file, "voodoo.scale.embed")?;
        let s_intent = scale(&file, "voodoo.scale.intent")?;
        let s_persona = scale(&file, "voodoo.scale.persona")?;

        let checksum = file
            .get_u32("voodoo.checksum")
            .ok_or(LoadError::MissingKey("voodoo.checksum"))?;
        if fnv1a_bytes(file.data_region()) != checksum {
            return Err(LoadError::Checksum);
        }

        let h = hidden as u64;
        let b = buckets as u64;
        let ni = INTENT_COUNT as u64;
        let np = PERSONA_COUNT as u64;
        let embed = tensor(&file, "embed.weight", gguf::ggml::I8, [h, b])?;
        let embed_bias = tensor(&file, "embed.bias", gguf::ggml::I32, [h, 1])?;
        let intent_w = tensor(&file, "intent.weight", gguf::ggml::I8, [h, ni])?;
        let intent_b = tensor(&file, "intent.bias", gguf::ggml::I32, [ni, 1])?;
        let persona_w = tensor(&file, "persona.weight", gguf::ggml::I8, [h, np])?;
        let persona_b = tensor(&file, "persona.bias", gguf::ggml::I32, [np, 1])?;

        let k = 1.0 / (1u32 << FRAC_BITS) as f32;
        Ok(VoodooNet {
            name: file.get_str("general.name").unwrap_or("?"),
            file_len: file.file_len(),
            tensor_count: file.tensor_count,
            kv_count: file.kv_count,
            buckets,
            hidden,
            checksum,
            embed,
            embed_bias,
            intent_w,
            intent_b,
            persona_w,
            persona_b,
            intent_scale: s_embed * s_intent * k,
            persona_scale: s_embed * s_persona * k,
        })
    }

    /// Целочисленный инференс. None — во входе нет ни одного признака.
    pub fn infer_raw(&self, input: &[u32]) -> Option<RawLogits> {
        let feats = featurize(input, self.buckets);
        if feats.len == 0 {
            return None;
        }
        let n = feats.len as i64;
        let hd = self.hidden;

        let mut acc = [0i64; MAX_HIDDEN];
        for &f in feats.as_slice() {
            let row = &self.embed[f as usize * hd..(f as usize + 1) * hd];
            for j in 0..hd {
                acc[j] += row[j] as i8 as i64;
            }
        }
        let mut hidden = [0i64; MAX_HIDDEN];
        for j in 0..hd {
            // Деление с усечением к нулю — так же, как trunc_div в vnn.py
            let h =
                (acc[j] << FRAC_BITS) / n + ((read_i32(self.embed_bias, j) as i64) << FRAC_BITS);
            hidden[j] = if h > 0 { h } else { 0 };
        }

        let mut out = RawLogits {
            features: feats.len as u16,
            intent: [0; INTENT_COUNT],
            persona: [0; PERSONA_COUNT],
        };
        head(&hidden[..hd], self.intent_w, self.intent_b, &mut out.intent);
        head(
            &hidden[..hd],
            self.persona_w,
            self.persona_b,
            &mut out.persona,
        );
        Some(out)
    }

    pub fn read(&self, input: &[u32]) -> Option<Reading> {
        let raw = self.infer_raw(input)?;
        Some(Reading {
            intent: softmax_scaled(&raw.intent, self.intent_scale),
            persona: softmax_scaled(&raw.persona, self.persona_scale),
            features: raw.features,
            neural: true,
        })
    }
}

fn head<const K: usize>(hidden: &[i64], w: &[u8], b: &[u8], out: &mut [i64; K]) {
    let hd = hidden.len();
    for k in 0..K {
        let mut acc = (read_i32(b, k) as i64) << FRAC_BITS;
        let row = &w[k * hd..(k + 1) * hd];
        for j in 0..hd {
            acc += hidden[j] * (row[j] as i8 as i64);
        }
        out[k] = acc;
    }
}

fn scale(file: &Gguf<'static>, key: &'static str) -> Result<f32, LoadError> {
    match file.get_f32(key) {
        Some(s) if s > 0.0 && s < 1.0e6 => Ok(s),
        Some(_) => Err(LoadError::BadShape(key)),
        None => Err(LoadError::MissingKey(key)),
    }
}

/// Тензор нужного типа и формы [ne0, ne1] — иначе ошибка с его именем.
fn tensor(
    file: &Gguf<'static>,
    name: &'static str,
    ggml_type: u32,
    dims: [u64; 2],
) -> Result<&'static [u8], LoadError> {
    let t = file.tensor(name).ok_or(LoadError::MissingTensor(name))?;
    if t.ggml_type != ggml_type
        || t.dims[0] != dims[0]
        || t.dims[1] != dims[1]
        || t.dims[2] != 1
        || t.dims[3] != 1
    {
        return Err(LoadError::BadShape(name));
    }
    Ok(t.data)
}

fn labels_match(file: &Gguf<'static>, key: &str, expected: &[&str]) -> bool {
    let Some(arr) = file.get_array(key) else {
        return false;
    };
    if arr.elem_type != gguf::vtype::STRING || arr.len != expected.len() as u64 {
        return false;
    }
    arr.iter()
        .zip(expected.iter())
        .all(|(v, e)| matches!(v, Value::Str(s) if s == *e))
}

// ============================================================
// Глобальная модель (лениво, один раз)
// ============================================================
static NET: spin::Once<Result<VoodooNet, LoadError>> = spin::Once::new();

pub fn net() -> Result<&'static VoodooNet, LoadError> {
    match NET.call_once(|| VoodooNet::load(ORACLE_GGUF)) {
        Ok(net) => Ok(net),
        Err(e) => Err(*e),
    }
}

/// Главный вход для диалога: нейросеть, а без неё — ключевые слова.
pub fn read(input: &[u32]) -> Option<Reading> {
    match net() {
        Ok(net) => net.read(input),
        Err(_) => Reading::from_keywords(input),
    }
}

// ============================================================
// Featurizer (зеркало vnn.featurize)
// ============================================================
pub struct Features {
    buckets: [u16; MAX_FEATURES],
    len: usize,
}

impl Features {
    pub fn as_slice(&self) -> &[u16] {
        &self.buckets[..self.len]
    }

    fn push(&mut self, hash: u32, buckets: u32) {
        if self.len < MAX_FEATURES {
            self.buckets[self.len] = (hash % buckets) as u16;
            self.len += 1;
        }
    }
}

/// Регистр, ё→е; всё, что не буква/цифра — разделитель.
pub fn normalize_cp(cp: u32) -> u32 {
    let cp = match cp {
        0x41..=0x5A => return cp + 0x20,
        0x410..=0x42F => cp + 0x20,
        _ => cp,
    };
    match cp {
        0x401 | 0x451 => 0x435,
        0x61..=0x7A | 0x30..=0x39 | 0x430..=0x44F => cp,
        _ => SEP,
    }
}

#[inline]
fn fnv_step(h: u32, v: u32) -> u32 {
    (h ^ v).wrapping_mul(FNV_PRIME)
}

fn fnv1a_bytes(data: &[u8]) -> u32 {
    data.iter().fold(FNV_OFFSET, |h, &b| fnv_step(h, b as u32))
}

/// Нормализует вход и режет на слова: (start, len) в `norm`.
fn split_words(
    input: &[u32],
    norm: &mut [u32; MAX_INPUT_CPS],
    spans: &mut [(usize, usize); MAX_WORDS],
) -> usize {
    let n = input.len().min(MAX_INPUT_CPS);
    for i in 0..n {
        norm[i] = normalize_cp(input[i]);
    }
    let mut count = 0;
    let mut i = 0;
    while i < n {
        if norm[i] == SEP {
            i += 1;
            continue;
        }
        let start = i;
        while i < n && norm[i] != SEP {
            i += 1;
        }
        if count < MAX_WORDS {
            spans[count] = (start, (i - start).min(MAX_WORD_LEN));
            count += 1;
        }
    }
    count
}

pub fn featurize(input: &[u32], buckets: u32) -> Features {
    let mut norm = [0u32; MAX_INPUT_CPS];
    let mut spans = [(0usize, 0usize); MAX_WORDS];
    let words = split_words(input, &mut norm, &mut spans);
    let mut out = Features {
        buckets: [0; MAX_FEATURES],
        len: 0,
    };

    for &(start, len) in &spans[..words] {
        let w = &norm[start..start + len];
        let h = w
            .iter()
            .fold(fnv_step(FNV_OFFSET, TAG_WORD), |h, &cp| fnv_step(h, cp));
        out.push(h, buckets);

        // ^слово$ → символьные 2- и 3-граммы
        let padded = |k: usize| -> u32 {
            if k == 0 {
                PAD_LEFT
            } else if k == len + 1 {
                PAD_RIGHT
            } else {
                w[k - 1]
            }
        };
        for n in 2..=3 {
            for i in 0..=(len + 2 - n) {
                let mut h = fnv_step(FNV_OFFSET, TAG_CHAR);
                for k in i..i + n {
                    h = fnv_step(h, padded(k));
                }
                out.push(h, buckets);
            }
        }
    }

    for pair in spans[..words].windows(2) {
        let (a_start, a_len) = pair[0];
        let (b_start, b_len) = pair[1];
        let mut h = fnv_step(FNV_OFFSET, TAG_BIGRAM);
        for &cp in &norm[a_start..a_start + a_len] {
            h = fnv_step(h, cp);
        }
        h = fnv_step(h, SEP);
        for &cp in &norm[b_start..b_start + b_len] {
            h = fnv_step(h, cp);
        }
        out.push(h, buckets);
    }
    out
}

fn first_word(input: &[u32], out: &mut [u32; MAX_WORD_LEN]) -> usize {
    let mut norm = [0u32; MAX_INPUT_CPS];
    let mut spans = [(0usize, 0usize); MAX_WORDS];
    if split_words(input, &mut norm, &mut spans) == 0 {
        return 0;
    }
    let (start, len) = spans[0];
    out[..len].copy_from_slice(&norm[start..start + len]);
    len
}

fn matches_any(word: &[u32], options: &[&str]) -> bool {
    options.iter().any(|opt| {
        let mut chars = opt.chars();
        word.iter()
            .all(|&cp| chars.next().map(|c| c as u32) == Some(cp))
            && chars.next().is_none()
    })
}

// ============================================================
// Числа
// ============================================================
#[inline]
fn read_i32(bytes: &[u8], idx: usize) -> i32 {
    let o = idx * 4;
    i32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]])
}

fn argmax(xs: &[f32]) -> usize {
    let mut best = 0;
    for i in 1..xs.len() {
        if xs[i] > xs[best] {
            best = i;
        }
    }
    best
}

fn softmax_scaled<const K: usize>(acc: &[i64; K], scale: f32) -> [f32; K] {
    let mut out = [0.0f32; K];
    let mut max = f32::MIN;
    for k in 0..K {
        out[k] = acc[k] as f32 * scale;
        if out[k] > max {
            max = out[k];
        }
    }
    let mut sum = 0.0f32;
    for v in &mut out {
        *v = exp_approx(*v - max);
        sum += *v;
    }
    for v in &mut out {
        *v /= sum;
    }
    out
}

/// e^x без libm и без SSE: 2^(x·log2e) = 2^i · 2^f, полином для 2^f.
/// Относительная ошибка < 2e-6 на [-60, 60] — для softmax с запасом.
pub fn exp_approx(x: f32) -> f32 {
    let x = x.clamp(-60.0, 60.0);
    let t = x * core::f32::consts::LOG2_E;
    let mut i = t as i32; // усечение к нулю
    if (i as f32) > t {
        i -= 1; // → floor
    }
    let f = t - i as f32; // [0, 1)
    let p = 1.0
        + f * (0.693_147_2
            + f * (0.240_226_5 + f * (0.055_504_1 + f * (0.009_618_1 + f * 0.001_333_6))));
    f32::from_bits(((i + 127) as u32) << 23) * p
}
