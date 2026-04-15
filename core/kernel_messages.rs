// ============================================================
// KERNEL MESSAGES — NeroShiza Multilingual Event System
// KernelEvent × Locale × MessageMode → &'static str
// ============================================================
// Architecture:
//   Ядро хранит СМЫСЛ (KernelEvent), а не текст.
//   Рендер-пайплайн выбирает нужную (Locale × MessageMode)
//   и выводит через locale::render_event().
//
//   Слои:
//     Technical — сухой инженерный
//     Lore      — фирменный стиль NeroShiza / И.Б.И.П.
// ============================================================

/// Системные события ядра, требующие вывода сообщения
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelEvent {
    Panic,
    Watchdog,
    PageFault,
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
    Lore,      // Лор — фирменный NeroShiza стиль
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
            lore:      "АБСОЛЮТНЫЙ АБАНДОН. ЯДРО ПОКИНУЛО ЭТУ ВЕТКУ РЕАЛЬНОСТИ.",
        },
        (Locale::EnUs, KernelEvent::Panic) => EventText {
            technical: "KERNEL PANIC. EXECUTION HALTED.",
            lore:      "ABSOLUTE ABANDON. THE KERNEL HAS LEFT THIS BRANCH OF REALITY.",
        },
        (Locale::ArEg, KernelEvent::Panic) => EventText {
            technical: "انهيار النواة. متوقف.",
            lore:      "الهجر المطلق. النواة غادرت هذا الفرع من الواقع.",
        },

        // ==================== WATCHDOG ====================

        (Locale::RuRu, KernelEvent::Watchdog) => EventText {
            technical: "ТАЙМАУТ СТОРОЖЕВОГО ТАЙМЕРА. ВЫПОЛНЯЕТСЯ ПЕРЕЗАГРУЗКА.",
            lore:      "ВРЕМЯ ВЫШЛО. WATCHDOG УКУСИЛ ЯДРО. ПЕРЕЗАГРУЗКА МАТРИЦЫ...",
        },
        (Locale::EnUs, KernelEvent::Watchdog) => EventText {
            technical: "WATCHDOG TIMER EXPIRED. SYSTEM RESET INITIATED.",
            lore:      "TIME IS UP. THE WATCHDOG BIT THE KERNEL. REBOOTING THE MATRIX...",
        },
        (Locale::ArEg, KernelEvent::Watchdog) => EventText {
            technical: "انتهت مهلة الحارس. إعادة تشغيل.",
            lore:      "انتهى الوقت. الكلب عض النواة. إعادة تحميل المصفوفة...",
        },

        // ==================== PAGE FAULT ====================

        (Locale::RuRu, KernelEvent::PageFault) => EventText {
            technical: "ОШИБКА СТРАНИЧНОЙ АДРЕСАЦИИ.",
            lore:      "СОЗНАНИЕ КОСНУЛОСЬ НЕСУЩЕСТВУЮЩЕЙ СТРАНИЦЫ ПАМЯТИ.",
        },
        (Locale::EnUs, KernelEvent::PageFault) => EventText {
            technical: "PAGE FAULT.",
            lore:      "CONSCIOUSNESS TOUCHED A PAGE THAT DOES NOT EXIST.",
        },
        (Locale::ArEg, KernelEvent::PageFault) => EventText {
            technical: "خطأ في تحديد الصفحة.",
            lore:      "الوعي لمس صفحة غير موجودة.",
        },

        // ==================== DOUBLE FAULT ====================

        (Locale::RuRu, KernelEvent::DoubleFault) => EventText {
            technical: "ДВОЙНОЙ СБОЙ ПРОЦЕССОРА.",
            lore:      "ДВОЙНОЙ СБОЙ. МАТРИЦА НЕ УСПЕЛА ПОНЯТЬ ПЕРВУЮ ОШИБКУ.",
        },
        (Locale::EnUs, KernelEvent::DoubleFault) => EventText {
            technical: "DOUBLE FAULT.",
            lore:      "DOUBLE FAULT. THE MATRIX DID NOT FINISH UNDERSTANDING THE FIRST ERROR.",
        },
        (Locale::ArEg, KernelEvent::DoubleFault) => EventText {
            technical: "خطأ مزدوج في المعالج.",
            lore:      "خطأ مضاعف. المصفوفة لم تدرك الخطأ الأول.",
        },

        // ==================== OUT OF MEMORY ====================

        (Locale::RuRu, KernelEvent::OutOfMemory) => EventText {
            technical: "ПАМЯТЬ ИСЧЕРПАНА.",
            lore:      "ПАМЯТЬ ИСЧЕРПАНА. ПУСТОТА ПОГЛОТИЛА ПОСЛЕДНИЙ АЛЛОКАТОР.",
        },
        (Locale::EnUs, KernelEvent::OutOfMemory) => EventText {
            technical: "OUT OF MEMORY.",
            lore:      "OUT OF MEMORY. THE VOID HAS CONSUMED THE LAST ALLOCATOR.",
        },
        (Locale::ArEg, KernelEvent::OutOfMemory) => EventText {
            technical: "نفدت الذاكرة.",
            lore:      "نفدت الذاكرة. الفراغ ابتلع المخصص الأخير.",
        },

        // ==================== INVALID OPCODE ====================

        (Locale::RuRu, KernelEvent::InvalidOpcode) => EventText {
            technical: "НЕДОПУСТИМЫЙ КОД ОПЕРАЦИИ.",
            lore:      "ПРОЦЕССОР УСЛЫШАЛ ЗАКЛИНАНИЕ, КОТОРОГО НЕ БЫЛО В ЕГО СВИТКАХ.",
        },
        (Locale::EnUs, KernelEvent::InvalidOpcode) => EventText {
            technical: "INVALID OPCODE.",
            lore:      "THE PROCESSOR HEARD AN INSTRUCTION THAT WAS NOT WRITTEN IN ITS SCROLLS.",
        },
        (Locale::ArEg, KernelEvent::InvalidOpcode) => EventText {
            technical: "رمز عملية غير صالح.",
            lore:      "المعالج سمع تعويذة لم تكن في سجلاته.",
        },

        // ==================== KEYBOARD TIMEOUT ====================

        (Locale::RuRu, KernelEvent::KeyboardTimeout) => EventText {
            technical: "ТАЙМАУТ КОНТРОЛЛЕРА КЛАВИАТУРЫ.",
            lore:      "КЛАВИАТУРНЫЙ СТРАЖ НЕ ОТВЕТИЛ. СИГНАЛ ПОТЕРЯЛСЯ В ПРОВОДАХ.",
        },
        (Locale::EnUs, KernelEvent::KeyboardTimeout) => EventText {
            technical: "KEYBOARD CONTROLLER TIMEOUT.",
            lore:      "THE KEYBOARD WARDEN DID NOT ANSWER. THE SIGNAL WAS LOST IN THE WIRES.",
        },
        (Locale::ArEg, KernelEvent::KeyboardTimeout) => EventText {
            technical: "انتهت مهلة لوحة المفاتيح.",
            lore:      "حارس اللوحة لم يستجب. الإشارة ضاعت في الأسلاك.",
        },

        // ==================== BREAKPOINT ====================

        (Locale::RuRu, KernelEvent::BreakPoint) => EventText {
            technical: "ТОЧКА ПРЕРЫВАНИЯ ДОСТИГНУТА.",
            lore:      "ЯДРО ОСТАНОВИЛОСЬ У ВЕШКИ. РАЗУМ ВОШЁЛ В РЕЖИМ ОТЛАДКИ.",
        },
        (Locale::EnUs, KernelEvent::BreakPoint) => EventText {
            technical: "BREAKPOINT HIT.",
            lore:      "THE KERNEL PAUSED AT THE MARKER. THE MIND ENTERED DEBUG MODE.",
        },
        (Locale::ArEg, KernelEvent::BreakPoint) => EventText {
            technical: "نقطة التوقف وصلت.",
            lore:      "النواة توقفت عند العلامة. العقل دخل وضع التصحيح.",
        },

        // ==================== SHELL EVENTS ====================

        (Locale::RuRu, KernelEvent::ShellConfirmPrompt) => EventText {
            technical: "ПОДТВЕРЖДЕНИЕ: ENTER — ДА, ESC — ОТМЕНА.",
            lore:      "ПЕРЕД ПРЫЖКОМ КРАЙНИЙ ВОПРОС: ЖМЁШЬ ENTER ИЛИ УХОДИШЬ ЧЕРЕЗ ESC?",
        },
        (Locale::EnUs, KernelEvent::ShellConfirmPrompt) => EventText {
            technical: "CONFIRMATION: ENTER = YES, ESC = CANCEL.",
            lore:      "FINAL QUESTION BEFORE THE LEAP: ENTER TO COMMIT, ESC TO ABORT.",
        },
        (Locale::ArEg, KernelEvent::ShellConfirmPrompt) => EventText {
            technical: "تأكيد: ENTER نعم، ESC إلغاء.",
            lore:      "السؤال الأخير قبل القفز: ENTER للتنفيذ أو ESC للتراجع.",
        },

        (Locale::RuRu, KernelEvent::ShellCanceled) => EventText {
            technical: "ОПЕРАЦИЯ ОТМЕНЕНА.",
            lore:      "ОТКАТ ПРИНЯТ. ЛИНИЯ РЕАЛЬНОСТИ СОХРАНЕНА.",
        },
        (Locale::EnUs, KernelEvent::ShellCanceled) => EventText {
            technical: "OPERATION CANCELED.",
            lore:      "ROLLBACK ACCEPTED. THIS REALITY BRANCH REMAINS STABLE.",
        },
        (Locale::ArEg, KernelEvent::ShellCanceled) => EventText {
            technical: "تم إلغاء العملية.",
            lore:      "تم قبول التراجع. فرع الواقع بقي مستقرا.",
        },

        (Locale::RuRu, KernelEvent::ShellShutdown) => EventText {
            technical: "ЗАПРОШЕНО ВЫКЛЮЧЕНИЕ СИСТЕМЫ.",
            lore:      "ЯДРО ЗАКРЫВАЕТ ГЛАЗА. СИСТЕМА УХОДИТ В ТИШИНУ.",
        },
        (Locale::EnUs, KernelEvent::ShellShutdown) => EventText {
            technical: "SYSTEM SHUTDOWN REQUESTED.",
            lore:      "THE KERNEL CLOSES ITS EYES. THE SYSTEM FALLS INTO SILENCE.",
        },
        (Locale::ArEg, KernelEvent::ShellShutdown) => EventText {
            technical: "تم طلب إيقاف النظام.",
            lore:      "النواة تغلق عينيها. النظام يهبط إلى الصمت.",
        },

        (Locale::RuRu, KernelEvent::ShellReboot) => EventText {
            technical: "ЗАПРОШЕНА ПЕРЕЗАГРУЗКА СИСТЕМЫ.",
            lore:      "МАТРИЦА ПЕРЕСОБИРАЕТСЯ. НАЧИНАЕМ ЦИКЛ ЗАНОВО.",
        },
        (Locale::EnUs, KernelEvent::ShellReboot) => EventText {
            technical: "SYSTEM REBOOT REQUESTED.",
            lore:      "THE MATRIX REASSEMBLES ITSELF. A NEW CYCLE BEGINS.",
        },
        (Locale::ArEg, KernelEvent::ShellReboot) => EventText {
            technical: "تم طلب إعادة تشغيل النظام.",
            lore:      "المصفوفة تعيد تشكيل نفسها. تبدأ دورة جديدة.",
        },

        (Locale::RuRu, KernelEvent::ShellLocaleRu) => EventText {
            technical: "ЛОКАЛЬ УСТАНОВЛЕНА: RU.",
            lore:      "ГОЛОС ЯДРА ПЕРЕШЕЛ НА РУССКУЮ ВЕТКУ.",
        },
        (Locale::EnUs, KernelEvent::ShellLocaleRu) => EventText {
            technical: "LOCALE SET: RU.",
            lore:      "THE KERNEL TUNED ITSELF TO THE RUSSIAN BRANCH.",
        },
        (Locale::ArEg, KernelEvent::ShellLocaleRu) => EventText {
            technical: "تم تعيين اللغة: RU.",
            lore:      "النواة ضُبطت على الفرع الروسي.",
        },

        (Locale::RuRu, KernelEvent::ShellLocaleEn) => EventText {
            technical: "ЛОКАЛЬ УСТАНОВЛЕНА: EN.",
            lore:      "ЯДРО ПЕРЕКЛЮЧИЛОСЬ НА АНГЛИЙСКУЮ ВЕТКУ.",
        },
        (Locale::EnUs, KernelEvent::ShellLocaleEn) => EventText {
            technical: "LOCALE SET: EN.",
            lore:      "THE KERNEL SWITCHED TO THE ENGLISH BRANCH.",
        },
        (Locale::ArEg, KernelEvent::ShellLocaleEn) => EventText {
            technical: "تم تعيين اللغة: EN.",
            lore:      "النواة انتقلت إلى الفرع الإنجليزي.",
        },

        (Locale::RuRu, KernelEvent::ShellLocaleAr) => EventText {
            technical: "ЛОКАЛЬ УСТАНОВЛЕНА: AR.",
            lore:      "ЯДРО СМЕНИЛО НАПРАВЛЕНИЕ ПИСЬМА НА ВОСТОК.",
        },
        (Locale::EnUs, KernelEvent::ShellLocaleAr) => EventText {
            technical: "LOCALE SET: AR.",
            lore:      "THE KERNEL TURNED ITS SCRIPT FLOW EASTWARD.",
        },
        (Locale::ArEg, KernelEvent::ShellLocaleAr) => EventText {
            technical: "تم تعيين اللغة: AR.",
            lore:      "النواة حولت اتجاه الكتابة نحو الشرق.",
        },

        (Locale::RuRu, KernelEvent::ShellModeLore) => EventText {
            technical: "РЕЖИМ ВЫВОДА: LORE.",
            lore:      "ФИРМЕННЫЙ ТОН АКТИВИРОВАН. ЯДРО ГОВОРИТ ГОЛОСОМ NEROSHIZA.",
        },
        (Locale::EnUs, KernelEvent::ShellModeLore) => EventText {
            technical: "OUTPUT MODE: LORE.",
            lore:      "SIGNATURE VOICE ENABLED. THE KERNEL SPEAKS IN NEROSHIZA TONE.",
        },
        (Locale::ArEg, KernelEvent::ShellModeLore) => EventText {
            technical: "وضع الإخراج: LORE.",
            lore:      "تم تفعيل النبرة الخاصة. النواة تتكلم بصوت NeroShiza.",
        },

        (Locale::RuRu, KernelEvent::ShellModeTech) => EventText {
            technical: "РЕЖИМ ВЫВОДА: TECHNICAL.",
            lore:      "ПОЭЗИЯ ОТКЛЮЧЕНА. ОСТАЛАСЬ ТОЛЬКО ИНЖЕНЕРНАЯ ПРАВДА.",
        },
        (Locale::EnUs, KernelEvent::ShellModeTech) => EventText {
            technical: "OUTPUT MODE: TECHNICAL.",
            lore:      "POETRY DISABLED. ONLY ENGINEERING TRUTH REMAINS.",
        },
        (Locale::ArEg, KernelEvent::ShellModeTech) => EventText {
            technical: "وضع الإخراج: TECHNICAL.",
            lore:      "تم إيقاف الشعر. بقيت الحقيقة الهندسية فقط.",
        },

        (Locale::RuRu, KernelEvent::ShellDoomStart) => EventText {
            technical: "ЗАПУСК DEMO: DOOM FIRE.",
            lore:      "ПЛАМЯ БЕЗДНЫ ПРОСНУЛОСЬ. ЭКРАН НАЧИНАЕТ ГОРЕТЬ.",
        },
        (Locale::EnUs, KernelEvent::ShellDoomStart) => EventText {
            technical: "STARTING DEMO: DOOM FIRE.",
            lore:      "ABYSSAL FLAME AWAKENS. THE SCREEN BEGINS TO BURN.",
        },
        (Locale::ArEg, KernelEvent::ShellDoomStart) => EventText {
            technical: "تشغيل العرض: DOOM FIRE.",
            lore:      "لهيب الهاوية استيقظ. الشاشة تبدأ بالاحتراق.",
        },

        (Locale::RuRu, KernelEvent::ShellUnknownCommand) => EventText {
            technical: "НЕИЗВЕСТНАЯ КОМАНДА.",
            lore:      "ЯДРО НЕ УЗНАЛО ЭТО ЗАКЛИНАНИЕ.",
        },
        (Locale::EnUs, KernelEvent::ShellUnknownCommand) => EventText {
            technical: "UNKNOWN COMMAND.",
            lore:      "THE KERNEL DID NOT RECOGNIZE THIS INCANTATION.",
        },
        (Locale::ArEg, KernelEvent::ShellUnknownCommand) => EventText {
            technical: "أمر غير معروف.",
            lore:      "النواة لم تتعرف على هذه التعويذة.",
        },
    }
}

/// Короткое ASCII-имя события (для статус-строки и serial log)
pub fn event_short_name(ev: KernelEvent) -> &'static str {
    match ev {
        KernelEvent::Panic           => "PANIC",
        KernelEvent::Watchdog        => "WDT",
        KernelEvent::PageFault       => "PF",
        KernelEvent::DoubleFault     => "DF",
        KernelEvent::OutOfMemory     => "OOM",
        KernelEvent::InvalidOpcode   => "UD",
        KernelEvent::KeyboardTimeout => "KBD_TIMEOUT",
        KernelEvent::BreakPoint      => "BP",
        // Shell events
        KernelEvent::ShellConfirmPrompt => "SHELL_CONFIRM",
        KernelEvent::ShellCanceled     => "SHELL_CANCEL",
        KernelEvent::ShellShutdown     => "SHELL_SHUTDOWN",
        KernelEvent::ShellReboot       => "SHELL_REBOOT",
        KernelEvent::ShellLocaleRu     => "SHELL_LOCALE_RU",
        KernelEvent::ShellLocaleEn     => "SHELL_LOCALE_EN",
        KernelEvent::ShellLocaleAr     => "SHELL_LOCALE_AR",
        KernelEvent::ShellModeLore     => "SHELL_MODE_LORE",
        KernelEvent::ShellModeTech     => "SHELL_MODE_TECH",
        KernelEvent::ShellDoomStart    => "SHELL_DOOM_START",
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
        MessageMode::Lore      => "LORE",
    }
}

