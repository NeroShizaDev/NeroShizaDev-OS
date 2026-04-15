use crate::kernel_messages::Locale;

pub const LOGO_ART: [&str; 12] = [
    "        ████████████",
    "      ██            ██",
    "    ██   ██████████   ██",
    "   ██  ██          ██  ██",
    "  ██  ██   /\\  /\\   ██  ██",
    "  ██ ██   ( o  o )   ██ ██",
    "  ██ ██    \\ ^^ /    ██ ██",
    "  ██  ██  ████████  ██  ██",
    "   ██  ██████████████  ██",
    "    ██   ██████████   ██",
    "      ██            ██",
    "        ████████████",
];
pub const LOGO_TAGLINE: &str = "        #[no_mangle]";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiText {
    Phase2CpuOk,
    Phase3MemoryOk,
    Phase5SpeakerOk,
    Phase5SpeakerFail,
    Phase6FontLocaleOk,
    BootBannerTitle,
    BootBannerUnicode,
    SystemReadyHint,
    MengerDone,
    BeeperStart,
    BeeperDone,
    ChronosHexNoChip,
    ChronosHexDeadBattery,
    ChronosTimeNoChip,
    ChronosTimeDeadBattery,
    ChronosHeader,
    ChronosHuman,
    ChronosHex,
    ChronosPsychotown,
    ChronosPsychotownHint,
    ChronosTrigClock,
    ChronosFooter,
    ChronosTrigAngleFmt,
    ChronosTrigSinFmt,
    ChronosTrigCosFmt,
    ChronosTrigRootsFmt,
    RngHeader,
    RngNotSupported,
    RngSupported,
    RngValueErrorFmt,
    RngValueFmt,
    RngDiceFmt,
    RngDone,
    FpuHeader,
    FpuEntropyLowFmt,
    FpuEntropyHighFmt,
    FpuSqrtFmt,
    FpuLogFmt,
    FpuDone,
    ValidatorPs2Ok,
    ValidatorPs2NoResp,
    ValidatorCmosDead,
    ValidatorCmosStatusFmt,
    ValidatorWarnFmt,
    ValidatorErrFmt,
    ValidatorFatalFmt,
    RtcNoChip,
    RtcDeadBattery,
    RtcUipStuck,
    RtcLineFmt,
    RtcCpuThrottleFmt,
    RtcCpuTempFmt,
    VoodooModeFmt,
    VoodooLoopBreak,
    VoodooTooManyAnswers,
    VoodooEntropyCollapse,
    VoodooDemoHeader,
    VoodooStepFmt,
}

pub fn text(locale: Locale, key: UiText) -> &'static str {
    match (locale, key) {
        (Locale::RuRu, UiText::Phase2CpuOk) => "[Фаза 2] CPU: GDT / IDT / PICS / FPU - ОК",
        (Locale::EnUs, UiText::Phase2CpuOk) => "[Phase 2] CPU: GDT / IDT / PICS / FPU - OK",
        (Locale::ArEg, UiText::Phase2CpuOk) => "[المرحلة 2] CPU: GDT / IDT / PICS / FPU - OK",
        (Locale::RuRu, UiText::Phase3MemoryOk) => "[Фаза 3] Память: загружена загрузчиком - ОК",
        (Locale::EnUs, UiText::Phase3MemoryOk) => "[Phase 3] Memory: bootloader map loaded - OK",
        (Locale::ArEg, UiText::Phase3MemoryOk) => "[المرحلة 3] الذاكرة: خريطة محمل الإقلاع جاهزة - OK",
        (Locale::RuRu, UiText::Phase5SpeakerOk) => "[Фаза 5] Speaker: ОК - подаем сигнал",
        (Locale::EnUs, UiText::Phase5SpeakerOk) => "[Phase 5] Speaker: OK - sending signal",
        (Locale::ArEg, UiText::Phase5SpeakerOk) => "[المرحلة 5] Speaker: OK - ارسال اشارة",
        (Locale::RuRu, UiText::Phase5SpeakerFail) => "[Фаза 5] Speaker: нет ответа",
        (Locale::EnUs, UiText::Phase5SpeakerFail) => "[Phase 5] Speaker: no response",
        (Locale::ArEg, UiText::Phase5SpeakerFail) => "[المرحلة 5] Speaker: لا توجد استجابة",
        (Locale::RuRu, UiText::Phase6FontLocaleOk) => "[Фаза 6] Шрифт: кириллица загружена | Локаль: активна",
        (Locale::EnUs, UiText::Phase6FontLocaleOk) => "[Phase 6] Font: Cyrillic loaded | Locale: active",
        (Locale::ArEg, UiText::Phase6FontLocaleOk) => "[المرحلة 6] الخط: Cyrillic محمل | اللغة: مفعلة",
        (Locale::RuRu, UiText::BootBannerTitle) => "=== NeroShizaDev OS v0.3 ===",
        (Locale::EnUs, UiText::BootBannerTitle) => "=== NeroShizaDev OS v0.3 ===",
        (Locale::ArEg, UiText::BootBannerTitle) => "=== NeroShizaDev OS v0.3 ===",
        (Locale::RuRu, UiText::BootBannerUnicode) => "Unicode 17.0 / UTF-32 / UCS-4",
        (Locale::EnUs, UiText::BootBannerUnicode) => "Unicode 17.0 / UTF-32 / UCS-4",
        (Locale::ArEg, UiText::BootBannerUnicode) => "Unicode 17.0 / UTF-32 / UCS-4",
        (Locale::RuRu, UiText::SystemReadyHint) => "Система готова. Введи 'помощь' или 'help'",
        (Locale::EnUs, UiText::SystemReadyHint) => "System ready. Type 'help'",
        (Locale::ArEg, UiText::SystemReadyHint) => "النظام جاهز. اكتب 'help'",
        (Locale::RuRu, UiText::MengerDone) => "NeroShiza: Губка Менгера завершена.",
        (Locale::EnUs, UiText::MengerDone) => "NeroShiza: Menger sponge complete.",
        (Locale::ArEg, UiText::MengerDone) => "NeroShiza: اكتمل عرض Menger.",
        (Locale::RuRu, UiText::BeeperStart) => "--- 16-нотная гексатоника (PC Speaker) ---",
        (Locale::EnUs, UiText::BeeperStart) => "--- 16-note hexatonic (PC Speaker) ---",
        (Locale::ArEg, UiText::BeeperStart) => "--- مقياس 16 نغمة (PC Speaker) ---",
        (Locale::RuRu, UiText::BeeperDone) => "--- Бипер OK ---",
        (Locale::EnUs, UiText::BeeperDone) => "--- Beeper OK ---",
        (Locale::ArEg, UiText::BeeperDone) => "--- Beeper OK ---",
        (Locale::RuRu, UiText::ChronosHexNoChip) => "[HEX TIME] CMOS чип не отвечает (0xFF)",
        (Locale::EnUs, UiText::ChronosHexNoChip) => "[HEX TIME] CMOS chip does not respond (0xFF)",
        (Locale::ArEg, UiText::ChronosHexNoChip) => "[HEX TIME] شريحة CMOS لا تستجيب (0xFF)",
        (Locale::RuRu, UiText::ChronosHexDeadBattery) => "[HEX TIME] Батарейка CMOS мертва - данные ненадежны",
        (Locale::EnUs, UiText::ChronosHexDeadBattery) => "[HEX TIME] CMOS battery is dead - data unreliable",
        (Locale::ArEg, UiText::ChronosHexDeadBattery) => "[HEX TIME] بطارية CMOS ميتة - البيانات غير موثوقة",
        (Locale::RuRu, UiText::ChronosTimeNoChip) => "[ВРЕМЯ] CMOS чип не отвечает - время недоступно",
        (Locale::EnUs, UiText::ChronosTimeNoChip) => "[TIME] CMOS chip does not respond - time unavailable",
        (Locale::ArEg, UiText::ChronosTimeNoChip) => "[الوقت] شريحة CMOS لا تستجيب - الوقت غير متاح",
        (Locale::RuRu, UiText::ChronosTimeDeadBattery) => "[ВРЕМЯ] Батарейка CMOS мертва - время недостоверно",
        (Locale::EnUs, UiText::ChronosTimeDeadBattery) => "[TIME] CMOS battery is dead - time is unreliable",
        (Locale::ArEg, UiText::ChronosTimeDeadBattery) => "[الوقت] بطارية CMOS ميتة - الوقت غير موثوق",
        (Locale::RuRu, UiText::ChronosHeader) => "=== ВРЕМЯ В ЧЕТЫРЕХ РЕАЛЬНОСТЯХ ===",
        (Locale::EnUs, UiText::ChronosHeader) => "=== TIME IN FOUR REALITIES ===",
        (Locale::ArEg, UiText::ChronosHeader) => "=== الوقت في اربع حقائق ===",
        (Locale::RuRu, UiText::ChronosHuman) => "[Человеческое]",
        (Locale::EnUs, UiText::ChronosHuman) => "[Human]",
        (Locale::ArEg, UiText::ChronosHuman) => "[بشري]",
        (Locale::RuRu, UiText::ChronosHex) => "[Шестнадцатеричное]",
        (Locale::EnUs, UiText::ChronosHex) => "[Hexadecimal]",
        (Locale::ArEg, UiText::ChronosHex) => "[سداسي عشري]",
        (Locale::RuRu, UiText::ChronosPsychotown) => "[Психотаунское - Мир 100]",
        (Locale::EnUs, UiText::ChronosPsychotown) => "[Psychotown - World 100]",
        (Locale::ArEg, UiText::ChronosPsychotown) => "[Psychotown - عالم 100]",
        (Locale::RuRu, UiText::ChronosPsychotownHint) => "  100 дней в месяце, 1200 дней в году",
        (Locale::EnUs, UiText::ChronosPsychotownHint) => "  100 days per month, 1200 days per year",
        (Locale::ArEg, UiText::ChronosPsychotownHint) => "  100 يوم في الشهر، 1200 يوم في السنة",
        (Locale::RuRu, UiText::ChronosTrigClock) => "[Тригочасы Архитектора Хаоса]",
        (Locale::EnUs, UiText::ChronosTrigClock) => "[Chaos Architect TrigClock]",
        (Locale::ArEg, UiText::ChronosTrigClock) => "[ساعة المثلثات - مهندس الفوضى]",
        (Locale::RuRu, UiText::ChronosFooter) => "--- Если ты это читаешь, все в порядке ---",
        (Locale::EnUs, UiText::ChronosFooter) => "--- If you read this, all is well ---",
        (Locale::ArEg, UiText::ChronosFooter) => "--- اذا كنت تقرأ هذا فكل شيء بخير ---",
        (Locale::RuRu, UiText::ChronosTrigAngleFmt) => "  Угол: {}.{:02}*",
        (Locale::EnUs, UiText::ChronosTrigAngleFmt) => "  Angle: {}.{:02}*",
        (Locale::ArEg, UiText::ChronosTrigAngleFmt) => "  زاوية: {}.{:02}*",
        (Locale::RuRu, UiText::ChronosTrigSinFmt) => "  sin = {}{}.{:02}  [{}]",
        (Locale::EnUs, UiText::ChronosTrigSinFmt) => "  sin = {}{}.{:02}  [{}]",
        (Locale::ArEg, UiText::ChronosTrigSinFmt) => "  sin = {}{}.{:02}  [{}]",
        (Locale::RuRu, UiText::ChronosTrigCosFmt) => "  cos = {}{}.{:02}  [{}]",
        (Locale::EnUs, UiText::ChronosTrigCosFmt) => "  cos = {}{}.{:02}  [{}]",
        (Locale::ArEg, UiText::ChronosTrigCosFmt) => "  cos = {}{}.{:02}  [{}]",
        (Locale::RuRu, UiText::ChronosTrigRootsFmt) => "  Корни: sin~{}, cos~{}",
        (Locale::EnUs, UiText::ChronosTrigRootsFmt) => "  Roots: sin~{}, cos~{}",
        (Locale::ArEg, UiText::ChronosTrigRootsFmt) => "  الجذور: sin~{}, cos~{}",
        (Locale::RuRu, UiText::RngHeader) => "--- RNG (тепловой шум CPU) ---",
        (Locale::EnUs, UiText::RngHeader) => "--- RNG (CPU thermal noise) ---",
        (Locale::ArEg, UiText::RngHeader) => "--- RNG (ضجيج CPU الحراري) ---",
        (Locale::RuRu, UiText::RngNotSupported) => "RDRAND не поддерживается!",
        (Locale::EnUs, UiText::RngNotSupported) => "RDRAND is not supported!",
        (Locale::ArEg, UiText::RngNotSupported) => "RDRAND غير مدعوم!",
        (Locale::RuRu, UiText::RngSupported) => "RDRAND: поддерживается",
        (Locale::EnUs, UiText::RngSupported) => "RDRAND: supported",
        (Locale::ArEg, UiText::RngSupported) => "RDRAND: مدعوم",
        (Locale::RuRu, UiText::RngValueErrorFmt) => "  rand[{}] = ОШИБКА",
        (Locale::EnUs, UiText::RngValueErrorFmt) => "  rand[{}] = ERROR",
        (Locale::ArEg, UiText::RngValueErrorFmt) => "  rand[{}] = خطأ",
        (Locale::RuRu, UiText::RngValueFmt) => "  rand[{}] = {:#018X}",
        (Locale::EnUs, UiText::RngValueFmt) => "  rand[{}] = {:#018X}",
        (Locale::ArEg, UiText::RngValueFmt) => "  rand[{}] = {:#018X}",
        (Locale::RuRu, UiText::RngDiceFmt) => "  Кубик: {}",
        (Locale::EnUs, UiText::RngDiceFmt) => "  Dice: {}",
        (Locale::ArEg, UiText::RngDiceFmt) => "  نرد: {}",
        (Locale::RuRu, UiText::RngDone) => "--- RNG OK ---",
        (Locale::EnUs, UiText::RngDone) => "--- RNG OK ---",
        (Locale::ArEg, UiText::RngDone) => "--- RNG OK ---",
        (Locale::RuRu, UiText::FpuHeader) => "--- x87 FPU Demo ---",
        (Locale::EnUs, UiText::FpuHeader) => "--- x87 FPU Demo ---",
        (Locale::ArEg, UiText::FpuHeader) => "--- x87 FPU Demo ---",
        (Locale::RuRu, UiText::FpuEntropyLowFmt) => "Энтропия 'AAAA...' = {}.{}",
        (Locale::EnUs, UiText::FpuEntropyLowFmt) => "Entropy 'AAAA...' = {}.{}",
        (Locale::ArEg, UiText::FpuEntropyLowFmt) => "Entropy 'AAAA...' = {}.{}",
        (Locale::RuRu, UiText::FpuEntropyHighFmt) => "Энтропия 'abcd...' = {}.{}",
        (Locale::EnUs, UiText::FpuEntropyHighFmt) => "Entropy 'abcd...' = {}.{}",
        (Locale::ArEg, UiText::FpuEntropyHighFmt) => "Entropy 'abcd...' = {}.{}",
        (Locale::RuRu, UiText::FpuSqrtFmt) => "sqrt({}) = {}",
        (Locale::EnUs, UiText::FpuSqrtFmt) => "sqrt({}) = {}",
        (Locale::ArEg, UiText::FpuSqrtFmt) => "sqrt({}) = {}",
        (Locale::RuRu, UiText::FpuLogFmt) => "log2({}) = {}",
        (Locale::EnUs, UiText::FpuLogFmt) => "log2({}) = {}",
        (Locale::ArEg, UiText::FpuLogFmt) => "log2({}) = {}",
        (Locale::RuRu, UiText::FpuDone) => "--- FPU OK ---",
        (Locale::EnUs, UiText::FpuDone) => "--- FPU OK ---",
        (Locale::ArEg, UiText::FpuDone) => "--- FPU OK ---",
        (Locale::RuRu, UiText::ValidatorPs2Ok) => "[PS/2] Контроллер: ОК - контроллер обнаружен",
        (Locale::EnUs, UiText::ValidatorPs2Ok) => "[PS/2] Controller: OK - device detected",
        (Locale::ArEg, UiText::ValidatorPs2Ok) => "[PS/2] المتحكم: OK - تم اكتشافه",
        (Locale::RuRu, UiText::ValidatorPs2NoResp) => "[PS/2] Контроллер: нет ответа (USB-only режим?)",
        (Locale::EnUs, UiText::ValidatorPs2NoResp) => "[PS/2] Controller: no response (USB-only mode?)",
        (Locale::ArEg, UiText::ValidatorPs2NoResp) => "[PS/2] المتحكم: لا استجابة (وضع USB فقط؟)",
        (Locale::RuRu, UiText::ValidatorCmosDead) => "[CMOS] ЧИП НЕ ОТВЕЧАЕТ (0xFF) - RTC мертв",
        (Locale::EnUs, UiText::ValidatorCmosDead) => "[CMOS] CHIP NO RESPONSE (0xFF) - RTC dead",
        (Locale::ArEg, UiText::ValidatorCmosDead) => "[CMOS] لا استجابة من الشريحة (0xFF) - RTC ميت",
        (Locale::RuRu, UiText::ValidatorCmosStatusFmt) => "[CMOS] Чип: ОК (StatA=0x{:02X}) | Батарейка: {} | RTC: {}",
        (Locale::EnUs, UiText::ValidatorCmosStatusFmt) => "[CMOS] Chip: OK (StatA=0x{:02X}) | Battery: {} | RTC: {}",
        (Locale::ArEg, UiText::ValidatorCmosStatusFmt) => "[CMOS] Chip: OK (StatA=0x{:02X}) | Battery: {} | RTC: {}",
        (Locale::RuRu, UiText::ValidatorWarnFmt) => "[!] {}",
        (Locale::EnUs, UiText::ValidatorWarnFmt) => "[!] {}",
        (Locale::ArEg, UiText::ValidatorWarnFmt) => "[!] {}",
        (Locale::RuRu, UiText::ValidatorErrFmt) => "[ERR] {} - возврат в shell",
        (Locale::EnUs, UiText::ValidatorErrFmt) => "[ERR] {} - returning to shell",
        (Locale::ArEg, UiText::ValidatorErrFmt) => "[ERR] {} - عودة الى shell",
        (Locale::RuRu, UiText::ValidatorFatalFmt) => "[FATAL] {} - перезагрузка...",
        (Locale::EnUs, UiText::ValidatorFatalFmt) => "[FATAL] {} - rebooting...",
        (Locale::ArEg, UiText::ValidatorFatalFmt) => "[FATAL] {} - اعادة تشغيل...",
        (Locale::RuRu, UiText::RtcNoChip) => "Время: недоступно (чип RTC мертв)",
        (Locale::EnUs, UiText::RtcNoChip) => "Time: unavailable (RTC chip is dead)",
        (Locale::ArEg, UiText::RtcNoChip) => "الوقت: غير متاح (شريحة RTC ميتة)",
        (Locale::RuRu, UiText::RtcDeadBattery) => "Батарейка CMOS: СДОХЛА! Время недостоверно.",
        (Locale::EnUs, UiText::RtcDeadBattery) => "CMOS battery: DEAD! Time is unreliable.",
        (Locale::ArEg, UiText::RtcDeadBattery) => "بطارية CMOS: ميتة! الوقت غير موثوق.",
        (Locale::RuRu, UiText::RtcUipStuck) => "RTC: не готов (UIP завис) - время пропущено",
        (Locale::EnUs, UiText::RtcUipStuck) => "RTC: not ready (UIP stuck) - skipping time read",
        (Locale::ArEg, UiText::RtcUipStuck) => "RTC: غير جاهز (UIP عالق) - تم تجاوز القراءة",
        (Locale::RuRu, UiText::RtcLineFmt) => "Время: {:02}:{:02}:{:02}  {:02}.{:02}.20{:02} | UTC{}",
        (Locale::EnUs, UiText::RtcLineFmt) => "Time: {:02}:{:02}:{:02}  {:02}.{:02}.20{:02} | UTC{}",
        (Locale::ArEg, UiText::RtcLineFmt) => "Time: {:02}:{:02}:{:02}  {:02}.{:02}.20{:02} | UTC{}",
        (Locale::RuRu, UiText::RtcCpuThrottleFmt) => "CPU: ТРОТТЛИНГ! ~{}°C (до TjMax: {}°C)",
        (Locale::EnUs, UiText::RtcCpuThrottleFmt) => "CPU: THROTTLING! ~{}C (to TjMax: {}C)",
        (Locale::ArEg, UiText::RtcCpuThrottleFmt) => "CPU: THROTTLING! ~{}C (to TjMax: {}C)",
        (Locale::RuRu, UiText::RtcCpuTempFmt) => "CPU: ~{}°C | До троттлинга: {}°C",
        (Locale::EnUs, UiText::RtcCpuTempFmt) => "CPU: ~{}C | Margin to throttle: {}C",
        (Locale::ArEg, UiText::RtcCpuTempFmt) => "CPU: ~{}C | الهامش قبل الخنق: {}C",
        (Locale::RuRu, UiText::VoodooModeFmt) => "Режим клеточного автомата: {}",
        (Locale::EnUs, UiText::VoodooModeFmt) => "Cellular automaton mode: {}",
        (Locale::ArEg, UiText::VoodooModeFmt) => "وضع الخلية الالية: {}",
        (Locale::RuRu, UiText::VoodooLoopBreak) => "[ОБРЫВ СВЯЗИ] Выбор вопроса зациклился. Сброс.",
        (Locale::EnUs, UiText::VoodooLoopBreak) => "[LINK BREAK] Question selection looped. Reset.",
        (Locale::ArEg, UiText::VoodooLoopBreak) => "[انقطاع] اختيار السؤال دخل حلقة. اعادة ضبط.",
        (Locale::RuRu, UiText::VoodooTooManyAnswers) => "[ОБРЫВ СВЯЗИ] Слишком много ответов. Сброс.",
        (Locale::EnUs, UiText::VoodooTooManyAnswers) => "[LINK BREAK] Too many answers. Reset.",
        (Locale::ArEg, UiText::VoodooTooManyAnswers) => "[انقطاع] عدد كبير من الاجابات. اعادة ضبط.",
        (Locale::RuRu, UiText::VoodooEntropyCollapse) => "КОЛЛАПС ЭНТРОПИИ: система не сходится, сброс!",
        (Locale::EnUs, UiText::VoodooEntropyCollapse) => "ENTROPY COLLAPSE: system diverges, reset!",
        (Locale::ArEg, UiText::VoodooEntropyCollapse) => "انهيار الانتروبيا: النظام لا يتقارب، اعادة ضبط!",
        (Locale::RuRu, UiText::VoodooDemoHeader) => "=== ДЕМО КЛЕТОЧНОГО АВТОМАТА (Rule90) ===",
        (Locale::EnUs, UiText::VoodooDemoHeader) => "=== CELLULAR AUTOMATON DEMO (Rule90) ===",
        (Locale::ArEg, UiText::VoodooDemoHeader) => "=== عرض الخلية الالية (Rule90) ===",
        (Locale::RuRu, UiText::VoodooStepFmt) => "Шаг {:>2}: [{:.3}, {:.3}, {:.3}, {:.3}, {:.3}]",
        (Locale::EnUs, UiText::VoodooStepFmt) => "Step {:>2}: [{:.3}, {:.3}, {:.3}, {:.3}, {:.3}]",
        (Locale::ArEg, UiText::VoodooStepFmt) => "Step {:>2}: [{:.3}, {:.3}, {:.3}, {:.3}, {:.3}]",
    }
}

pub fn current(key: UiText) -> &'static str {
    text(crate::locale::get_locale(), key)
}

pub fn print_rng_value(i: usize, val: u64) {
    crate::locale::print_localized_fmt(0x0E, format_args!("  rand[{}] = {:#018X}", i, val));
}
pub fn print_rng_error(i: usize) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(0x0C, format_args!("  rand[{}] = ОШИБКА", i)),
        Locale::EnUs => crate::locale::print_localized_fmt(0x0C, format_args!("  rand[{}] = ERROR", i)),
        Locale::ArEg => crate::locale::print_localized_fmt(0x0C, format_args!("  rand[{}] = خطأ", i)),
    }
}
pub fn print_rng_dice(dice: u64) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(0x0E, format_args!("  Кубик: {}", dice)),
        Locale::EnUs => crate::locale::print_localized_fmt(0x0E, format_args!("  Dice: {}", dice)),
        Locale::ArEg => crate::locale::print_localized_fmt(0x0E, format_args!("  نرد: {}", dice)),
    }
}

pub fn print_fpu_sqrt(input: u64, out: u64) {
    crate::locale::print_localized_fmt(0x0E, format_args!("sqrt({}) = {}", input, out));
}
pub fn print_fpu_log(input: u64, out: i64) {
    crate::locale::print_localized_fmt(0x0E, format_args!("log2({}) = {}", input, out));
}
pub fn print_fpu_entropy(label: &'static str, whole: u64, frac: u64) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(0x0E, format_args!("Энтропия '{}' = {}.{}", label, whole, frac)),
        Locale::EnUs | Locale::ArEg => crate::locale::print_localized_fmt(0x0E, format_args!("Entropy '{}' = {}.{}", label, whole, frac)),
    }
}

pub fn print_validator_cmos(status_a: u8, bat: &str, rtc: &str) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(0x0E, format_args!("[CMOS] Чип: ОК (StatA=0x{:02X}) | Батарейка: {} | RTC: {}", status_a, bat, rtc)),
        Locale::EnUs | Locale::ArEg => crate::locale::print_localized_fmt(0x0E, format_args!("[CMOS] Chip: OK (StatA=0x{:02X}) | Battery: {} | RTC: {}", status_a, bat, rtc)),
    }
}
pub fn print_validator_warn(msg: &str) {
    crate::locale::print_localized_fmt(0x0E, format_args!("[!] {}", msg));
}
pub fn print_validator_err(msg: &str) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(0x0C, format_args!("[ERR] {} - возврат в shell", msg)),
        Locale::EnUs => crate::locale::print_localized_fmt(0x0C, format_args!("[ERR] {} - returning to shell", msg)),
        Locale::ArEg => crate::locale::print_localized_fmt(0x0C, format_args!("[ERR] {} - عودة الى shell", msg)),
    }
}
pub fn print_validator_fatal(msg: &str) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(0x0C, format_args!("[FATAL] {} - перезагрузка...", msg)),
        Locale::EnUs => crate::locale::print_localized_fmt(0x0C, format_args!("[FATAL] {} - rebooting...", msg)),
        Locale::ArEg => crate::locale::print_localized_fmt(0x0C, format_args!("[FATAL] {} - اعادة تشغيل...", msg)),
    }
}

pub fn print_rtc_line(h: u8, m: u8, s: u8, d: u8, mo: u8, y: u16) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(0x0E, format_args!("Время (МСК): {:02}:{:02}:{:02}  Дата: {:02}.{:02}.{}", h, m, s, d, mo, y)),
        Locale::EnUs | Locale::ArEg => crate::locale::print_localized_fmt(0x0E, format_args!("Time (UTC+3): {:02}:{:02}:{:02}  Date: {:02}.{:02}.{}", h, m, s, d, mo, y)),
    }
}
pub fn print_rtc_throttle(temp: u32, margin: u32) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(0x0C, format_args!("CPU: ТРОТТЛИНГ! ~{}°C (до TjMax: {}°C)", temp, margin)),
        Locale::EnUs | Locale::ArEg => crate::locale::print_localized_fmt(0x0C, format_args!("CPU: THROTTLING! ~{}C (to TjMax: {}C)", temp, margin)),
    }
}
pub fn print_rtc_temp(temp: u32, margin: u32) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(0x0E, format_args!("CPU: ~{}°C | До троттлинга: {}°C", temp, margin)),
        Locale::EnUs => crate::locale::print_localized_fmt(0x0E, format_args!("CPU: ~{}C | Margin to throttle: {}C", temp, margin)),
        Locale::ArEg => crate::locale::print_localized_fmt(0x0E, format_args!("CPU: ~{}C | الهامش قبل الخنق: {}C", temp, margin)),
    }
}

pub fn print_chronos_angle(deg_whole: i64, deg_frac: i64) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(0x0E, format_args!("  Угол: {}.{:02}*", deg_whole, deg_frac)),
        Locale::EnUs => crate::locale::print_localized_fmt(0x0E, format_args!("  Angle: {}.{:02}*", deg_whole, deg_frac)),
        Locale::ArEg => crate::locale::print_localized_fmt(0x0E, format_args!("  زاوية: {}.{:02}*", deg_whole, deg_frac)),
    }
}
pub fn print_chronos_roots(sin_r: &str, cos_r: &str) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(0x0E, format_args!("  Корни: sin~{}, cos~{}", sin_r, cos_r)),
        Locale::EnUs => crate::locale::print_localized_fmt(0x0E, format_args!("  Roots: sin~{}, cos~{}", sin_r, cos_r)),
        Locale::ArEg => crate::locale::print_localized_fmt(0x0E, format_args!("  الجذور: sin~{}, cos~{}", sin_r, cos_r)),
    }
}

pub fn print_voodoo_mode(mode: &str) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(0x0E, format_args!("Режим клеточного автомата: {}", mode)),
        Locale::EnUs => crate::locale::print_localized_fmt(0x0E, format_args!("Cellular automaton mode: {}", mode)),
        Locale::ArEg => crate::locale::print_localized_fmt(0x0E, format_args!("وضع الخلية الالية: {}", mode)),
    }
}
pub fn print_voodoo_step(step: usize, p0: f32, p1: f32, p2: f32, p3: f32, p4: f32) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(0x0E, format_args!("Шаг {:>2}: [{:.3}, {:.3}, {:.3}, {:.3}, {:.3}]", step, p0, p1, p2, p3, p4)),
        Locale::EnUs | Locale::ArEg => crate::locale::print_localized_fmt(0x0E, format_args!("Step {:>2}: [{:.3}, {:.3}, {:.3}, {:.3}, {:.3}]", step, p0, p1, p2, p3, p4)),
    }
}
