// ============================================================
// UTF-32 Unicode Intent Engine — NeroShizaDev Way
// ============================================================
// Каждый символ = 32 бита, без исключений.
// Работаем с [u32], без преобразований в core::str.
//
// Русская «Я», тайская «ห», арабская «أ», грузинская «ან» —
// для ядра это кодпоинты одинакового формата.
// ============================================================

/// Максимальная длина команды в кодпоинтах (фиксированный блок)
pub const MAX_WORD_LEN: usize = 16;

/// Вектор Намерения — фиксированный массив u32 кодпоинтов.
/// Каждое слово занимает ровно 64 байта. Без исключений.
pub type IntentVector = [u32; MAX_WORD_LEN];

/// Намерения системы — числовые коды команд
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Intent {
    Unknown     = 0x00,
    Help        = 0x01,
    Clear       = 0x02,
    Status      = 0x03,
    Reboot      = 0x04,
    Exit        = 0xFE,  // Сброс
    // --- Мультиязычность ---
    LocaleCycle = 0x20,  // Переключить локаль: RU→EN→AR→RU
    LocaleRu    = 0x21,  // Установить RU
    LocaleEn    = 0x22,  // Установить EN
    LocaleAr    = 0x23,  // Установить AR
    ModeLore    = 0x24,  // Переключить в режим Lore
    ModeTech    = 0x25,  // Переключить в режим Technical
    Apps        = 0x31,  // Запустить меню приложений
    // --- И.Б.И.П. команды ---
    WhoAmI      = 0x40,  // Случайный резидент из Voodoo-матрицы
    Manifest    = 0x41,  // Философия NERO & SHIZA
    Entropy     = 0x42,  // Энтропия Шеннона введённой строки
    Rng         = 0x43,  // Случайное число через RDRAND
    Voodoo      = 0x44,  // Интерактивный Акинатор (Байесовский оракул)
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
// Процессор сравнивает пары u32 (64 бита) за одну операцию.
// Без склейки байтов и смешивания кодировок.
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

    // ==================== APPS MENU (0x31) ====================

    // "apps"
    (word_to_vector(&[0x0061, 0x0070, 0x0070, 0x0073]), Intent::Apps),

    // "menu"
    (word_to_vector(&[0x006D, 0x0065, 0x006E, 0x0075]), Intent::Apps),

    // "launcher"
    (word_to_vector(&[0x006C, 0x0061, 0x0075, 0x006E, 0x0063, 0x0068, 0x0065, 0x0072]), Intent::Apps),

    // "проги"
    (word_to_vector(&[0x043F, 0x0440, 0x043E, 0x0433, 0x0438]), Intent::Apps),

    // "меню"
    (word_to_vector(&[0x043C, 0x0435, 0x043D, 0x044E]), Intent::Apps),

    // "تطبيقات" (apps)
    (word_to_vector(&[0x062A, 0x0637, 0x0628, 0x064A, 0x0642, 0x0627, 0x062A]), Intent::Apps),

    // ==================== WHOAMI (0x40) ====================

    // "whoami"
    (word_to_vector(&[0x0077, 0x0068, 0x006F, 0x0061, 0x006D, 0x0069]), Intent::WhoAmI),

    // "кто я" [к=0x043A т=0x0442 о=0x043E] — первое слово
    (word_to_vector(&[0x043A, 0x0442, 0x043E]), Intent::WhoAmI),

    // "кто"
    (word_to_vector(&[0x043A, 0x0442, 0x043E]), Intent::WhoAmI),

    // "личность"
    (word_to_vector(&[0x043B, 0x0438, 0x0447, 0x043D, 0x043E, 0x0441, 0x0442, 0x044C]), Intent::WhoAmI),

    // ==================== MANIFEST (0x41) ====================

    // "manifest"
    (word_to_vector(&[0x006D, 0x0061, 0x006E, 0x0069, 0x0066, 0x0065, 0x0073, 0x0074]), Intent::Manifest),

    // "манифест" [м=0x043C а=0x0430 н=0x043D и=0x0438 ф=0x0444 е=0x0435 с=0x0441 т=0x0442]
    (word_to_vector(&[0x043C, 0x0430, 0x043D, 0x0438, 0x0444, 0x0435, 0x0441, 0x0442]), Intent::Manifest),

    // "nero" — вызов манифеста
    (word_to_vector(&[0x006E, 0x0065, 0x0072, 0x006F]), Intent::Manifest),

    // "shiza"
    (word_to_vector(&[0x0073, 0x0068, 0x0069, 0x007A, 0x0061]), Intent::Manifest),

    // "нейро" [н=0x043D е=0x0435 й=0x0439 р=0x0440 о=0x043E]
    (word_to_vector(&[0x043D, 0x0435, 0x0439, 0x0440, 0x043E]), Intent::Manifest),

    // "шиза" [ш=0x0448 и=0x0438 з=0x0437 а=0x0430]
    (word_to_vector(&[0x0448, 0x0438, 0x0437, 0x0430]), Intent::Manifest),

    // ==================== ENTROPY (0x42) ====================

    // "entropy"
    (word_to_vector(&[0x0065, 0x006E, 0x0074, 0x0072, 0x006F, 0x0070, 0x0079]), Intent::Entropy),

    // "энтропия" [э=0x044D н=0x043D т=0x0442 р=0x0440 о=0x043E п=0x043F и=0x0438 я=0x044F]
    (word_to_vector(&[0x044D, 0x043D, 0x0442, 0x0440, 0x043E, 0x043F, 0x0438, 0x044F]), Intent::Entropy),

    // "шеннон" [ш=0x0448 е=0x0435 н=0x043D н=0x043D о=0x043E н=0x043D]
    (word_to_vector(&[0x0448, 0x0435, 0x043D, 0x043D, 0x043E, 0x043D]), Intent::Entropy),

    // "shannon"
    (word_to_vector(&[0x0073, 0x0068, 0x0061, 0x006E, 0x006E, 0x006F, 0x006E]), Intent::Entropy),

    // ==================== RNG (0x43) ====================

    // "rng"
    (word_to_vector(&[0x0072, 0x006E, 0x0067]), Intent::Rng),

    // "rand"
    (word_to_vector(&[0x0072, 0x0061, 0x006E, 0x0064]), Intent::Rng),

    // "random"
    (word_to_vector(&[0x0072, 0x0061, 0x006E, 0x0064, 0x006F, 0x006D]), Intent::Rng),

    // "рандом" [р=0x0440 а=0x0430 н=0x043D д=0x0434 о=0x043E м=0x043C]
    (word_to_vector(&[0x0440, 0x0430, 0x043D, 0x0434, 0x043E, 0x043C]), Intent::Rng),

    // "кубик" [к=0x043A у=0x0443 б=0x0431 и=0x0438 к=0x043A]
    (word_to_vector(&[0x043A, 0x0443, 0x0431, 0x0438, 0x043A]), Intent::Rng),

    // ==================== VOODOO (0x44) ====================

    // "voodoo"
    (word_to_vector(&[0x0076, 0x006F, 0x006F, 0x0064, 0x006F, 0x006F]), Intent::Voodoo),

    // "вуду" [в=0x0432 у=0x0443 д=0x0434 у=0x0443]
    (word_to_vector(&[0x0432, 0x0443, 0x0434, 0x0443]), Intent::Voodoo),

    // "акинатор" [а=0x0430 к=0x043A и=0x0438 н=0x043D а=0x0430 т=0x0442 о=0x043E р=0x0440]
    (word_to_vector(&[0x0430, 0x043A, 0x0438, 0x043D, 0x0430, 0x0442, 0x043E, 0x0440]), Intent::Voodoo),

    // "oracle" — оракул
    (word_to_vector(&[0x006F, 0x0072, 0x0061, 0x0063, 0x006C, 0x0065]), Intent::Voodoo),

    // "оракул" [о=0x043E р=0x0440 а=0x0430 к=0x043A у=0x0443 л=0x043B]
    (word_to_vector(&[0x043E, 0x0440, 0x0430, 0x043A, 0x0443, 0x043B]), Intent::Voodoo),

    // ==================== MIRROR ALIASES (RTL typing) ====================
    // Латинские команды задом наперёд — для набора в AR-режиме справа налево.
    // Формат: зеркало_команды → тот же Intent что у оригинала.

    // "tixe" ← exit
    (word_to_vector(&[0x74, 0x69, 0x78, 0x65]), Intent::Exit),
    // "tiuq" ← quit
    (word_to_vector(&[0x74, 0x69, 0x75, 0x71]), Intent::Exit),

    // "pleh" ← help
    (word_to_vector(&[0x70, 0x6C, 0x65, 0x68]), Intent::Help),

    // "raelc" ← clear  ("slc" ← cls уже есть в словаре)
    (word_to_vector(&[0x72, 0x61, 0x65, 0x6C, 0x63]), Intent::Clear),

    // "sutats" ← status
    (word_to_vector(&[0x73, 0x75, 0x74, 0x61, 0x74, 0x73]), Intent::Status),

    // "toober" ← reboot
    (word_to_vector(&[0x74, 0x6F, 0x6F, 0x62, 0x65, 0x72]), Intent::Reboot),

    // "elacol" ← locale
    (word_to_vector(&[0x65, 0x6C, 0x61, 0x63, 0x6F, 0x6C]), Intent::LocaleCycle),
    // "gnal" ← lang
    (word_to_vector(&[0x67, 0x6E, 0x61, 0x6C]), Intent::LocaleCycle),

    // "ur" ← ru
    (word_to_vector(&[0x75, 0x72]), Intent::LocaleRu),

    // "ne" ← en
    (word_to_vector(&[0x6E, 0x65]), Intent::LocaleEn),
    // "gne" ← eng
    (word_to_vector(&[0x67, 0x6E, 0x65]), Intent::LocaleEn),

    // "ra" ← ar
    (word_to_vector(&[0x72, 0x61]), Intent::LocaleAr),
    // "bara" ← arab
    (word_to_vector(&[0x62, 0x61, 0x72, 0x61]), Intent::LocaleAr),

    // "erol" ← lore
    (word_to_vector(&[0x65, 0x72, 0x6F, 0x6C]), Intent::ModeLore),

    // "hcet" ← tech
    (word_to_vector(&[0x68, 0x63, 0x65, 0x74]), Intent::ModeTech),
    // "lacinhcet" ← technical
    (word_to_vector(&[0x6C, 0x61, 0x63, 0x69, 0x6E, 0x68, 0x63, 0x65, 0x74]), Intent::ModeTech),

    // "sapp" ← apps
    (word_to_vector(&[0x73, 0x61, 0x70, 0x70]), Intent::Apps),
    // "unem" ← menu
    (word_to_vector(&[0x75, 0x6E, 0x65, 0x6D]), Intent::Apps),
    // "rehcnual" ← launcher
    (word_to_vector(&[0x72, 0x65, 0x68, 0x63, 0x6E, 0x75, 0x61, 0x6C]), Intent::Apps),

    // "imaohw" ← whoami
    (word_to_vector(&[0x69, 0x6D, 0x61, 0x6F, 0x68, 0x77]), Intent::WhoAmI),

    // "tsefinam" ← manifest
    (word_to_vector(&[0x74, 0x73, 0x65, 0x66, 0x69, 0x6E, 0x61, 0x6D]), Intent::Manifest),
    // "oren" ← nero
    (word_to_vector(&[0x6F, 0x72, 0x65, 0x6E]), Intent::Manifest),

    // "yportne" ← entropy
    (word_to_vector(&[0x79, 0x70, 0x6F, 0x72, 0x74, 0x6E, 0x65]), Intent::Entropy),
    // "nonnahs" ← shannon
    (word_to_vector(&[0x6E, 0x6F, 0x6E, 0x6E, 0x61, 0x68, 0x73]), Intent::Entropy),

    // "gnr" ← rng
    (word_to_vector(&[0x67, 0x6E, 0x72]), Intent::Rng),
    // "dnar" ← rand
    (word_to_vector(&[0x64, 0x6E, 0x61, 0x72]), Intent::Rng),
    // "modnar" ← random
    (word_to_vector(&[0x6D, 0x6F, 0x64, 0x6E, 0x61, 0x72]), Intent::Rng),

    // "oodoov" ← voodoo
    (word_to_vector(&[0x6F, 0x6F, 0x64, 0x6F, 0x6F, 0x76]), Intent::Voodoo),
    // "elcaro" ← oracle
    (word_to_vector(&[0x65, 0x6C, 0x63, 0x61, 0x72, 0x6F]), Intent::Voodoo),
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

/// Fuzzy match: Хэмминг-расстояние между двумя IntentVector'ами.
/// Считает позиции где кодпоинты не совпадают (0 = идеально).
fn hamming_distance(a: &IntentVector, b: &IntentVector) -> u32 {
    let mut dist = 0u32;
    for i in 0..MAX_WORD_LEN {
        if a[i] != b[i] { dist += 1; }
        if a[i] == 0 && b[i] == 0 { break; }
    }
    dist
}

/// Находит ближайшую команду к введённому буферу.
/// Возвращает `(intent, &'static str имя команды, расстояние)`.
/// Если расстояние > порога (4) — возвращает None (слишком далеко).
pub fn closest_intent(input: &[u32]) -> Option<(Intent, &'static str, u32)> {
    const THRESHOLD: u32 = 4;

    // Имена для каждого Intent (для вывода подсказки)
    const INTENT_NAMES: &[(Intent, &str)] = &[
        (Intent::Help,        "help"),
        (Intent::Clear,       "clear"),
        (Intent::Status,      "status"),
        (Intent::Reboot,      "reboot"),
        (Intent::Exit,        "exit"),
        (Intent::Apps,        "apps"),
        (Intent::WhoAmI,      "whoami"),
        (Intent::Manifest,    "manifest"),
        (Intent::Entropy,     "entropy"),
        (Intent::Rng,         "rng"),
        (Intent::Voodoo,      "voodoo"),
        (Intent::LocaleCycle, "locale"),
        (Intent::ModeLore,    "lore"),
        (Intent::ModeTech,    "tech"),
    ];

    let mut vec = [0u32; MAX_WORD_LEN];
    let len = input.len().min(MAX_WORD_LEN);
    let mut i = 0;
    while i < len { vec[i] = input[i]; i += 1; }

    let mut best_dist = u32::MAX;
    let mut best_intent = Intent::Unknown;
    let mut best_name: &'static str = "";

    for &(ref pattern, _intent) in GIGA_DICT.iter() {
        let d = hamming_distance(&vec, pattern);
        if d < best_dist {
            best_dist = d;
            best_intent = _intent;
        }
    }

    if best_dist == 0 || best_dist > THRESHOLD {
        return None;
    }

    // Найти имя для best_intent
    for &(intent, name) in INTENT_NAMES.iter() {
        if intent == best_intent { best_name = name; break; }
    }

    Some((best_intent, best_name, best_dist))
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
