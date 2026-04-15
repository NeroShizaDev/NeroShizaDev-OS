// ============================================================
// UTF-32 Unicode Intent Engine — NeroShiza Way
// ============================================================
// Каждый символ = 32 бита. Всегда. Везде.
// Никакого core::str. Только [u32]. Математика, а не филология.
//
// Русская «Я», тайская «ห», арабская «أ», грузинская «ან» —
// для ядра это числа одинаковой длины. Два за такт на i5.
// ============================================================

/// Максимальная длина команды в кодпоинтах (фиксированный блок)
pub const MAX_WORD_LEN: usize = 16;

/// Вектор Намерения — фиксированный массив u32 кодпоинтов.
/// Каждое слово занимает ровно 64 байта. Без исключений.
pub type IntentVector = [u32; MAX_WORD_LEN];

/// Намерения системы — числовые коды воли Демиурга
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Intent {
    Unknown     = 0x00,
    Help        = 0x01,
    Clear       = 0x02,
    Status      = 0x03,
    Reboot      = 0x04,
    Menger      = 0x10,  // Губка Менгера — демосцена
    Beep        = 0x11,  // PC Speaker — 16-нотная гамма
    Time        = 0x12,  // Тройное время + тригочасы
    Exit        = 0xFE,  // Сброс
    // --- Мультиязычность ---
    LocaleCycle = 0x20,  // Переключить локаль: RU→EN→AR→RU
    LocaleRu    = 0x21,  // Установить RU
    LocaleEn    = 0x22,  // Установить EN
    LocaleAr    = 0x23,  // Установить AR
    ModeLore    = 0x24,  // Переключить в режим Lore
    ModeTech    = 0x25,  // Переключить в режим Technical
    // --- Doom ---
    Doom        = 0x30,  // Запустить Doom (fire demo / полная игра с WAD)
}

/// Превращает срез u32 кодпоинтов в фиксированный IntentVector.
/// Остаток заполняется нулями — детерминированное выравнивание.
pub const fn word_to_vector(codepoints: &[u32]) -> IntentVector {
    let mut v = [0u32; MAX_WORD_LEN];
    let mut i = 0;
    while i < codepoints.len() && i < MAX_WORD_LEN {
        v[i] = codepoints[i];
        i += 1;
    }
    v
}

// ============================================================
// GIGA_DICT — Вселенская Матрица Смыслов
// ============================================================
// Числовые отпечатки слов → ID Намерения.
// Процессор сравнивает два u32 за один «ам» (64 бита).
// Никаких склеек байтов, никакой шизофрении кодировок.
// ============================================================

static GIGA_DICT: &[(IntentVector, Intent)] = &[
    // ==================== EXIT / СБРОС (0xFE) ====================

    // "выход" [в=0x0432 ы=0x044B х=0x0445 о=0x043E д=0x0434]
    (word_to_vector(&[0x0432, 0x044B, 0x0445, 0x043E, 0x0434]), Intent::Exit),

    // "свали" [с=0x0441 в=0x0432 а=0x0430 л=0x043B и=0x0438]
    (word_to_vector(&[0x0441, 0x0432, 0x0430, 0x043B, 0x0438]), Intent::Exit),

    // "exit" [e=0x65 x=0x78 i=0x69 t=0x74]
    (word_to_vector(&[0x0065, 0x0078, 0x0069, 0x0074]), Intent::Exit),

    // "quit" [q=0x71 u=0x75 i=0x69 t=0x74]
    (word_to_vector(&[0x0071, 0x0075, 0x0069, 0x0074]), Intent::Exit),

    // "cdfkb" (свали на EN раскладке)
    (word_to_vector(&[0x0063, 0x0064, 0x0066, 0x006B, 0x0062]), Intent::Exit),

    // "ds[jl" (выход на EN раскладке)
    (word_to_vector(&[0x0064, 0x0073, 0x005B, 0x006A, 0x006C]), Intent::Exit),

    // "اخرج" — арабское "выйди" [alif=0x0627 kha=0x062E ra=0x0631 jim=0x062C]
    (word_to_vector(&[0x0627, 0x062E, 0x0631, 0x062C]), Intent::Exit),

    // "გასვლა" — грузинское "выход"
    (word_to_vector(&[0x10D2, 0x10D0, 0x10E1, 0x10D5, 0x10DA, 0x10D0]), Intent::Exit),

    // "出口" — китайское "выход" [chu=0x51FA kou=0x53E3]
    (word_to_vector(&[0x51FA, 0x53E3]), Intent::Exit),

    // "終了" — японское "конец" [shuu=0x7D42 ryou=0x4E86]
    (word_to_vector(&[0x7D42, 0x4E86]), Intent::Exit),

    // "خروج" — персидское "выход" [khe=0x062E ra=0x0631 vav=0x0648 jim=0x062C]
    (word_to_vector(&[0x062E, 0x0631, 0x0648, 0x062C]), Intent::Exit),

    // ==================== HELP / ПОМОЩЬ (0x01) ====================

    // "помощь" [п=0x043F о=0x043E м=0x043C о=0x043E щ=0x0449 ь=0x044C]
    (word_to_vector(&[0x043F, 0x043E, 0x043C, 0x043E, 0x0449, 0x044C]), Intent::Help),

    // "help"
    (word_to_vector(&[0x0068, 0x0065, 0x006C, 0x0070]), Intent::Help),

    // "?" — универсальный
    (word_to_vector(&[0x003F]), Intent::Help),

    // "помоги"
    (word_to_vector(&[0x043F, 0x043E, 0x043C, 0x043E, 0x0433, 0x0438]), Intent::Help),

    // "مساعدة" — арабское "помощь"
    (word_to_vector(&[0x0645, 0x0633, 0x0627, 0x0639, 0x062F, 0x0629]), Intent::Help),

    // ==================== CLEAR / ОЧИСТКА (0x02) ====================

    // "очистить"
    (word_to_vector(&[0x043E, 0x0447, 0x0438, 0x0441, 0x0442, 0x0438, 0x0442, 0x044C]), Intent::Clear),

    // "cls"
    (word_to_vector(&[0x0063, 0x006C, 0x0073]), Intent::Clear),

    // "clear"
    (word_to_vector(&[0x0063, 0x006C, 0x0065, 0x0061, 0x0072]), Intent::Clear),

    // "مسح" — арабское "очистить"
    (word_to_vector(&[0x0645, 0x0633, 0x062D]), Intent::Clear),

    // ==================== STATUS (0x03) ====================

    // "статус"
    (word_to_vector(&[0x0441, 0x0442, 0x0430, 0x0442, 0x0443, 0x0441]), Intent::Status),

    // "status"
    (word_to_vector(&[0x0073, 0x0074, 0x0061, 0x0074, 0x0075, 0x0073]), Intent::Status),

    // "حالة" — арабское "статус"
    (word_to_vector(&[0x062D, 0x0627, 0x0644, 0x0629]), Intent::Status),

    // ==================== REBOOT (0x04) ====================

    // "ребут"
    (word_to_vector(&[0x0440, 0x0435, 0x0431, 0x0443, 0x0442]), Intent::Reboot),

    // "reboot"
    (word_to_vector(&[0x0072, 0x0065, 0x0062, 0x006F, 0x006F, 0x0074]), Intent::Reboot),

    // "перезагрузка"
    (word_to_vector(&[0x043F, 0x0435, 0x0440, 0x0435, 0x0437, 0x0430,
                      0x0433, 0x0440, 0x0443, 0x0437, 0x043A, 0x0430]), Intent::Reboot),

    // "اعادة" — арабское "перезапуск" без хамзы для простоты ввода
    (word_to_vector(&[0x0627, 0x0639, 0x0627, 0x062F, 0x0629]), Intent::Reboot),

    // ==================== MENGER / ГУБКА (0x10) ====================

    // "губка" [г=0x0433 у=0x0443 б=0x0431 к=0x043A а=0x0430]
    (word_to_vector(&[0x0433, 0x0443, 0x0431, 0x043A, 0x0430]), Intent::Menger),

    // "менгер" [м=0x043C е=0x0435 н=0x043D г=0x0433 е=0x0435 р=0x0440]
    (word_to_vector(&[0x043C, 0x0435, 0x043D, 0x0433, 0x0435, 0x0440]), Intent::Menger),

    // "menger"
    (word_to_vector(&[0x006D, 0x0065, 0x006E, 0x0067, 0x0065, 0x0072]), Intent::Menger),

    // "фрактал" [ф=0x0444 р=0x0440 а=0x0430 к=0x043A т=0x0442 а=0x0430 л=0x043B]
    (word_to_vector(&[0x0444, 0x0440, 0x0430, 0x043A, 0x0442, 0x0430, 0x043B]), Intent::Menger),

    // "vtyuth" (менгер на EN раскладке)
    (word_to_vector(&[0x0076, 0x0074, 0x0079, 0x0075, 0x0074, 0x0068]), Intent::Menger),

    // "fractal"
    (word_to_vector(&[0x0066, 0x0072, 0x0061, 0x0063, 0x0074, 0x0061, 0x006C]), Intent::Menger),

    // ==================== BEEP / ЗВУК (0x11) ====================

    // "звук" [з=0x0437 в=0x0432 у=0x0443 к=0x043A]
    (word_to_vector(&[0x0437, 0x0432, 0x0443, 0x043A]), Intent::Beep),

    // "бипер" [б=0x0431 и=0x0438 п=0x043F е=0x0435 р=0x0440]
    (word_to_vector(&[0x0431, 0x0438, 0x043F, 0x0435, 0x0440]), Intent::Beep),

    // "beep"
    (word_to_vector(&[0x0062, 0x0065, 0x0065, 0x0070]), Intent::Beep),

    // "sound"
    (word_to_vector(&[0x0073, 0x006F, 0x0075, 0x006E, 0x0064]), Intent::Beep),

    // "صوت" — арабское "звук"
    (word_to_vector(&[0x0635, 0x0648, 0x062A]), Intent::Beep),

    // ==================== TIME / ВРЕМЯ (0x12) ====================

    // "время" [в=0x0432 р=0x0440 е=0x0435 м=0x043C я=0x044F]
    (word_to_vector(&[0x0432, 0x0440, 0x0435, 0x043C, 0x044F]), Intent::Time),

    // "часы" [ч=0x0447 а=0x0430 с=0x0441 ы=0x044B]
    (word_to_vector(&[0x0447, 0x0430, 0x0441, 0x044B]), Intent::Time),

    // "time"
    (word_to_vector(&[0x0074, 0x0069, 0x006D, 0x0065]), Intent::Time),

    // "clock"
    (word_to_vector(&[0x0063, 0x006C, 0x006F, 0x0063, 0x006B]), Intent::Time),

    // "триго" [т=0x0442 р=0x0440 и=0x0438 г=0x0433 о=0x043E]
    (word_to_vector(&[0x0442, 0x0440, 0x0438, 0x0433, 0x043E]), Intent::Time),

    // "وقت" — арабское "время"
    (word_to_vector(&[0x0648, 0x0642, 0x062A]), Intent::Time),

    // ==================== LOCALE CYCLE (0x20) ====================

    // "locale"
    (word_to_vector(&[0x006C, 0x006F, 0x0063, 0x0061, 0x006C, 0x0065]), Intent::LocaleCycle),

    // "локаль" [л=0x043B о=0x043E к=0x043A а=0x0430 л=0x043B ь=0x044C]
    (word_to_vector(&[0x043B, 0x043E, 0x043A, 0x0430, 0x043B, 0x044C]), Intent::LocaleCycle),

    // "язык" [я=0x044F з=0x0437 ы=0x044B к=0x043A]
    (word_to_vector(&[0x044F, 0x0437, 0x044B, 0x043A]), Intent::LocaleCycle),

    // "lang"
    (word_to_vector(&[0x006C, 0x0061, 0x006E, 0x0067]), Intent::LocaleCycle),

    // ==================== LOCALE RU (0x21) ====================

    // "locale_ru" → слишком длинно. Используем "ru"
    (word_to_vector(&[0x0072, 0x0075]), Intent::LocaleRu),

    // "рус"
    (word_to_vector(&[0x0440, 0x0443, 0x0441]), Intent::LocaleRu),

    // ==================== LOCALE EN (0x22) ====================

    // "en"
    (word_to_vector(&[0x0065, 0x006E]), Intent::LocaleEn),

    // "eng"
    (word_to_vector(&[0x0065, 0x006E, 0x0067]), Intent::LocaleEn),

    // ==================== LOCALE AR (0x23) ====================

    // "ar"
    (word_to_vector(&[0x0061, 0x0072]), Intent::LocaleAr),

    // "arab"
    (word_to_vector(&[0x0061, 0x0072, 0x0061, 0x0062]), Intent::LocaleAr),

    // "عربي" — арабское "арабский" [ayn=0x0639 ra=0x0631 ba=0x0628 ya=0x064A]
    (word_to_vector(&[0x0639, 0x0631, 0x0628, 0x064A]), Intent::LocaleAr),

    // ==================== MODE LORE (0x24) ====================

    // "lore"
    (word_to_vector(&[0x006C, 0x006F, 0x0072, 0x0065]), Intent::ModeLore),

    // "лор" [л=0x043B о=0x043E р=0x0440]
    (word_to_vector(&[0x043B, 0x043E, 0x0440]), Intent::ModeLore),

    // "shiza" — фирменный режим
    (word_to_vector(&[0x0073, 0x0068, 0x0069, 0x007A, 0x0061]), Intent::ModeLore),

    // ==================== MODE TECH (0x25) ====================

    // "tech"
    (word_to_vector(&[0x0074, 0x0065, 0x0063, 0x0068]), Intent::ModeTech),

    // "тех" [т=0x0442 е=0x0435 х=0x0445]
    (word_to_vector(&[0x0442, 0x0435, 0x0445]), Intent::ModeTech),

    // "technical"
    (word_to_vector(&[0x0074, 0x0065, 0x0063, 0x0068, 0x006E, 0x0069,
                      0x0063, 0x0061, 0x006C]), Intent::ModeTech),

    // ==================== DOOM (0x30) ====================

    // "doom" [d=0x64 o=0x6F o=0x6F m=0x6D]
    (word_to_vector(&[0x0064, 0x006F, 0x006F, 0x006D]), Intent::Doom),

    // "погонять" [п=0x043F о=0x043E г=0x0433 о=0x043E н=0x043D я=0x044F т=0x0442 ь=0x044C]
    (word_to_vector(&[0x043F, 0x043E, 0x0433, 0x043E, 0x043D, 0x044F, 0x0442, 0x044C]), Intent::Doom),

    // "ад" [а=0x0430 д=0x0434] — краткая команда
    (word_to_vector(&[0x0430, 0x0434]), Intent::Doom),

    // "c4" [c=0x0063 4=0x0034] — фирменная кодовая команда NeroShiza
    (word_to_vector(&[0x0063, 0x0034]), Intent::Doom),

    // "fire" [f=0x66 i=0x69 r=0x72 e=0x65]
    (word_to_vector(&[0x0066, 0x0069, 0x0072, 0x0065]), Intent::Doom),

    // "огонь" [о=0x043E г=0x0433 о=0x043E н=0x043D ь=0x044C]
    (word_to_vector(&[0x043E, 0x0433, 0x043E, 0x043D, 0x044C]), Intent::Doom),

    // "النار" — арабское "огонь/ад" [al=0x0627 lam=0x0644 nun=0x0646 alif=0x0627 ra=0x0631]
    (word_to_vector(&[0x0627, 0x0644, 0x0646, 0x0627, 0x0631]), Intent::Doom),
];

// ============================================================
// СУПЕРПОИСК — молниеносное сравнение чисел
// ============================================================

/// Ищет намерение по буферу u32 кодпоинтов.
/// Сравнивает фиксированные блоки по 64 байта — два u32 за такт.
/// Никакого парсинга UTF-8. Только чистая математика.
pub fn lookup_intent(input: &[u32]) -> Intent {
    // Превращаем входной срез в фиксированный вектор
    let mut vec = [0u32; MAX_WORD_LEN];
    let len = if input.len() < MAX_WORD_LEN { input.len() } else { MAX_WORD_LEN };
    let mut i = 0;
    while i < len {
        vec[i] = input[i];
        i += 1;
    }

    // Быстрый поиск по первому кодпоинту (отсечение за 1 такт)
    let first = if len > 0 { vec[0] } else { return Intent::Unknown };

    for &(ref pattern, intent) in GIGA_DICT.iter() {
        // Сначала сравниваем первый кодпоинт — мгновенное отсечение
        if pattern[0] == first && vec == *pattern {
            return intent;
        }
    }

    Intent::Unknown
}

/// Конвертирует char в u32 кодпоинт.
/// Тривиально: в Rust char уже IS Unicode scalar value.
#[inline(always)]
pub fn char_to_codepoint(c: char) -> u32 {
    c as u32
}

/// К какому блоку Unicode принадлежит кодпоинт.
/// Теперь использует ПОЛНУЮ таблицу из 346 блоков Unicode 17.0.
/// Бинарный поиск — O(log 346) ≈ 9 сравнений.
pub fn unicode_block_name(cp: u32) -> &'static str {
    match crate::unicode_blocks::find_block(cp) {
        Some(block) => block.name,
        None => "No Block",
    }
}

/// К какому скрипту (системе письменности) принадлежит кодпоинт.
/// 174 скрипта Unicode 17.0 — все языки человечества.
pub fn unicode_script_name(cp: u32) -> &'static str {
    crate::unicode_scripts::find_script(cp).name()
}

/// General Category кодпоинта (буква, цифра, пунктуация, etc.)
pub fn unicode_category(cp: u32) -> crate::unicode_categories::GeneralCategory {
    crate::unicode_categories::find_category(cp)
}

/// Это буква (любого языка)?
pub fn is_letter(cp: u32) -> bool {
    crate::unicode_categories::find_category(cp).is_letter()
}

/// Это цифра (любой системы счисления)?
pub fn is_number(cp: u32) -> bool {
    crate::unicode_categories::find_category(cp).is_number()
}

/// Количество записей в словаре (для диагностики)
pub fn dict_size() -> usize {
    GIGA_DICT.len()
}

/// Размер словаря в байтах (для отчёта о памяти)
pub fn dict_bytes() -> usize {
    GIGA_DICT.len() * core::mem::size_of::<(IntentVector, Intent)>()
}
