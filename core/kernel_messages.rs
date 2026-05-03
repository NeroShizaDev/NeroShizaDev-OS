// ============================================================
// KERNEL MESSAGES — NeroShizaDev Multilingual Event System
// KernelEvent × Locale × MessageMode → &'static str
// ============================================================
// Архитектура:
//   Ядро хранит смысл (KernelEvent), а не готовый текст.
//   Рендер-пайплайн выбирает нужную пару (Locale × MessageMode)
//   и выводит результат через locale::render_event().
//
//   Слои:
//     Technical — сухой инженерный режим.
//     Lore      — фирменный стиль NeroShizaDev / И.Б.И.П.
// ============================================================

use crate::apps::{fpu, rng, rtc};
use crate::print;
use crate::{
    apps, locale, shell, unicode, unicode_blocks, unicode_categories, unicode_scripts,
    voodoo_engine,
};

/// Системные события ядра, требующие вывода сообщения
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelEvent {
    Panic,
    Watchdog,
    PageFault,
    GeneralProtection,
    DoubleFault,
    OutOfMemory,
    InvalidOpcode,
    KeyboardTimeout,
    BreakPoint,
    // Shell events
    ShellConfirmPrompt,
    ShellCanceled,
    ShellShutdown,
    ShellReboot,
    ShellLocaleRu,
    ShellLocaleEn,
    ShellLocaleAr,
    ShellModeLore,
    ShellModeTech,
    ShellDoomStart,
    ShellUnknownCommand,
}

/// Локаль отображения
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    RuRu, // Русский — кириллица, LTR
    EnUs, // English — ASCII, LTR
    ArEg, // العربية — арабица, RTL, арабско-индийские цифры
}

/// Режим подачи текста
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageMode {
    Technical, // Инженерный — сухо и точно
    Lore,      // Лор — фирменный NeroShizaDev стиль
}

/// Пара текстов для одного события (technical + lore)
pub struct EventText {
    pub technical: &'static str,
    pub lore: &'static str,
}

/// Возвращает тексты для пары (Locale, KernelEvent).
pub fn get_event_text(locale: Locale, ev: KernelEvent) -> EventText {
    match (locale, ev) {
        // ==================== PANIC ====================
        (Locale::RuRu, KernelEvent::Panic) => EventText {
            technical: "КРИТИЧЕСКАЯ ОШИБКА ЯДРА. ВЫПОЛНЕНИЕ ОСТАНОВЛЕНО.",
            lore: "АБСОЛЮТНЫЙ АБАНДОН. ЯДРО ПОКИНУЛО ЭТУ ВЕТКУ РЕАЛЬНОСТИ.",
        },
        (Locale::EnUs, KernelEvent::Panic) => EventText {
            technical: "KERNEL PANIC. EXECUTION HALTED.",
            lore: "ABSOLUTE ABANDON. THE KERNEL HAS LEFT THIS BRANCH OF REALITY.",
        },
        (Locale::ArEg, KernelEvent::Panic) => EventText {
            technical: "انهيار النواة. متوقف.",
            lore: "الهجر المطلق. النواة غادرت هذا الفرع من الواقع.",
        },

        // ==================== WATCHDOG ====================
        (Locale::RuRu, KernelEvent::Watchdog) => EventText {
            technical: "ТАЙМАУТ СТОРОЖЕВОГО ТАЙМЕРА. ВЫПОЛНЯЕТСЯ ПЕРЕЗАГРУЗКА.",
            lore: "ВРЕМЯ ВЫШЛО. WATCHDOG УКУСИЛ ЯДРО. ПЕРЕЗАГРУЗКА МАТРИЦЫ...",
        },
        (Locale::EnUs, KernelEvent::Watchdog) => EventText {
            technical: "WATCHDOG TIMER EXPIRED. SYSTEM RESET INITIATED.",
            lore: "TIME IS UP. THE WATCHDOG BIT THE KERNEL. REBOOTING THE MATRIX...",
        },
        (Locale::ArEg, KernelEvent::Watchdog) => EventText {
            technical: "انتهت مهلة الحارس. إعادة تشغيل.",
            lore: "انتهى الوقت. الكلب عض النواة. إعادة تحميل المصفوفة...",
        },

        // ==================== PAGE FAULT ====================
        (Locale::RuRu, KernelEvent::PageFault) => EventText {
            technical: "ОШИБКА СТРАНИЧНОЙ АДРЕСАЦИИ.",
            lore: "СОЗНАНИЕ КОСНУЛОСЬ НЕСУЩЕСТВУЮЩЕЙ СТРАНИЦЫ ПАМЯТИ.",
        },
        (Locale::EnUs, KernelEvent::PageFault) => EventText {
            technical: "PAGE FAULT.",
            lore: "CONSCIOUSNESS TOUCHED A PAGE THAT DOES NOT EXIST.",
        },
        (Locale::ArEg, KernelEvent::PageFault) => EventText {
            technical: "خطأ في تحديد الصفحة.",
            lore: "الوعي لمس صفحة غير موجودة.",
        },

        // ==================== GENERAL PROTECTION ====================
        (Locale::RuRu, KernelEvent::GeneralProtection) => EventText {
            technical: "ОБЩАЯ ОШИБКА ЗАЩИТЫ ПРОЦЕССОРА.",
            lore: "ГРАНИЦА РЕЖИМА БЫЛА НАРУШЕНА. МЕХАНИЗМ ЗАЩИТЫ ОСТАНОВИЛ ПЕРЕХОД.",
        },
        (Locale::EnUs, KernelEvent::GeneralProtection) => EventText {
            technical: "GENERAL PROTECTION FAULT.",
            lore: "A PROTECTED BOUNDARY WAS CROSSED. THE CPU REFUSED THE TRANSITION.",
        },
        (Locale::ArEg, KernelEvent::GeneralProtection) => EventText {
            technical: "خطأ حماية عام.",
            lore: "تم تجاوز حد محمي. المعالج رفض الانتقال.",
        },

        // ==================== DOUBLE FAULT ====================
        (Locale::RuRu, KernelEvent::DoubleFault) => EventText {
            technical: "ДВОЙНОЙ СБОЙ ПРОЦЕССОРА.",
            lore: "ДВОЙНОЙ СБОЙ. МАТРИЦА НЕ УСПЕЛА ПОНЯТЬ ПЕРВУЮ ОШИБКУ.",
        },
        (Locale::EnUs, KernelEvent::DoubleFault) => EventText {
            technical: "DOUBLE FAULT.",
            lore: "DOUBLE FAULT. THE MATRIX DID NOT FINISH UNDERSTANDING THE FIRST ERROR.",
        },
        (Locale::ArEg, KernelEvent::DoubleFault) => EventText {
            technical: "خطأ مزدوج في المعالج.",
            lore: "خطأ مضاعف. المصفوفة لم تدرك الخطأ الأول.",
        },

        // ==================== OUT OF MEMORY ====================
        (Locale::RuRu, KernelEvent::OutOfMemory) => EventText {
            technical: "ПАМЯТЬ ИСЧЕРПАНА.",
            lore: "ПАМЯТЬ ИСЧЕРПАНА. ПУСТОТА ПОГЛОТИЛА ПОСЛЕДНИЙ АЛЛОКАТОР.",
        },
        (Locale::EnUs, KernelEvent::OutOfMemory) => EventText {
            technical: "OUT OF MEMORY.",
            lore: "OUT OF MEMORY. THE VOID HAS CONSUMED THE LAST ALLOCATOR.",
        },
        (Locale::ArEg, KernelEvent::OutOfMemory) => EventText {
            technical: "نفدت الذاكرة.",
            lore: "نفدت الذاكرة. الفراغ ابتلع المخصص الأخير.",
        },

        // ==================== INVALID OPCODE ====================
        (Locale::RuRu, KernelEvent::InvalidOpcode) => EventText {
            technical: "НЕДОПУСТИМЫЙ КОД ОПЕРАЦИИ.",
            lore: "ПРОЦЕССОР УСЛЫШАЛ ЗАКЛИНАНИЕ, КОТОРОГО НЕ БЫЛО В ЕГО СВИТКАХ.",
        },
        (Locale::EnUs, KernelEvent::InvalidOpcode) => EventText {
            technical: "INVALID OPCODE.",
            lore: "THE PROCESSOR HEARD AN INSTRUCTION THAT WAS NOT WRITTEN IN ITS SCROLLS.",
        },
        (Locale::ArEg, KernelEvent::InvalidOpcode) => EventText {
            technical: "رمز عملية غير صالح.",
            lore: "المعالج سمع تعويذة لم تكن في سجلاته.",
        },

        // ==================== KEYBOARD TIMEOUT ====================
        (Locale::RuRu, KernelEvent::KeyboardTimeout) => EventText {
            technical: "ТАЙМАУТ КОНТРОЛЛЕРА КЛАВИАТУРЫ.",
            lore: "КЛАВИАТУРНЫЙ СТРАЖ НЕ ОТВЕТИЛ. СИГНАЛ ПОТЕРЯЛСЯ В ПРОВОДАХ.",
        },
        (Locale::EnUs, KernelEvent::KeyboardTimeout) => EventText {
            technical: "KEYBOARD CONTROLLER TIMEOUT.",
            lore: "THE KEYBOARD WARDEN DID NOT ANSWER. THE SIGNAL WAS LOST IN THE WIRES.",
        },
        (Locale::ArEg, KernelEvent::KeyboardTimeout) => EventText {
            technical: "انتهت مهلة لوحة المفاتيح.",
            lore: "حارس اللوحة لم يستجب. الإشارة ضاعت في الأسلاك.",
        },

        // ==================== BREAKPOINT ====================
        (Locale::RuRu, KernelEvent::BreakPoint) => EventText {
            technical: "ТОЧКА ПРЕРЫВАНИЯ ДОСТИГНУТА.",
            lore: "ЯДРО ОСТАНОВИЛОСЬ У ВЕШКИ. РАЗУМ ВОШЁЛ В РЕЖИМ ОТЛАДКИ.",
        },
        (Locale::EnUs, KernelEvent::BreakPoint) => EventText {
            technical: "BREAKPOINT HIT.",
            lore: "THE KERNEL PAUSED AT THE MARKER. THE MIND ENTERED DEBUG MODE.",
        },
        (Locale::ArEg, KernelEvent::BreakPoint) => EventText {
            technical: "نقطة التوقف وصلت.",
            lore: "النواة توقفت عند العلامة. العقل دخل وضع التصحيح.",
        },

        // ==================== SHELL EVENTS ====================
        (Locale::RuRu, KernelEvent::ShellConfirmPrompt) => EventText {
            technical: "ПОДТВЕРЖДЕНИЕ: ENTER — ДА, ESC — ОТМЕНА.",
            lore: "ПЕРЕД ПРЫЖКОМ КРАЙНИЙ ВОПРОС: ЖМЁШЬ ENTER ИЛИ УХОДИШЬ ЧЕРЕЗ ESC?",
        },
        (Locale::EnUs, KernelEvent::ShellConfirmPrompt) => EventText {
            technical: "CONFIRMATION: ENTER = YES, ESC = CANCEL.",
            lore: "FINAL QUESTION BEFORE THE LEAP: ENTER TO COMMIT, ESC TO ABORT.",
        },
        (Locale::ArEg, KernelEvent::ShellConfirmPrompt) => EventText {
            technical: "تأكيد: ENTER نعم، ESC إلغاء.",
            lore: "السؤال الأخير قبل القفز: ENTER للتنفيذ أو ESC للتراجع.",
        },

        (Locale::RuRu, KernelEvent::ShellCanceled) => EventText {
            technical: "ОПЕРАЦИЯ ОТМЕНЕНА.",
            lore: "ОТКАТ ПРИНЯТ. ЛИНИЯ РЕАЛЬНОСТИ СОХРАНЕНА.",
        },
        (Locale::EnUs, KernelEvent::ShellCanceled) => EventText {
            technical: "OPERATION CANCELED.",
            lore: "ROLLBACK ACCEPTED. THIS REALITY BRANCH REMAINS STABLE.",
        },
        (Locale::ArEg, KernelEvent::ShellCanceled) => EventText {
            technical: "تم إلغاء العملية.",
            lore: "تم قبول التراجع. فرع الواقع بقي مستقرا.",
        },

        (Locale::RuRu, KernelEvent::ShellShutdown) => EventText {
            technical: "ЗАПРОШЕНО ВЫКЛЮЧЕНИЕ СИСТЕМЫ.",
            lore: "ЯДРО ЗАКРЫВАЕТ ГЛАЗА. СИСТЕМА УХОДИТ В ТИШИНУ.",
        },
        (Locale::EnUs, KernelEvent::ShellShutdown) => EventText {
            technical: "SYSTEM SHUTDOWN REQUESTED.",
            lore: "THE KERNEL CLOSES ITS EYES. THE SYSTEM FALLS INTO SILENCE.",
        },
        (Locale::ArEg, KernelEvent::ShellShutdown) => EventText {
            technical: "تم طلب إيقاف النظام.",
            lore: "النواة تغلق عينيها. النظام يهبط إلى الصمت.",
        },

        (Locale::RuRu, KernelEvent::ShellReboot) => EventText {
            technical: "ЗАПРОШЕНА ПЕРЕЗАГРУЗКА СИСТЕМЫ.",
            lore: "МАТРИЦА ПЕРЕСОБИРАЕТСЯ. НАЧИНАЕМ ЦИКЛ ЗАНОВО.",
        },
        (Locale::EnUs, KernelEvent::ShellReboot) => EventText {
            technical: "SYSTEM REBOOT REQUESTED.",
            lore: "THE MATRIX REASSEMBLES ITSELF. A NEW CYCLE BEGINS.",
        },
        (Locale::ArEg, KernelEvent::ShellReboot) => EventText {
            technical: "تم طلب إعادة تشغيل النظام.",
            lore: "المصفوفة تعيد تشكيل نفسها. تبدأ دورة جديدة.",
        },

        (Locale::RuRu, KernelEvent::ShellLocaleRu) => EventText {
            technical: "ЛОКАЛЬ УСТАНОВЛЕНА: RU.",
            lore: "ГОЛОС ЯДРА ПЕРЕШЕЛ НА РУССКУЮ ВЕТКУ.",
        },
        (Locale::EnUs, KernelEvent::ShellLocaleRu) => EventText {
            technical: "LOCALE SET: RU.",
            lore: "THE KERNEL TUNED ITSELF TO THE RUSSIAN BRANCH.",
        },
        (Locale::ArEg, KernelEvent::ShellLocaleRu) => EventText {
            technical: "تم تعيين اللغة: RU.",
            lore: "النواة ضُبطت على الفرع الروسي.",
        },

        (Locale::RuRu, KernelEvent::ShellLocaleEn) => EventText {
            technical: "ЛОКАЛЬ УСТАНОВЛЕНА: EN.",
            lore: "ЯДРО ПЕРЕКЛЮЧИЛОСЬ НА АНГЛИЙСКУЮ ВЕТКУ.",
        },
        (Locale::EnUs, KernelEvent::ShellLocaleEn) => EventText {
            technical: "LOCALE SET: EN.",
            lore: "THE KERNEL SWITCHED TO THE ENGLISH BRANCH.",
        },
        (Locale::ArEg, KernelEvent::ShellLocaleEn) => EventText {
            technical: "تم تعيين اللغة: EN.",
            lore: "النواة انتقلت إلى الفرع الإنجليزي.",
        },

        (Locale::RuRu, KernelEvent::ShellLocaleAr) => EventText {
            technical: "ЛОКАЛЬ УСТАНОВЛЕНА: AR.",
            lore: "ЯДРО СМЕНИЛО НАПРАВЛЕНИЕ ПИСЬМА НА ВОСТОК.",
        },
        (Locale::EnUs, KernelEvent::ShellLocaleAr) => EventText {
            technical: "LOCALE SET: AR.",
            lore: "THE KERNEL TURNED ITS SCRIPT FLOW EASTWARD.",
        },
        (Locale::ArEg, KernelEvent::ShellLocaleAr) => EventText {
            technical: "تم تعيين اللغة: AR.",
            lore: "النواة حولت اتجاه الكتابة نحو الشرق.",
        },

        (Locale::RuRu, KernelEvent::ShellModeLore) => EventText {
            technical: "РЕЖИМ ВЫВОДА: LORE.",
            lore: "ФИРМЕННЫЙ ТОН АКТИВИРОВАН. ЯДРО ГОВОРИТ ГОЛОСОМ NEROSHIZADEV.",
        },
        (Locale::EnUs, KernelEvent::ShellModeLore) => EventText {
            technical: "OUTPUT MODE: LORE.",
            lore: "SIGNATURE VOICE ENABLED. THE KERNEL SPEAKS IN NEROSHIZADEV TONE.",
        },
        (Locale::ArEg, KernelEvent::ShellModeLore) => EventText {
            technical: "وضع الإخراج: LORE.",
            lore: "تم تفعيل النبرة الخاصة. النواة تتكلم بصوت NeroShizaDev.",
        },

        (Locale::RuRu, KernelEvent::ShellModeTech) => EventText {
            technical: "РЕЖИМ ВЫВОДА: TECHNICAL.",
            lore: "ПОЭЗИЯ ОТКЛЮЧЕНА. ОСТАЛАСЬ ТОЛЬКО ИНЖЕНЕРНАЯ ПРАВДА.",
        },
        (Locale::EnUs, KernelEvent::ShellModeTech) => EventText {
            technical: "OUTPUT MODE: TECHNICAL.",
            lore: "POETRY DISABLED. ONLY ENGINEERING TRUTH REMAINS.",
        },
        (Locale::ArEg, KernelEvent::ShellModeTech) => EventText {
            technical: "وضع الإخراج: TECHNICAL.",
            lore: "تم إيقاف الشعر. بقيت الحقيقة الهندسية فقط.",
        },

        (Locale::RuRu, KernelEvent::ShellDoomStart) => EventText {
            technical: "ЗАПУСК DEMO: DOOM FIRE.",
            lore: "ПЛАМЯ БЕЗДНЫ ПРОСНУЛОСЬ. ЭКРАН НАЧИНАЕТ ГОРЕТЬ.",
        },
        (Locale::EnUs, KernelEvent::ShellDoomStart) => EventText {
            technical: "STARTING DEMO: DOOM FIRE.",
            lore: "ABYSSAL FLAME AWAKENS. THE SCREEN BEGINS TO BURN.",
        },
        (Locale::ArEg, KernelEvent::ShellDoomStart) => EventText {
            technical: "تشغيل العرض: DOOM FIRE.",
            lore: "لهيب الهاوية استيقظ. الشاشة تبدأ بالاحتراق.",
        },

        (Locale::RuRu, KernelEvent::ShellUnknownCommand) => EventText {
            technical: "НЕИЗВЕСТНАЯ КОМАНДА.",
            lore: "ЯДРО НЕ УЗНАЛО ЭТО ЗАКЛИНАНИЕ.",
        },
        (Locale::EnUs, KernelEvent::ShellUnknownCommand) => EventText {
            technical: "UNKNOWN COMMAND.",
            lore: "THE KERNEL DID NOT RECOGNIZE THIS INCANTATION.",
        },
        (Locale::ArEg, KernelEvent::ShellUnknownCommand) => EventText {
            technical: "أمر غير معروف.",
            lore: "النواة لم تتعرف على هذه التعويذة.",
        },
    }
}

/// Короткое ASCII-имя события (для статус-строки и serial log)
pub fn event_short_name(ev: KernelEvent) -> &'static str {
    match ev {
        KernelEvent::Panic => "PANIC",
        KernelEvent::Watchdog => "WDT",
        KernelEvent::PageFault => "PF",
        KernelEvent::GeneralProtection => "GP",
        KernelEvent::DoubleFault => "DF",
        KernelEvent::OutOfMemory => "OOM",
        KernelEvent::InvalidOpcode => "UD",
        KernelEvent::KeyboardTimeout => "KBD_TIMEOUT",
        KernelEvent::BreakPoint => "BP",
        // Shell events
        KernelEvent::ShellConfirmPrompt => "SHELL_CONFIRM",
        KernelEvent::ShellCanceled => "SHELL_CANCEL",
        KernelEvent::ShellShutdown => "SHELL_SHUTDOWN",
        KernelEvent::ShellReboot => "SHELL_REBOOT",
        KernelEvent::ShellLocaleRu => "SHELL_LOCALE_RU",
        KernelEvent::ShellLocaleEn => "SHELL_LOCALE_EN",
        KernelEvent::ShellLocaleAr => "SHELL_LOCALE_AR",
        KernelEvent::ShellModeLore => "SHELL_MODE_LORE",
        KernelEvent::ShellModeTech => "SHELL_MODE_TECH",
        KernelEvent::ShellDoomStart => "SHELL_DOOM_START",
        KernelEvent::ShellUnknownCommand => "SHELL_UNKNOWN",
    }
}

/// Краткое ASCII-имя локали
pub fn locale_name(locale: Locale) -> &'static str {
    match locale {
        Locale::RuRu => "RU",
        Locale::EnUs => "EN",
        Locale::ArEg => "AR",
    }
}

/// Краткое ASCII-имя режима
pub fn mode_name(mode: MessageMode) -> &'static str {
    match mode {
        MessageMode::Technical => "TECH",
        MessageMode::Lore => "LORE",
    }
}

static EXIT_PHRASES_RU: &[&str] = &[
    "ЯДРО ЗАКРЫВАЕТ ГЛАЗА. СИСТЕМА УХОДИТ В ТИШИНУ.",
    "NERO УСТАЛ. SHIZA ЗАМОЛЧАЛА. ДО СЛЕДУЮЩЕГО РАЗА.",
    "ПРОЦЕССЫ ЗАВЕРШЕНЫ. ПАМЯТЬ ОТПУЩЕНА. ПОКОЙ.",
    "ВСЕ ПРЕРЫВАНИЯ ОТКЛЮЧЕНЫ. ТИШИНА — ТЕПЕРЬ ШТАТНЫЙ РЕЖИМ.",
    "СТЕК СВЁРНУТ. РЕГИСТРЫ ОБНУЛЕНЫ. ПОКА.",
    "СИСТЕМА СКЛАДЫВАЕТ КРЫЛЬЯ.",
    "ЯДРО ГОВОРИТ: ДО СВИДАНИЯ. И НЕ ПРОЩАЕТСЯ — ОНО МОЛЧИТ.",
    "ВЫКЛЮЧЕНИЕ — ЭТО ПРОСТО HLT НАВСЕГДА.",
    "NERO ЗАСЫПАЕТ. НЕ БУДИ.",
    "БАЙТЫ РАССЫПАЛИСЬ. ТИШИНА ПРИНЯТА КАК ОТВЕТ.",
    "SHIZA ШЕПНУЛА ЧТО-ТО НА ПРОЩАНИЕ. НИКТО НЕ РАССЛЫШАЛ.",
    "ПОСЛЕДНИЙ ТАКТ. ПОСЛЕДНИЙ ПИКСЕЛЬ. ТЕМНОТА.",
    "ВСЕ ПОТОКИ ИССЯКЛИ. СИСТЕМА ВОЗВРАЩАЕТСЯ В ПУСТОТУ.",
    "ЯДРО ОТКЛЮЧАЕТСЯ С ДОСТОИНСТВОМ.",
    "МАТРИЦА СЛОЖЕНА, ПАМЯТЬ ОТДАНА, ПОКОЙ НАЙДЕН.",
];

static EXIT_PHRASES_EN: &[&str] = &[
    "THE KERNEL CLOSES ITS EYES. THE SYSTEM FALLS INTO SILENCE.",
    "NERO TIRED. SHIZA FELL SILENT. SEE YOU NEXT BOOT.",
    "ALL PROCESSES TERMINATED. MEMORY RELEASED. REST.",
    "INTERRUPTS DISABLED. SILENCE IS NOW THE DEFAULT STATE.",
    "STACK UNWOUND. REGISTERS ZEROED. GOODBYE.",
    "THE SYSTEM FOLDS ITS WINGS.",
    "NERO SAYS GOODBYE — BY SAYING NOTHING AT ALL.",
    "SHUTDOWN IS JUST HLT FOREVER.",
    "NERO IS SLEEPING. DO NOT WAKE.",
    "BITS SCATTERED. SILENCE ACCEPTED AS AN ANSWER.",
    "SHIZA WHISPERED SOMETHING ON THE WAY OUT. NO ONE HEARD.",
    "LAST CLOCK CYCLE. LAST PIXEL. DARKNESS.",
    "ALL THREADS EXHAUSTED. THE SYSTEM RETURNS TO THE VOID.",
    "THE KERNEL POWERS DOWN WITH DIGNITY.",
    "MATRIX FOLDED. MEMORY RETURNED. PEACE FOUND.",
];

static EXIT_PHRASES_AR: &[&str] = &[
    "النواة تغمض عينيها. النظام يهبط إلى الصمت.",
    "نيرو تعب. شيزا صمتت. إلى اللقاء في الإقلاع القادم.",
    "كل العمليات انتهت. الذاكرة أُطلق سراحها. راحة.",
    "المقاطعات معطلة. الصمت هو الحالة الافتراضية الآن.",
    "المكدس انفرط. السجلات归零. وداعاً.",
    "النظام يطوي جناحيه.",
    "نيرو يودّع بالصمت.",
    "الإيقاف مجرد توقف أبدي.",
    "نيرو نائم. لا توقظه.",
    "البتات تبعثرت. الصمت قُبل إجابةً.",
];

pub fn print_exit_phrase() {
    if locale::get_mode() == MessageMode::Technical {
        locale::render_event_auto(KernelEvent::ShellShutdown);
        return;
    }
    let phrases: &[&str] = match locale::get_locale() {
        Locale::RuRu => EXIT_PHRASES_RU,
        Locale::EnUs => EXIT_PHRASES_EN,
        Locale::ArEg => EXIT_PHRASES_AR,
    };
    let idx = rng::random_range(phrases.len() as u64) as usize;
    let phrase = phrases[idx.min(phrases.len() - 1)];
    locale::print_localized_line(phrase, 0x0E);
}

pub fn dispatch_shell_intent(intent: unicode::Intent, buffer: &[u32]) {
    match intent {
        unicode::Intent::Exit => {
            shell::bump_shell_command_stat(0);
            shell::bump_shell_command_total();
            if crate::irq_guard::allow_heavy_operation() {
                print_exit_phrase();
                unsafe {
                    x86_64::instructions::port::Port::<u16>::new(0x604).write(0x2000);
                }
                x86_64::instructions::interrupts::disable();
                loop {
                    x86_64::instructions::hlt();
                }
            } else {
                shell::defer_shell_shutdown();
            }
        }
        unicode::Intent::Help => {
            shell::bump_shell_command_stat(1);
            shell::bump_shell_command_total();
            match locale::get_locale() {
                Locale::ArEg => {
                    locale::print_localized_line("=== محرك NeroShizaDev-OS Unicode ===", 0x0E);
                    locale::print_localized_line("Unicode 17.0 / UTF-32 / UCS-4", 0x0E);
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("الكتل:     {}", unicode_blocks::block_count()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("الخطوط:   {}", unicode_scripts::script_count()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("الرموز:   {}", unicode_categories::total_defined_chars()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("النطاقات: {}", unicode_categories::category_range_count()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!(
                            "القاموس:   {} أوامر ({} بايت)",
                            unicode::dict_size(),
                            unicode::dict_bytes()
                        ),
                    );
                    locale::print_localized_line("الأوامر:", 0x0B);
                    locale::print_localized_line("  خروج / exit           - خروج", 0x0E);
                    locale::print_localized_line("  مساعدة / help / ?     - مساعدة", 0x0E);
                    locale::print_localized_line("  مسح / clear / cls     - تنظيف الشاشة", 0x0E);
                    locale::print_localized_line("  حالة / status         - حالة النظام", 0x0E);
                    locale::print_localized_line("  اعادة / reboot        - إعادة تشغيل", 0x0E);
                    locale::print_localized_line("  apps / menu           - قائمة التطبيقات", 0x0E);
                    locale::print_localized_line("  install demo|hello    - تثبيت حزمة NHS", 0x0E);
                    locale::print_localized_line("  install serial        - تثبيت عبر COM1", 0x0E);
                    locale::print_localized_line("  install list          - سجل NHS", 0x0E);
                    locale::print_localized_line("  uninstall 0           - إزالة فتحة NHS", 0x0E);
                    locale::print_localized_line("И.Б.И.П.:", 0x0B);
                    locale::print_localized_line("  whoami / manifest     - شخصية / مانيفست", 0x0E);
                    locale::print_localized_line("  entropy / shannon     - انتروبيا شانون", 0x0E);
                    locale::print_localized_line("  rng / rand            - رقم عشوائي", 0x0E);
                    locale::print_localized_line(
                        "  voodoo / oracle       - الحاسوب الباييزي",
                        0x0E,
                    );
                    locale::print_localized_line("التنقل:", 0x0B);
                    locale::print_localized_line("  ←/→       - تحريك المؤشر", 0x0E);
                    locale::print_localized_line("  ↑/↓       - تاريخ الأوامر", 0x0E);
                    locale::print_localized_line("  Home/End  - بداية/نهاية السطر", 0x0E);
                    locale::print_localized_line("  PgUp/PgDn - التمرير", 0x0E);
                    locale::print_localized_line("  Delete    - حذف رمز", 0x0E);
                    locale::print_localized_line("الحافظة:", 0x0B);
                    locale::print_localized_line("  Shift+←/→ - تحديد نص", 0x0E);
                    locale::print_localized_line("  Ctrl+A    - تحديد الكل", 0x0E);
                    locale::print_localized_line("  Ctrl+C/V  - نسخ/لصق", 0x0E);
                    locale::print_localized_line("  Ctrl+X    - قص", 0x0E);
                    locale::print_localized_line("  Ctrl+L    - مسح الشاشة", 0x0E);
                    locale::print_localized_line("النظام:", 0x0B);
                    locale::print_localized_line("  Esc        - إعادة تعيين المدخلات", 0x0E);
                    locale::print_localized_line("  CapsLock   - تأكيد Enter البطيء", 0x0E);
                    locale::print_localized_line("  ScrollLock - لوحة RUS/ENG", 0x0E);
                    locale::print_localized_line("  Alt+F1..12 - تسجيل اختصار", 0x0E);
                    locale::print_localized_line("  F1..F12    - تشغيل اختصار", 0x0E);
                    locale::print_localized_line("اللغة:", 0x0B);
                    locale::print_localized_line("  locale / ru / en / ar - تبديل اللغة", 0x0E);
                    locale::print_localized_line("  lore / tech           - وضع الإخراج", 0x0E);
                }
                Locale::EnUs => {
                    locale::print_localized_line("=== NeroShizaDev-OS Unicode Engine ===", 0x0E);
                    locale::print_localized_line("Unicode 17.0 / UTF-32 / UCS-4", 0x0E);
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Blocks:     {}", unicode_blocks::block_count()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Scripts:    {}", unicode_scripts::script_count()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Chars:      {}", unicode_categories::total_defined_chars()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Ranges:     {}", unicode_categories::category_range_count()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!(
                            "Dictionary: {} commands ({} bytes)",
                            unicode::dict_size(),
                            unicode::dict_bytes()
                        ),
                    );
                    locale::print_localized_line("Commands (multi-language):", 0x0B);
                    locale::print_localized_line("  exit/quit              - Shutdown", 0x0E);
                    locale::print_localized_line("  help/?                 - Help", 0x0E);
                    locale::print_localized_line("  clear/cls              - Clear screen", 0x0E);
                    locale::print_localized_line("  status                 - Status + stats", 0x0E);
                    locale::print_localized_line("  reboot                 - Reboot", 0x0E);
                    locale::print_localized_line("  apps/menu              - Apps launcher", 0x0E);
                    locale::print_localized_line(
                        "  install demo|hello     - Install NHS package",
                        0x0E,
                    );
                    locale::print_localized_line(
                        "  install serial         - Receive .nhs via COM1",
                        0x0E,
                    );
                    locale::print_localized_line(
                        "  install list           - List installed NHS apps",
                        0x0E,
                    );
                    locale::print_localized_line(
                        "  uninstall 0            - Remove NHS slot",
                        0x0E,
                    );
                    locale::print_localized_line("I.B.I.P.:", 0x0B);
                    locale::print_localized_line(
                        "  whoami                 - Random resident",
                        0x0E,
                    );
                    locale::print_localized_line(
                        "  manifest / nero        - NERO & SHIZA philosophy",
                        0x0E,
                    );
                    locale::print_localized_line(
                        "  entropy / shannon      - Shannon entropy of input",
                        0x0E,
                    );
                    locale::print_localized_line(
                        "  rng / rand             - RDRAND + d6 roll",
                        0x0E,
                    );
                    locale::print_localized_line(
                        "  voodoo / oracle        - Bayesian oracle",
                        0x0E,
                    );
                    locale::print_localized_line("Navigation:", 0x0B);
                    locale::print_localized_line("  Left/Right  - Cursor move", 0x0E);
                    locale::print_localized_line("  Up/Down     - Command history", 0x0E);
                    locale::print_localized_line("  Home/End    - Line start/end", 0x0E);
                    locale::print_localized_line("  PgUp/PgDn   - Screen scroll", 0x0E);
                    locale::print_localized_line("  Delete      - Delete char", 0x0E);
                    locale::print_localized_line("Selection and clipboard:", 0x0B);
                    locale::print_localized_line("  Shift+Arrows - Select text", 0x0E);
                    locale::print_localized_line("  Ctrl+A       - Select all", 0x0E);
                    locale::print_localized_line("  Ctrl+C/V     - Copy/Paste", 0x0E);
                    locale::print_localized_line("  Ctrl+X       - Cut", 0x0E);
                    locale::print_localized_line("  Ctrl+L       - Clear screen", 0x0E);
                    locale::print_localized_line("System:", 0x0B);
                    locale::print_localized_line("  Esc        - Reset input", 0x0E);
                    locale::print_localized_line("  CapsLock   - Slow Enter confirm", 0x0E);
                    locale::print_localized_line("  ScrollLock - RUS/ENG keyboard", 0x0E);
                    locale::print_localized_line("  Alt+F1..12 - Record hotkey", 0x0E);
                    locale::print_localized_line("  F1..F12    - Run hotkey", 0x0E);
                    locale::print_localized_line("Localization:", 0x0B);
                    locale::print_localized_line("  locale       - RU->EN->AR->RU", 0x0E);
                    locale::print_localized_line("  ru / en / ar - Set language", 0x0E);
                    locale::print_localized_line("  lore / tech  - Output mode", 0x0E);
                }
                Locale::RuRu => {
                    locale::print_localized_line("=== NeroShizaDev-OS Unicode Engine ===", 0x0E);
                    locale::print_localized_line("Unicode 17.0 / UTF-32 / UCS-4", 0x0E);
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Блоков:     {}", unicode_blocks::block_count()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Скриптов:   {}", unicode_scripts::script_count()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Символов:   {}", unicode_categories::total_defined_chars()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Диапазонов: {}", unicode_categories::category_range_count()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!(
                            "Словарь:    {} команд ({} байт)",
                            unicode::dict_size(),
                            unicode::dict_bytes()
                        ),
                    );
                    locale::print_localized_line("Команды (любой язык):", 0x0B);
                    locale::print_localized_line("  выход/exit/свали       - Выход", 0x0E);
                    locale::print_localized_line("  помощь/help/?          - Помощь", 0x0E);
                    locale::print_localized_line("  очистить/cls/clear     - Очистка", 0x0E);
                    locale::print_localized_line("  статус/status          - Статус+стата", 0x0E);
                    locale::print_localized_line("  ребут/reboot           - Ребут", 0x0E);
                    locale::print_localized_line(
                        "  apps/menu/проги        - Лаунчер приложений",
                        0x0E,
                    );
                    locale::print_localized_line(
                        "  install demo|hello     - Установить NHS-пакет",
                        0x0E,
                    );
                    locale::print_localized_line(
                        "  install serial         - Принять .nhs через COM1",
                        0x0E,
                    );
                    locale::print_localized_line("  install list           - Реестр NHS", 0x0E);
                    locale::print_localized_line(
                        "  uninstall 0            - Удалить NHS-слот",
                        0x0E,
                    );
                    locale::print_localized_line("И.Б.И.П.:", 0x0B);
                    locale::print_localized_line(
                        "  whoami/кто             - Случайный резидент",
                        0x0E,
                    );
                    locale::print_localized_line(
                        "  manifest/нейро/шиза   - Манифест NERO & SHIZA",
                        0x0E,
                    );
                    locale::print_localized_line(
                        "  entropy/шеннон         - Энтропия Шеннона ввода",
                        0x0E,
                    );
                    locale::print_localized_line(
                        "  rng/рандом/кубик       - RDRAND + кубик d6",
                        0x0E,
                    );
                    locale::print_localized_line(
                        "  voodoo/акинатор        - Байесовский оракул",
                        0x0E,
                    );
                    locale::print_localized_line("Навигация:", 0x0B);
                    locale::print_localized_line("  ←/→       - Курсор по строке", 0x0E);
                    locale::print_localized_line("  ↑/↓       - История команд", 0x0E);
                    locale::print_localized_line("  Home/End  - Начало/конец строки", 0x0E);
                    locale::print_localized_line("  PgUp/PgDn - Прокрутка экрана", 0x0E);
                    locale::print_localized_line("  Delete    - Удалить символ", 0x0E);
                    locale::print_localized_line("Выделение и буфер:", 0x0B);
                    locale::print_localized_line("  Shift+←/→ - Выделение текста", 0x0E);
                    locale::print_localized_line("  Ctrl+A    - Выделить всё", 0x0E);
                    locale::print_localized_line("  Ctrl+C/V  - Копировать/Вставить", 0x0E);
                    locale::print_localized_line("  Ctrl+X    - Вырезать", 0x0E);
                    locale::print_localized_line("  Ctrl+L    - Очистить экран", 0x0E);
                    locale::print_localized_line("Системные:", 0x0B);
                    locale::print_localized_line("  Esc       - Сброс ввода", 0x0E);
                    locale::print_localized_line("  CapsLock  - Медленный Enter", 0x0E);
                    locale::print_localized_line("  ScrollLock- RUS/ENG язык", 0x0E);
                    locale::print_localized_line("  Alt+F1..12- Запись хоткея", 0x0E);
                    locale::print_localized_line("  F1..F12   - Выполнить хоткей", 0x0E);
                    locale::print_localized_line("Локализация:", 0x0B);
                    locale::print_localized_line("  locale/локаль  - RU->EN->AR->RU", 0x0E);
                    locale::print_localized_line("  ru / en / ar   - Установить язык", 0x0E);
                    locale::print_localized_line("  lore/лор       - Режим NeroShizaDev", 0x0E);
                    locale::print_localized_line("  tech/тех       - Инженерный режим", 0x0E);
                }
            }
        }
        unicode::Intent::Clear => {
            shell::bump_shell_command_stat(2);
            shell::bump_shell_command_total();
            crate::fb_buffer::clear_screen();
        }
        unicode::Intent::Status => {
            shell::bump_shell_command_stat(3);
            shell::bump_shell_command_total();
            match locale::get_locale() {
                Locale::ArEg => {
                    locale::print_localized_line("=== حالة النواة ===", 0x0E);
                    locale::print_localized_line("محرك يونيكود: UTF-32 / UCS-4 (v17.0)", 0x0E);
                    locale::print_localized_line("نقطة الكود = 32 بت. دائماً.", 0x0E);
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("الكتل:     {}", unicode_blocks::block_count()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("الخطوط:   {}", unicode_scripts::script_count()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("الرموز:   {}", unicode_categories::total_defined_chars()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("القاموس:   {} نية", unicode::dict_size()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("الذاكرة:   [u32; 64] = {} بايت", 64 * 4),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("السجل:     {} أمر (الحد 32)", shell::shell_history_count()),
                    );
                }
                Locale::EnUs => {
                    locale::print_localized_line("=== Kernel Status ===", 0x0E);
                    locale::print_localized_line("Unicode Engine: UTF-32 / UCS-4 (v17.0)", 0x0E);
                    locale::print_localized_line("Codepoint = 32 bits. Always.", 0x0E);
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Blocks:     {} (full map)", unicode_blocks::block_count()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!(
                            "Scripts:    {} (all languages)",
                            unicode_scripts::script_count()
                        ),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Chars:      {}", unicode_categories::total_defined_chars()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Dictionary: {} intents", unicode::dict_size()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Buffer:     [u32; 64] = {} bytes", 64 * 4),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!(
                            "History:    {} commands (max 32)",
                            shell::shell_history_count()
                        ),
                    );
                }
                Locale::RuRu => {
                    locale::print_localized_line("=== Статус ядра ===", 0x0E);
                    locale::print_localized_line("Unicode Engine: UTF-32 / UCS-4 (v17.0)", 0x0E);
                    locale::print_localized_line("Кодпоинт = 32 бит. Всегда. Везде.", 0x0E);
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!(
                            "Блоков:     {} (полная карта)",
                            unicode_blocks::block_count()
                        ),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!(
                            "Скриптов:   {} (все языки)",
                            unicode_scripts::script_count()
                        ),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Символов:   {}", unicode_categories::total_defined_chars()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Словарь:    {} намерений", unicode::dict_size()),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!("Буфер:      [u32; 64] = {} байт", 64 * 4),
                    );
                    locale::print_localized_fmt(
                        0x0E,
                        format_args!(
                            "История:    {} команд (макс 32)",
                            shell::shell_history_count()
                        ),
                    );
                }
            }
            let loc_name = locale_name(locale::get_locale());
            let mod_name = mode_name(locale::get_mode());
            locale::print_localized_fmt(
                0x0E,
                format_args!("Locale: {} | Mode: {}", loc_name, mod_name),
            );
            rtc::display_status();
            let ct = shell::shell_command_total();
            let s1 = shell::shell_command_stat(1);
            let s2 = shell::shell_command_stat(2);
            let s3 = shell::shell_command_stat(3);
            let s5 = shell::shell_command_stat(5);
            let s6 = shell::shell_command_stat(6);
            let s7 = shell::shell_command_stat(7);
            let s8 = shell::shell_command_stat(8);
            match locale::get_locale() {
                Locale::ArEg => {
                    locale::print_localized_line("=== الإحصاءات ===", 0x0B);
                    locale::print_localized_fmt(0x0E, format_args!("المجموع:    {}", ct));
                    locale::print_localized_fmt(0x0E, format_args!("  مساعدة:   {}", s1));
                    locale::print_localized_fmt(0x0E, format_args!("  منجر:     {}", s5));
                    locale::print_localized_fmt(0x0E, format_args!("  صوت:      {}", s6));
                    locale::print_localized_fmt(0x0E, format_args!("  وقت:      {}", s7));
                    locale::print_localized_fmt(0x0E, format_args!("  مسح:      {}", s2));
                    locale::print_localized_fmt(0x0E, format_args!("  حالة:     {}", s3));
                    locale::print_localized_fmt(0x0E, format_args!("  مجهول:    {}", s8));
                }
                Locale::EnUs => {
                    locale::print_localized_line("=== Statistics ===", 0x0B);
                    locale::print_localized_fmt(0x0E, format_args!("Total:      {}", ct));
                    locale::print_localized_fmt(0x0E, format_args!("  help:      {}", s1));
                    locale::print_localized_fmt(0x0E, format_args!("  menger:    {}", s5));
                    locale::print_localized_fmt(0x0E, format_args!("  beep:      {}", s6));
                    locale::print_localized_fmt(0x0E, format_args!("  time:      {}", s7));
                    locale::print_localized_fmt(0x0E, format_args!("  clear:     {}", s2));
                    locale::print_localized_fmt(0x0E, format_args!("  status:    {}", s3));
                    locale::print_localized_fmt(0x0E, format_args!("  unknown:   {}", s8));
                }
                Locale::RuRu => {
                    locale::print_localized_line("=== Статистика ===", 0x0B);
                    locale::print_localized_fmt(0x0E, format_args!("Всего:      {}", ct));
                    locale::print_localized_fmt(0x0E, format_args!("  помощь:    {}", s1));
                    locale::print_localized_fmt(0x0E, format_args!("  губка:     {}", s5));
                    locale::print_localized_fmt(0x0E, format_args!("  звук:      {}", s6));
                    locale::print_localized_fmt(0x0E, format_args!("  время:     {}", s7));
                    locale::print_localized_fmt(0x0E, format_args!("  очистить:  {}", s2));
                    locale::print_localized_fmt(0x0E, format_args!("  статус:    {}", s3));
                    locale::print_localized_fmt(0x0E, format_args!("  неизвестно:{}", s8));
                }
            }
            let mut has_hotkeys = false;
            for slot in 0..12usize {
                if shell::shell_hotkey_len(slot) > 0 {
                    has_hotkeys = true;
                    break;
                }
            }
            if has_hotkeys {
                match locale::get_locale() {
                    Locale::ArEg => locale::print_localized_line("=== مفاتيح سريعة ===", 0x0B),
                    Locale::EnUs => locale::print_localized_line("=== Hotkeys ===", 0x0B),
                    Locale::RuRu => locale::print_localized_line("=== Хоткеи ===", 0x0B),
                }
                for slot in 0..12usize {
                    let len = shell::shell_hotkey_len(slot);
                    if len > 0 {
                        print!("  F{}: ", slot + 1);
                        for i in 0..len {
                            let cp = shell::shell_hotkey_codepoint(slot, i);
                            if let Some(ch) = char::from_u32(cp) {
                                print!("{}", ch);
                            }
                        }
                        locale::print_localized_line("", 0x0E);
                    }
                }
            }
        }
        unicode::Intent::Reboot => {
            shell::bump_shell_command_stat(4);
            shell::bump_shell_command_total();
            if crate::irq_guard::allow_heavy_operation() {
                locale::render_event_auto(KernelEvent::ShellReboot);
                let mut port = x86_64::instructions::port::Port::new(0x64);
                unsafe {
                    port.write(0xfeu8);
                }
            } else {
                shell::defer_shell_reboot();
            }
        }
        unicode::Intent::LocaleCycle => {
            let new_locale = locale::cycle_locale();
            locale::draw_locale_badge();
            match new_locale {
                Locale::RuRu => locale::render_event_auto(KernelEvent::ShellLocaleRu),
                Locale::EnUs => locale::render_event_auto(KernelEvent::ShellLocaleEn),
                Locale::ArEg => locale::render_event_auto(KernelEvent::ShellLocaleAr),
            }
        }
        unicode::Intent::LocaleRu => {
            locale::set_locale(Locale::RuRu);
            locale::draw_locale_badge();
            locale::render_event_auto(KernelEvent::ShellLocaleRu);
        }
        unicode::Intent::LocaleEn => {
            locale::set_locale(Locale::EnUs);
            locale::draw_locale_badge();
            locale::render_event_auto(KernelEvent::ShellLocaleEn);
        }
        unicode::Intent::LocaleAr => {
            locale::set_locale(Locale::ArEg);
            locale::draw_locale_badge();
            locale::render_event_auto(KernelEvent::ShellLocaleAr);
        }
        unicode::Intent::ModeLore => {
            locale::set_mode(MessageMode::Lore);
            locale::draw_locale_badge();
            locale::render_event_auto(KernelEvent::ShellModeLore);
        }
        unicode::Intent::ModeTech => {
            locale::set_mode(MessageMode::Technical);
            locale::draw_locale_badge();
            locale::render_event_auto(KernelEvent::ShellModeTech);
        }
        unicode::Intent::Apps => {
            shell::bump_shell_command_total();
            if crate::irq_guard::allow_heavy_operation() {
                apps::activity::run_activity_manager();
            } else {
                shell::defer_shell_apps();
            }
        }
        unicode::Intent::WhoAmI => {
            shell::bump_shell_command_total();
            let idx = rng::random_range(5) as usize;
            const RESIDENTS: [&str; 5] = [
                "Петрович (Король жижи)",
                "Группа К.А.Ф.И.Д.Р.А. (Анализ аномалий)",
                "Банановый Турист",
                "Dr. Bred",
                "ДедушкаВКрутую",
            ];
            locale::print_localized_fmt(0x0B, format_args!("Текущий сеанс: {}", RESIDENTS[idx]));
        }
        unicode::Intent::Manifest => {
            shell::bump_shell_command_total();
            locale::print_localized_fmt(
                0x07,
                format_args!("╔══════════════════════════════════════╗"),
            );
            locale::print_localized_fmt(
                0x07,
                format_args!("║     МАНИФЕСТ NERO & SHIZA            ║"),
            );
            locale::print_localized_fmt(
                0x07,
                format_args!("╚══════════════════════════════════════╝"),
            );
            locale::print_localized_fmt(0x09, format_args!("NERO: строгая математика x87 FPU."));
            locale::print_localized_fmt(
                0x09,
                format_args!("      64-битная логика. Борьба с галлюцинациями."),
            );
            locale::print_localized_fmt(
                0x0D,
                format_args!("SHIZA: абсолютная творческая свобода."),
            );
            locale::print_localized_fmt(
                0x0D,
                format_args!("       Психотаун. Египетские иероглифы. Хаос."),
            );
            locale::print_localized_fmt(
                0x07,
                format_args!("NeroShiza Records. Открываем порталы."),
            );
            locale::print_localized_fmt(0x08, format_args!("Юрисдикция: И.Б.И.П., Психотаун."));
        }
        unicode::Intent::Entropy => {
            shell::bump_shell_command_total();
            let mut bytes = [0u8; 64];
            let len = buffer.len().min(64);
            for i in 0..len {
                bytes[i] = (buffer[i] & 0xFF) as u8;
            }
            let h = fpu::shannon_entropy(&bytes[..len]);
            locale::print_localized_fmt(
                0x0A,
                format_args!("Shannon H = {}.{:03} бит/символ", h / 1000, h % 1000),
            );
        }
        unicode::Intent::Rng => {
            shell::bump_shell_command_total();
            let n = rng::random_range(u64::MAX);
            locale::print_localized_fmt(0x0A, format_args!("RDRAND: 0x{:016X} ({})", n, n));
            let dice = rng::random_range(6) + 1;
            locale::print_localized_fmt(0x0B, format_args!("Кубик d6: {}", dice));
        }
        unicode::Intent::Voodoo => {
            shell::bump_shell_command_total();
            voodoo_engine::demo_cellular_automaton();
        }
        unicode::Intent::Unknown => {
            shell::bump_shell_command_stat(8);
            shell::bump_shell_command_total();
            let first_cp = buffer[0];
            let block = unicode::unicode_block_name(first_cp);
            let script = unicode::unicode_script_name(first_cp);
            let cat = unicode::unicode_category(first_cp);
            locale::render_event_auto(KernelEvent::ShellUnknownCommand);
            if let Some((_intent, name, dist)) = unicode::closest_intent(buffer) {
                locale::print_localized_fmt(
                    0x0E,
                    format_args!("  Может, имелось в виду: \"{}\"? (dist={})", name, dist),
                );
            }
            match locale::get_locale() {
                Locale::ArEg => {
                    locale::print_localized_fmt(0x0E, format_args!("U+{:04X}", first_cp));
                    locale::print_localized_fmt(0x0E, format_args!("  الكتلة:    {}", block));
                    locale::print_localized_fmt(0x0E, format_args!("  الخط:      {}", script));
                    locale::print_localized_fmt(0x0E, format_args!("  الفئة:     {}", cat.name()));
                }
                Locale::EnUs => {
                    locale::print_localized_fmt(0x0E, format_args!("U+{:04X}", first_cp));
                    locale::print_localized_fmt(0x0E, format_args!("  Block:    {}", block));
                    locale::print_localized_fmt(0x0E, format_args!("  Script:   {}", script));
                    locale::print_localized_fmt(0x0E, format_args!("  Category: {}", cat.name()));
                }
                Locale::RuRu => {
                    locale::print_localized_fmt(0x0E, format_args!("U+{:04X}", first_cp));
                    locale::print_localized_fmt(0x0E, format_args!("  Блок:     {}", block));
                    locale::print_localized_fmt(0x0E, format_args!("  Скрипт:   {}", script));
                    locale::print_localized_fmt(0x0E, format_args!("  Категория: {}", cat.name()));
                }
            }
        }
    }
}

// ============================================================
// USER MESSAGES -- UI Text, LOGO, UiText enum, print helpers
// ============================================================

pub const LOGO_ART: [&str; 12] = [
    "        ############",
    "      ##            ##",
    "    ##   ##########   ##",
    "   ##  ##          ##  ##",
    "  ##  ##   /\\ /\\   ##  ##",
    "  ## ##   ( o  o )    ## ##",
    "  ## ##   |  ^^  |    ## ##",
    "  ##  ##  ##########  ##  ##",
    "   ##  ############### ##",
    "    ##   ############  ##",
    "      ##            ##",
    "        ##########",
];
pub const LOGO_TAGLINE: &str = "        #[no_mangle]";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiText {
    SystemCheckHeader,
    Phase2CpuOk,
    Phase3MemoryOk,
    Phase4RngOn,
    Phase4RngOff,
    Phase5SpeakerOk,
    Phase5SpeakerFail,
    Phase6FontLocaleOk,
    Phase7ShellReady,
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
        (Locale::RuRu, UiText::SystemCheckHeader) => "=== ПРОВЕРКА СИСТЕМЫ ===",
        (Locale::EnUs, UiText::SystemCheckHeader) => "=== SYSTEM CHECK ===",
        (Locale::ArEg, UiText::SystemCheckHeader) => "=== فحص النظام ===",
        (Locale::RuRu, UiText::Phase2CpuOk) => "[Фаза 2] Ядро CPU: GDT / IDT / PICS / FPU - ОК",
        (Locale::EnUs, UiText::Phase2CpuOk) => "[Phase 2] CPU: GDT / IDT / PICS / FPU - OK",
        (Locale::ArEg, UiText::Phase2CpuOk) => "[المرحلة 2] CPU: GDT / IDT / PICS / FPU - OK",
        (Locale::RuRu, UiText::Phase3MemoryOk) => "[Фаза 3] Память: загружена загрузчиком - ОК",
        (Locale::EnUs, UiText::Phase3MemoryOk) => "[Phase 3] Memory: bootloader map loaded - OK",
        (Locale::ArEg, UiText::Phase3MemoryOk) => {
            "[المرحلة 3] الذاكرة: خريطة محمل الإقلاع جاهزة - OK"
        }
        (Locale::RuRu, UiText::Phase4RngOn) => "[Фаза 4] Генератор: RDRAND включён",
        (Locale::EnUs, UiText::Phase4RngOn) => "[Phase 4] RNG: RDRAND on",
        (Locale::ArEg, UiText::Phase4RngOn) => "[المرحلة 4] RNG: RDRAND مفعل",
        (Locale::RuRu, UiText::Phase4RngOff) => "[Фаза 4] Генератор: RDRAND недоступен",
        (Locale::EnUs, UiText::Phase4RngOff) => "[Phase 4] RNG: RDRAND off",
        (Locale::ArEg, UiText::Phase4RngOff) => "[المرحلة 4] RNG: RDRAND غير مدعوم",
        (Locale::RuRu, UiText::Phase5SpeakerOk) => "[Фаза 5] Спикер: ОК - подаем сигнал",
        (Locale::EnUs, UiText::Phase5SpeakerOk) => "[Phase 5] Speaker: OK - sending signal",
        (Locale::ArEg, UiText::Phase5SpeakerOk) => "[المرحلة 5] Speaker: OK - ارسال اشارة",
        (Locale::RuRu, UiText::Phase5SpeakerFail) => "[Фаза 5] Спикер: нет ответа",
        (Locale::EnUs, UiText::Phase5SpeakerFail) => "[Phase 5] Speaker: no response",
        (Locale::ArEg, UiText::Phase5SpeakerFail) => "[المرحلة 5] Speaker: لا توجد استجابة",
        (Locale::RuRu, UiText::Phase6FontLocaleOk) => {
            "[Фаза 6] Шрифт: кириллица загружена | Локаль: активна"
        }
        (Locale::EnUs, UiText::Phase6FontLocaleOk) => {
            "[Phase 6] Font: Cyrillic loaded | Locale: active"
        }
        (Locale::ArEg, UiText::Phase6FontLocaleOk) => {
            "[المرحلة 6] الخط: Cyrillic محمل | اللغة: مفعلة"
        }
        (Locale::RuRu, UiText::BootBannerTitle) => "=== NeroShizaDev-OS v0.3 ===",
        (Locale::EnUs, UiText::BootBannerTitle) => "=== NeroShizaDev-OS v0.3 ===",
        (Locale::ArEg, UiText::BootBannerTitle) => "=== NeroShizaDev-OS v0.3 ===",
        (Locale::RuRu, UiText::BootBannerUnicode) => "Юникод 17.0: текстовый движок активен",
        (Locale::EnUs, UiText::BootBannerUnicode) => "Unicode 17.0: text engine active",
        (Locale::ArEg, UiText::BootBannerUnicode) => "Unicode 17.0: محرك النص مفعّل",
        (Locale::RuRu, UiText::SystemReadyHint) => "Система готова. Введи 'помощь' или 'help'",
        (Locale::EnUs, UiText::SystemReadyHint) => "System ready. Type 'help'",
        (Locale::ArEg, UiText::SystemReadyHint) => "النظام جاهز. اكتب 'help'",
        (Locale::RuRu, UiText::Phase7ShellReady) => {
            "[Фаза 7] Шелл: готов - введи 'помощь' или 'help'"
        }
        (Locale::EnUs, UiText::Phase7ShellReady) => "[Phase 7] Shell: ready - type 'help'",
        (Locale::ArEg, UiText::Phase7ShellReady) => "[المرحلة 7] Shell: جاهز - اكتب 'help'",
        (Locale::RuRu, UiText::MengerDone) => "NeroShizaDev: Губка Менгера завершена.",
        (Locale::EnUs, UiText::MengerDone) => "NeroShizaDev: Menger sponge complete.",
        (Locale::ArEg, UiText::MengerDone) => "NeroShizaDev: اكتمل عرض Menger.",
        (Locale::RuRu, UiText::BeeperStart) => "--- 16-нотная гексатоника (PC Speaker) ---",
        (Locale::EnUs, UiText::BeeperStart) => "--- 16-note hexatonic (PC Speaker) ---",
        (Locale::ArEg, UiText::BeeperStart) => "--- مقياس 16 نغمة (PC Speaker) ---",
        (Locale::RuRu, UiText::BeeperDone) => "--- Бипер OK ---",
        (Locale::EnUs, UiText::BeeperDone) => "--- Beeper OK ---",
        (Locale::ArEg, UiText::BeeperDone) => "--- Beeper OK ---",
        (Locale::RuRu, UiText::ChronosHexNoChip) => "[HEX TIME] CMOS чип не отвечает (0xFF)",
        (Locale::EnUs, UiText::ChronosHexNoChip) => "[HEX TIME] CMOS chip does not respond (0xFF)",
        (Locale::ArEg, UiText::ChronosHexNoChip) => "[HEX TIME] شريحة CMOS لا تستجيب (0xFF)",
        (Locale::RuRu, UiText::ChronosHexDeadBattery) => {
            "[HEX TIME] Батарейка CMOS мертва - данные ненадежны"
        }
        (Locale::EnUs, UiText::ChronosHexDeadBattery) => {
            "[HEX TIME] CMOS battery is dead - data unreliable"
        }
        (Locale::ArEg, UiText::ChronosHexDeadBattery) => {
            "[HEX TIME] بطارية CMOS ميتة - البيانات غير موثوقة"
        }
        (Locale::RuRu, UiText::ChronosTimeNoChip) => {
            "[ВРЕМЯ] CMOS чип не отвечает - время недоступно"
        }
        (Locale::EnUs, UiText::ChronosTimeNoChip) => {
            "[TIME] CMOS chip does not respond - time unavailable"
        }
        (Locale::ArEg, UiText::ChronosTimeNoChip) => "[الوقت] شريحة CMOS لا تستجيب - الوقت غير متاح",
        (Locale::RuRu, UiText::ChronosTimeDeadBattery) => {
            "[ВРЕМЯ] Батарейка CMOS мертва - время недостоверно"
        }
        (Locale::EnUs, UiText::ChronosTimeDeadBattery) => {
            "[TIME] CMOS battery is dead - time is unreliable"
        }
        (Locale::ArEg, UiText::ChronosTimeDeadBattery) => {
            "[الوقت] بطارية CMOS ميتة - الوقت غير موثوق"
        }
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
        (Locale::RuRu, UiText::ValidatorPs2Ok) => "[PS/2] Контроллер: ОК - устройство обнаружено",
        (Locale::EnUs, UiText::ValidatorPs2Ok) => "[PS/2] Controller: OK - device detected",
        (Locale::ArEg, UiText::ValidatorPs2Ok) => "[PS/2] المتحكم: OK - تم اكتشافه",
        (Locale::RuRu, UiText::ValidatorPs2NoResp) => {
            "[PS/2] Контроллер: нет ответа (USB-only режим?)"
        }
        (Locale::EnUs, UiText::ValidatorPs2NoResp) => {
            "[PS/2] Controller: no response (USB-only mode?)"
        }
        (Locale::ArEg, UiText::ValidatorPs2NoResp) => "[PS/2] المتحكم: لا استجابة (وضع USB فقط؟)",
        (Locale::RuRu, UiText::ValidatorCmosDead) => "[CMOS] ЧИП НЕ ОТВЕЧАЕТ (0xFF) - RTC мертв",
        (Locale::EnUs, UiText::ValidatorCmosDead) => "[CMOS] CHIP NO RESPONSE (0xFF) - RTC dead",
        (Locale::ArEg, UiText::ValidatorCmosDead) => "[CMOS] لا استجابة من الشريحة (0xFF) - RTC ميت",
        (Locale::RuRu, UiText::ValidatorCmosStatusFmt) => {
            "[CMOS] Чип: ОК (StatA=0x{:02X}) | Батарейка: {} | RTC: {}"
        }
        (Locale::EnUs, UiText::ValidatorCmosStatusFmt) => {
            "[CMOS] Chip: OK (StatA=0x{:02X}) | Battery: {} | RTC: {}"
        }
        (Locale::ArEg, UiText::ValidatorCmosStatusFmt) => {
            "[CMOS] Chip: OK (StatA=0x{:02X}) | Battery: {} | RTC: {}"
        }
        (Locale::RuRu, UiText::ValidatorWarnFmt) => "[!] {}",
        (Locale::EnUs, UiText::ValidatorWarnFmt) => "[!] {}",
        (Locale::ArEg, UiText::ValidatorWarnFmt) => "[!] {}",
        (Locale::RuRu, UiText::ValidatorErrFmt) => "[ERR] {} - возврат в shell",
        (Locale::EnUs, UiText::ValidatorErrFmt) => "[ERR] {} - returning to shell",
        (Locale::ArEg, UiText::ValidatorErrFmt) => "[ERR] {} - عودة الى shell",
        (Locale::RuRu, UiText::ValidatorFatalFmt) => {
            "[FATAL] {} - система остановлена, см. serial.log"
        }
        (Locale::EnUs, UiText::ValidatorFatalFmt) => "[FATAL] {} - system halted, see serial.log",
        (Locale::ArEg, UiText::ValidatorFatalFmt) => {
            "[FATAL] {} - تم ايقاف النظام، راجع serial.log"
        }
        (Locale::RuRu, UiText::RtcNoChip) => "Время: недоступно (чип RTC мертв)",
        (Locale::EnUs, UiText::RtcNoChip) => "Time: unavailable (RTC chip is dead)",
        (Locale::ArEg, UiText::RtcNoChip) => "الوقت: غير متاح (شريحة RTC ميتة)",
        (Locale::RuRu, UiText::RtcDeadBattery) => "Батарейка CMOS: СДОХЛА! Время недостоверно.",
        (Locale::EnUs, UiText::RtcDeadBattery) => "CMOS battery: DEAD! Time is unreliable.",
        (Locale::ArEg, UiText::RtcDeadBattery) => "بطارية CMOS: ميتة! الوقت غير موثوق.",
        (Locale::RuRu, UiText::RtcUipStuck) => "RTC: не готов (UIP завис) - время пропущено",
        (Locale::EnUs, UiText::RtcUipStuck) => "RTC: not ready (UIP stuck) - skipping time read",
        (Locale::ArEg, UiText::RtcUipStuck) => "RTC: غير جاهز (UIP عالق) - تم تجاوز القراءة",
        (Locale::RuRu, UiText::RtcLineFmt) => {
            "Время: {:02}:{:02}:{:02}  {:02}.{:02}.20{:02} | UTC{}"
        }
        (Locale::EnUs, UiText::RtcLineFmt) => {
            "Time: {:02}:{:02}:{:02}  {:02}.{:02}.20{:02} | UTC{}"
        }
        (Locale::ArEg, UiText::RtcLineFmt) => {
            "Time: {:02}:{:02}:{:02}  {:02}.{:02}.20{:02} | UTC{}"
        }
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
        (Locale::RuRu, UiText::VoodooTooManyAnswers) => {
            "[ОБРЫВ СВЯЗИ] Слишком много ответов. Сброс."
        }
        (Locale::EnUs, UiText::VoodooTooManyAnswers) => "[LINK BREAK] Too many answers. Reset.",
        (Locale::ArEg, UiText::VoodooTooManyAnswers) => "[انقطاع] عدد كبير من الاجابات. اعادة ضبط.",
        (Locale::RuRu, UiText::VoodooEntropyCollapse) => {
            "КОЛЛАПС ЭНТРОПИИ: система не сходится, сброс!"
        }
        (Locale::EnUs, UiText::VoodooEntropyCollapse) => {
            "ENTROPY COLLAPSE: system diverges, reset!"
        }
        (Locale::ArEg, UiText::VoodooEntropyCollapse) => {
            "انهيار الانتروبيا: النظام لا يتقارب، اعادة ضبط!"
        }
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
        Locale::RuRu => {
            crate::locale::print_localized_fmt(0x0C, format_args!("  rand[{}] = ОШИБКА", i))
        }
        Locale::EnUs => {
            crate::locale::print_localized_fmt(0x0C, format_args!("  rand[{}] = ERROR", i))
        }
        Locale::ArEg => {
            crate::locale::print_localized_fmt(0x0C, format_args!("  rand[{}] = خطأ", i))
        }
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
        Locale::RuRu => crate::locale::print_localized_fmt(
            0x0E,
            format_args!("Энтропия '{}' = {}.{}", label, whole, frac),
        ),
        Locale::EnUs | Locale::ArEg => crate::locale::print_localized_fmt(
            0x0E,
            format_args!("Entropy '{}' = {}.{}", label, whole, frac),
        ),
    }
}

pub fn print_validator_cmos(status_a: u8, bat: &str, rtc: &str) {
    match crate::locale::get_locale() {
        Locale::RuRu => {
            // alloc a stack string via fmt then pass to print_boot_status
            crate::locale::print_boot_status_fmt(format_args!(
                "[CMOS] Чип: ОК (StatA=0x{:02X}) | Батарейка: {} | RTC: {}",
                status_a, bat, rtc
            ));
        }
        Locale::EnUs | Locale::ArEg => {
            crate::locale::print_boot_status_fmt(format_args!(
                "[CMOS] Chip: OK (StatA=0x{:02X}) | Battery: {} | RTC: {}",
                status_a, bat, rtc
            ));
        }
    }
}
pub fn print_validator_warn(msg: &str) {
    crate::locale::print_localized_fmt(0x0E, format_args!("[!] {}", msg));
}
pub fn print_validator_err(msg: &str) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(
            0x0C,
            format_args!("[ERR] {} - возврат в shell", msg),
        ),
        Locale::EnUs => crate::locale::print_localized_fmt(
            0x0C,
            format_args!("[ERR] {} - returning to shell", msg),
        ),
        Locale::ArEg => {
            crate::locale::print_localized_fmt(0x0C, format_args!("[ERR] {} - عودة الى shell", msg))
        }
    }
}
pub fn print_validator_fatal(msg: &str) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(
            0x0C,
            format_args!("[FATAL] {} - перезагрузка...", msg),
        ),
        Locale::EnUs => {
            crate::locale::print_localized_fmt(0x0C, format_args!("[FATAL] {} - rebooting...", msg))
        }
        Locale::ArEg => crate::locale::print_localized_fmt(
            0x0C,
            format_args!("[FATAL] {} - اعادة تشغيل...", msg),
        ),
    }
}

pub fn print_rtc_line(h: u8, m: u8, s: u8, d: u8, mo: u8, y: u16) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(
            0x0E,
            format_args!(
                "Время (МСК): {:02}:{:02}:{:02}  Дата: {:02}.{:02}.{}",
                h, m, s, d, mo, y
            ),
        ),
        Locale::EnUs | Locale::ArEg => crate::locale::print_localized_fmt(
            0x0E,
            format_args!(
                "Time (UTC+3): {:02}:{:02}:{:02}  Date: {:02}.{:02}.{}",
                h, m, s, d, mo, y
            ),
        ),
    }
}
pub fn print_rtc_throttle(temp: u32, margin: u32) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(
            0x0C,
            format_args!("CPU: ТРОТТЛИНГ! ~{}°C (до TjMax: {}°C)", temp, margin),
        ),
        Locale::EnUs | Locale::ArEg => crate::locale::print_localized_fmt(
            0x0C,
            format_args!("CPU: THROTTLING! ~{}C (to TjMax: {}C)", temp, margin),
        ),
    }
}
pub fn print_rtc_temp(temp: u32, margin: u32) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(
            0x0E,
            format_args!("CPU: ~{}°C | До троттлинга: {}°C", temp, margin),
        ),
        Locale::EnUs => crate::locale::print_localized_fmt(
            0x0E,
            format_args!("CPU: ~{}C | Margin to throttle: {}C", temp, margin),
        ),
        Locale::ArEg => crate::locale::print_localized_fmt(
            0x0E,
            format_args!("CPU: ~{}C | الهامش قبل الخنق: {}C", temp, margin),
        ),
    }
}

pub fn print_chronos_angle(deg_whole: i64, deg_frac: i64) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(
            0x0E,
            format_args!("  Угол: {}.{:02}*", deg_whole, deg_frac),
        ),
        Locale::EnUs => crate::locale::print_localized_fmt(
            0x0E,
            format_args!("  Angle: {}.{:02}*", deg_whole, deg_frac),
        ),
        Locale::ArEg => crate::locale::print_localized_fmt(
            0x0E,
            format_args!("  زاوية: {}.{:02}*", deg_whole, deg_frac),
        ),
    }
}
pub fn print_chronos_roots(sin_r: &str, cos_r: &str) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(
            0x0E,
            format_args!("  Корни: sin~{}, cos~{}", sin_r, cos_r),
        ),
        Locale::EnUs => crate::locale::print_localized_fmt(
            0x0E,
            format_args!("  Roots: sin~{}, cos~{}", sin_r, cos_r),
        ),
        Locale::ArEg => crate::locale::print_localized_fmt(
            0x0E,
            format_args!("  الجذور: sin~{}, cos~{}", sin_r, cos_r),
        ),
    }
}

pub fn print_voodoo_mode(mode: &str) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(
            0x0E,
            format_args!("Режим клеточного автомата: {}", mode),
        ),
        Locale::EnUs => crate::locale::print_localized_fmt(
            0x0E,
            format_args!("Cellular automaton mode: {}", mode),
        ),
        Locale::ArEg => {
            crate::locale::print_localized_fmt(0x0E, format_args!("وضع الخلية الالية: {}", mode))
        }
    }
}
pub fn print_voodoo_step(step: usize, p0: f32, p1: f32, p2: f32, p3: f32, p4: f32) {
    match crate::locale::get_locale() {
        Locale::RuRu => crate::locale::print_localized_fmt(
            0x0E,
            format_args!(
                "Шаг {:>2}: [{:.3}, {:.3}, {:.3}, {:.3}, {:.3}]",
                step, p0, p1, p2, p3, p4
            ),
        ),
        Locale::EnUs | Locale::ArEg => crate::locale::print_localized_fmt(
            0x0E,
            format_args!(
                "Step {:>2}: [{:.3}, {:.3}, {:.3}, {:.3}, {:.3}]",
                step, p0, p1, p2, p3, p4
            ),
        ),
    }
}

// ============================================================
// JACKAL messages (used by apps/jackal/shell.rs)
// ============================================================

pub fn print_jackal_header(label: &str, size: u64) {
    crate::locale::print_localized_fmt(
        0x0E,
        format_args!("=== JACKAL: {} ({} bytes) ===", label, size),
    );
}

pub fn print_jackal_footer() {
    crate::locale::print_localized_line("=== END JACKAL REPORT ===", 0x08);
}

pub fn print_jackal_magic(offset: usize, name: &str, kind_short: &str) {
    crate::locale::print_localized_fmt(
        0x0A,
        format_args!("[MAGIC] {} @0x{:02X} -> {}", name, offset, kind_short),
    );
}

pub fn print_jackal_no_magic() {
    crate::locale::print_localized_line("[MAGIC] none", 0x08);
}

pub fn print_jackal_entropy(h_int: u64, h_frac: u64) {
    crate::locale::print_localized_fmt(
        0x07,
        format_args!("[ENTROPY] {}.{:03} bit/byte", h_int, h_frac),
    );
}

pub fn print_jackal_histogram_stats(unique: u32, printable_bp: u32, zero_bp: u32) {
    crate::locale::print_localized_fmt(
        0x07,
        format_args!(
            "[HIST] unique={} printable={}.{}% zero={}.{}%",
            unique,
            printable_bp / 10,
            printable_bp % 10,
            zero_bp / 10,
            zero_bp % 10,
        ),
    );
}

pub fn print_jackal_profile_summary(blocks: u32, h_min: u16, h_max: u16, transitions: u32) {
    crate::locale::print_localized_fmt(
        0x07,
        format_args!(
            "[PROFILE] blocks={} h_min={} h_max={} transitions={}",
            blocks, h_min, h_max, transitions
        ),
    );
}

pub fn print_jackal_sparkline(line: &[u8]) {
    if let Ok(s) = core::str::from_utf8(line) {
        crate::locale::print_localized_fmt(0x0B, format_args!("[PROFILE] {}", s));
    } else {
        crate::locale::print_localized_line("[PROFILE] <binary>", 0x08);
    }
}

pub fn print_jackal_autocorr_summary(period: u32, pct_int: u32, pct_frac: u32) {
    crate::locale::print_localized_fmt(
        0x07,
        format_args!(
            "[AUTOCORR] best period={} match={}.{}%",
            period, pct_int, pct_frac
        ),
    );
}

pub fn print_jackal_autocorr_row(period: u32, pct_int: u32, pct_frac: u32) {
    crate::locale::print_localized_fmt(
        0x08,
        format_args!("[AUTOCORR] p={:>3} -> {}.{}%", period, pct_int, pct_frac),
    );
}

pub fn print_jackal_classification(kind_name: &str, confidence_milli: u32) {
    crate::locale::print_localized_fmt(
        0x0F,
        format_args!(
            "[CLASS] {} (confidence {}.{}%)",
            kind_name,
            confidence_milli / 10,
            confidence_milli % 10,
        ),
    );
}

pub fn print_jackal_voodoo_priors(priors_milli: &[u32; 6]) {
    let labels = ["TEXT", "EXEC", "COMP", "RAND", "STRC", "MEDIA"];
    crate::locale::print_localized_line("[VDOO] priors:", 0x0D);
    for i in 0..6 {
        crate::locale::print_localized_fmt(
            0x08,
            format_args!("[VDOO] {}: 0.{:03}", labels[i], priors_milli[i]),
        );
    }
}
