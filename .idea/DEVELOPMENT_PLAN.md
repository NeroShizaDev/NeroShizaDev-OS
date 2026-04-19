# 🏗️ NeroShiza OS — ПЛАН РАЗВИТИЯ

> "Файлов не существует. Папок не существует. И кнопок тоже. Есть обращения в порт."
> "Мы не программируем. Мы совершаем обряды кибер шаманзма ии с магическими числами и магическими кодами."
> — Директор И.Б.И.П.

**Дата создания:** апрель 2026  
**Текущая версия ядра:** 0.3 (bootloader 0.11.x, Rust nightly, x86_64)
**Разрабочики:** NeroShizaDev (единственный разработчик и дизайнер)
**Статус:** Активная разработка

---

## 📦 ВЕСОВЫЕ ХАРАКТЕРИСТИКИ ЯДРА

| Компонент | Размер |
|---|---|
| **Сборочная директория (инструментарий разработчика)** | **~1,49 ГБ** (1,485,571,511 байт, `target/`) |
| **Ядро (ELF, debug)** | **~7,04 МБ** (7,039,440 байт / 6,71 MiB) |
| **Загрузочный BIOS-образ (финальный)** | **~9,93 МБ** (9,929,728 байт / 9,47 MiB) |
| **Исходники (.rs)** | **~913 КБ** (913,334 байта, 66 файлов) |
| **UCD таблицы (Unicode)** | **~2,40 МБ** (2,402,332 байта, 3 файла) |
| **Покрытие Unicode** | **299,382 символа** (Unicode 17.0) |

> Таблица обновлена по текущим артефактам после реструктуризации: BIOS-образ уже около 9,93 МБ, а debug ELF — около 7,04 МБ.

---

## 📋 ТЕКУЩЕЕ СОСТОЯНИЕ КОДОВОЙ БАЗЫ

### ✅ Полностью реализовано

| Модуль | Файл | Что работает |
|--------|------|-------------|
| **Загрузка** | `core/main.rs` | 7-фазная инициализация, физическая память, VGA identity mapping |
| **CPU** | `core/gdt.rs` | GDT + TSS (IST стек для double fault 20KB) |
| **Прерывания** | `core/interrupts.rs` | IDT (breakpoint, page fault, double fault, timer, keyboard) |
| **Память** | `core/memory.rs` | OffsetPageTable, BootInfoFrameAllocator, VGA region mapping |
| **Serial** | `core/serial.rs` | UART 16550 COM1, макросы serial_print!/serial_println! |
| **PS/2** | `core/ps2.rs` | has_scancode(), read_scancode(), wait_vblank(), InputOwner |
| **Валидатор** | `core/validator.rs` | probe_port(), probe_speaker/ps2/serial/vga/cmos/battery/rtc |
| **VGA буфер** | `vga/vga_buffer.rs` | Writer, скроллинг, scrollback 1000 строк (160KB), print!/println! |
| **VGA железо** | `vga/vga_hw.rs` | Mode 3 reset, Plane 2 доступ, VBE disable, 16-цветная палитра, **pub restore_text_mode() — 4 шага Ритуала** |
| **VGA Unicode** | `vga/vga_unicode.rs` | ASCII 0-127, Кириллица 128-191, LRU Glyph Cache 64 слота, `codepoint_to_vga_byte()` |
| **Шрифты** | `fonts/` | VGA_FONT_8X16 (ASCII), CYRILLIC_GLYPHS (64 символа), locale.rs |
| **Сообщения** | `core/kernel_messages.rs` | 20 KernelEvent × 3 локали × 2 режима (Technical/Lore) |
| **Паник-экран** | `fonts/locale.rs` | Фиолетовая рамка CP437 (╔═╗║╚═╝), лор-сообщения |
| **Shell** | `apps/shell/shell.rs` | ShellState, буфер ввода, clipboard, history 32, QWERTY↔ЙЦУКЕН |
| **Менгер** | `apps/menger/mod.rs` | Ray marching 3D, fixed-point, 10 уровней яркости, x87 sin/cos |
| **Бипер** | `apps/beeper/mod.rs` | PIT 8254 Ch.2, 16-нотная гексатоническая шкала, ±2 октавы |
| **Хронос** | `apps/chronos/mod.rs` | Psychotown календарь, HEX-время, тригонометрические часы |
| **RNG** | `apps/rng/mod.rs` | RDRAND с 10 повторами, random_range(), LCG fallback |
| **RTC** | `apps/rtc/mod.rs` | DateTime, Москва UTC+3, термальный сенсор, батарейка CMOS |
| **FPU** | `apps/fpu/mod.rs` | init CR0, fninit, log2, sqrt, Shannon entropy |
| **Voodoo** | `core/voodoo_engine.rs` | Байесовский оракул, 5 персонажей, клеточные автоматы, RDRAND-шум |
| **Doom Fire** | `apps/doom/` | Mode 13h, fire palette 37 уровней, плазма, watchdog RDTSC |
| **WAD** | `apps/doom/wad.rs` | Парсер WAD (IWAD/PWAD), LumpInfo, find/read lumps |
| **Лого** | `apps/shell/logo.rs` | 12-строчный ASCII-арт, `#[no_mangle]` |
| **APPS Launcher** | `apps/launcher.rs` | TUI-меню 11 пунктов (↑↓ Enter Esc), рамка CP437, InputGuard |
| **ActivityManager** | `apps/activity.rs` | ActivityStack, AppIntent, lifecycle: on_start/pause/resume/destroy, **11 AppKind** |

### ⚠️ Частично реализовано

| Что | Статус |
|-----|--------|
| Mode 90×30 | Регистры пишутся, QEMU не переключает — disabled |
| Arabic shaping | Глифы 28 букв есть (isolated), контекстных форм нет (начальная/средняя/конечная) |
| RTL wrap | Рендеринг RTL работает, wrap на правой границе — TODO |
| Voodoo интерактив | `demo_cellular_automaton()` подключён к `voodoo` и к APPS, интерактивный Акинатор (ввод да/нет) — TODO |
| Port Firewall | `safe_outb()/safe_inb()` реализованы, `probe_port()` защищён, прямые `Port::new()` в коде ещё не заменены |

### ✅ Недавно реализовано (Апр 2026)

| Что | Файл | Дата |
|-----|------|------|
| `whoami` — 5 резидентов через RDRAND | `apps/shell/shell.rs`, `fonts/unicode.rs` | Апр 2026 |
| `manifest` — NERO(0x09)/SHIZA(0x0D) | `apps/shell/shell.rs` | Апр 2026 |
| `entropy` — Shannon H через x87 | `apps/shell/shell.rs` | Апр 2026 |
| `rng` — RDRAND hex + кубик d6 | `apps/shell/shell.rs` | Апр 2026 |
| `voodoo` — cellular automaton | `apps/shell/shell.rs` | Апр 2026 |
| `Ctrl+L` — очистка экрана | `apps/shell/shell.rs` | Апр 2026 |
| Fuzzy matching Хэмминг (порог 4) | `fonts/unicode.rs` → `closest_intent()` | Апр 2026 |
| Port Firewall `safe_outb/inb` | `core/port_firewall.rs` | Апр 2026 |
| MODULE_ABANDON в `probe_port()` | `core/validator.rs` | Апр 2026 |
| `BumpAllocator` идемпотентность + underflow fix | `apps/doom/stubs.rs` | Апр 2026 |
| Порты 0x604/0x92/0x80 в вайтлист | `core/port_firewall.rs` | Апр 2026 |
| **Ритуал Восстановления Mode 3 (4 шага)** | `vga/vga_hw.rs` → `pub restore_text_mode()` | Апр 2026 |
| **`doom::on_destroy()` — lifecycle cleanup** | `apps/doom/mod.rs` | Апр 2026 |
| **AppKind::Shell** | `apps/activity.rs`, `apps/launcher.rs` | Shell как пункт APPS-меню; on_start/resume чистит экран и рисует `> `; dispatch_update → Pop (возврат в hlt_loop) | Апр 2026 |
| **DF handler → serial trace dump** | `core/interrupts.rs` | Апр 2026 |
| **moduls/ + demo/ → apps/** (7 модулей) | `apps/{menger,chronos,rng,rtc,beeper,fpu}/mod.rs` | Апр 2026 |
| **voodoo_math → core/voodoo_engine.rs** + алиас | `core/voodoo_engine.rs`, `core/lib.rs` | Апр 2026 |
| **shell/ → apps/shell/** | `apps/shell/shell.rs`, `apps/shell/logo.rs` | Апр 2026 |
| **APPS меню расширено до 11 пунктов** | `apps/launcher.rs`, `apps/activity.rs` | Апр 2026 |
| **AppKind: +7 вариантов** (Menger, Voodoo, Chronos, Rtc, Rng, Beeper, Fpu) | `apps/activity.rs` | Апр 2026 |

### ❌ Не начато

| Что | Описание |
|-----|----------|
| GGUF парсер | Загрузка нейросетевых весов на bare-metal |
| Нейро-компрессия | Байесовское предсказание байтов → арифметическое кодирование x87 |
| Семантический File Carving | Поиск magic bytes + эвристический парсинг бинарных дампов |
| Полный Doom | Сектора, линии, сущности, BSP-дерево |
| Египетские иероглифы | U+13000–U+1342F в Dynamic Glyph Cache |
| Шумерская клинопись | U+12000–U+123FF в Dynamic Glyph Cache |
| Мышь PS/2 | Драйвер мыши |
| Voodoo интерактивный Акинатор | Ввод да/нет через shell → Байесовский вывод |
| NeroShizaScript VM | x87 FPU как стековая VM, опкоды NHS |
| NHS Loader | Загрузчик .nhs пакетов без heap |

---

## 🗺️ ДОРОЖНАЯ КАРТА

### BUGFIX: СТАБИЛИЗАЦИЯ APPS LAYER (СРОЧНО)

**Цель:** привести `apps/` к одной модели выполнения: `ActivityManager` управляет стеком, а приложения не ломают lifecycle и не рисуют shell напрямую.

#### B0.1 — Исправить выход из APPS в Shell
- [ ] Убрать псевдо-activity `AppKind::Shell` как отдельный `Push`
- [ ] Пункт `Shell` в `apps/launcher.rs` должен завершать `ActivityManager`, а не возвращать обратно в Launcher
- [ ] После выхода из APPS: очистка экрана, `locale_badge`, возврат владения вводом в shell через `InputGuard::drop()`

#### B0.2 — Починить lifecycle и двойной запуск Games
- [ ] Убрать двойной запуск `apps::games::run()` из `on_start()/on_resume()` и `dispatch_update()`
- [ ] Зафиксировать единый контракт: lifecycle только подготавливает экран и состояние, а запуск app-loop идёт в одном месте
- [ ] Проверить аналогичный контракт для `Doom`, `Jackal`, `Menger`, `Locale`

#### B0.3 — Запретить прямые переходы apps → shell
- [ ] Убрать `crate::print!("> ")` из leaf-apps (`menger`, будущие demo-модули)
- [ ] Правило слоя: `apps/` никогда не рисует shell prompt и не вызывает shell API напрямую
- [ ] Возврат к shell только через `ActivityIntent::Pop` и завершение `run_activity_manager()`

#### B0.4 — Починить ввод PS/2 в приложениях
- [ ] Запретить `read_scancode()` без предварительного `has_scancode()`
- [ ] Проверить `menger`, `games/common_hw.rs`, `jackal`, `launcher`, `locale_switcher`, `doom/input.rs`
- [ ] Для всех app-циклов: игнорировать key-up, не читать сырой порт 0x60 вне готовности OBF

#### B0.5 — Убрать опасные тупики в APPS
- [ ] Пересмотреть `HALT SYSTEM` в `apps/games/launcher.rs`: либо удалить, либо вынести в явный debug-only режим
- [ ] Не допускать вечных циклов в UI-приложениях, которые обходят `ActivityManager`
- [ ] Любой выход из app должен либо вернуть `Pop`, либо выполнить контролируемый reboot/shutdown

#### B0.6 — Довести архитектуру ActivityStack до фактической реализации
- [ ] Либо реализовать `NO_HISTORY`, `SINGLE_TOP`, `CLEAR_TOP`, `IntentExtras`, либо убрать их из публичной архитектуры до реального внедрения
- [ ] Убрать расхождение между документацией и реальным поведением `apps/activity.rs`
- [ ] После bugfix: обновить описание `ActivityManager` в этом плане по фактическому коду, без галлюцинаций

**Критерий готовности bugfix-фазы:**
- [ ] `Apps -> Shell` выходит стабильно за один шаг
- [ ] `Games` не запускается дважды
- [ ] Ни одно приложение в `apps/` не печатает shell prompt напрямую
- [ ] Во всех app-циклах ввод читается только через безопасный шаблон `has_scancode() -> read_scancode()`
- [ ] Поведение lifecycle совпадает с описанием в `DEVELOPMENT_PLAN.md`

### Фаза 0: APPS LAUNCHER — TUI-меню с курсором (✅ ГОТОВО)

**Цель:** Отдельный прикладной слой, полностью изолированный от ядра

#### 0.1 — TUI-меню с подсветкой (вместо ввода номеров)
- [x] Курсор на стрелках ↑↓, выбор — Enter, выход — Esc
- [x] Подсветка: выбранный пункт — белый фон + чёрный текст (0x70)
- [x] Стрелка-указатель ► перед активным пунктом (CP437 0x10)
- [x] Рамка ╔═╗║╚═╝ вокруг меню (CP437 box-drawing)
- [x] Прямая запись в 0xB8000 — без блокировки WRITER

#### 0.2 — Структура Apps
- [x] `apps/launcher.rs` — точка входа, TUI-меню, `update(depth) -> AppIntent`
- [x] `apps/doom/` — Doom Fire, WAD, Mode 13h
- [x] `apps/games/` — 5-in-1 игры
- [x] `apps/jackal/` — анализатор и архиватор
- [x] `apps/activity.rs` — ActivityStack, lifecycle (on_start/pause/resume/destroy)
- [x] `InputGuard` — захват PS/2 на время работы ActivityManager, возврат через Drop
- [x] Shell как пункт меню → `AppKind::Shell` (реализовано: Pop → возврат в hlt_loop)

#### 0.3 — Главный цикл ActivityManager (описание архитектуры)

ActivityManager реализован как **кооперативный планировщик**:

```rust
// apps/activity.rs
pub fn run_activity_manager() {
    let _guard = InputGuard::new();
    let mut stack = ActivityStack::new();
    stack.push(AppKind::Launcher, ...);
    dispatch_lifecycle(AppKind::Launcher, Lifecycle::Start);

    loop {
        let depth = stack.depth();
        let top = stack.top_kind().unwrap();
        let intent = dispatch_update(top, depth);   // каждый app сам отдаёт управление

        match intent {
            Continue => hlt(),
            Pop => { destroy(); pop(); resume_previous(); }
            Push(next) => { pause_current(); push(next); start_new(); }
            Replace(next) => { destroy_current(); push(next); start_new(); }
        }
    }
}
Ключевая особенность: приложения не вытесняются — они сами возвращают AppIntent через свою функцию update(). Это позволяет им владеть экраном и вводом без гонок данных.

0.4 — Дальнейшие улучшения меню
Анимированный логотип над меню (ASCII-art NeroShiza)

Превью-описание выбранного пункта (нижняя панель)

Звуковой тик при переключении пунктов (PC Speaker, beeper.rs)

Версия и время в строке статуса

Фаза 1: КОНСОЛЬ И SHELL (Ближайшая)
Цель: Полноценный интерактивный терминал

1.1 — Shell REPL (замкнуть цикл ввода-вывода)
Полный dispatch команд: help, clear, status, reboot, exit

Команды демо: menger, beep, time, rng, voodoo, doom

Команды локали: locale ru, locale en, locale ar, mode lore, mode tech

Команда whoami — рандомный персонаж из Voodoo-матрицы

Команда manifest — философия NERO & SHIZA

Команда entropy <строка> — вычисление энтропии Шеннона

Unknown command handler с лор-сообщением

1.2 — Навигация и редактирование
Стрелки ←→ — перемещение курсора в строке ввода

Home/End — начало/конец строки

Стрелки ↑↓ — навигация по истории команд

Delete — удаление символа справа

Ctrl+C — отмена текущего ввода

Ctrl+L — очистка экрана (аналог clear)

1.3 — Выделение и буфер обмена
Shift+←→ — выделение текста

Ctrl+A — выделить всё

Ctrl+C (при выделении) — копировать

Ctrl+X — вырезать

Ctrl+V — вставить

Визуальная подсветка выделения (жёлтый на чёрном = 0xE0)

1.4 — Горячие клавиши
F1-F12 — макросы (Alt+Fn запись/воспроизведение)

1.5 — Scrollback
PageUp/PageDown — прокрутка scrollback-буфера (1000 строк)

Визуальный индикатор на строке 24 при просмотре

Escape — выход из scrollback

Фаза 2: МНОГОЯЗЫЧНОСТЬ (Основа)
Цель: Мультиязычная ОС с поддержкой RTL и разных систем счисления

2.1 — Арабские цифры
map_digit_char() — замена 0-9 на ٠-٩ по локали (locale.rs)

Применить к адресам памяти в panic-экране

Применить к времени в chronos

Применить к выводу RNG

2.2 — RTL рендеринг
write_codepoints_rtl_direct() — вывод строки справа налево (locale.rs)

Идентификация RTL строк по LocaleSpec.direction

Корректный wrap на правой границе экрана

Тесты: арабские сообщения в panic-экране

2.3 — Arabic Shaping (контекстные формы)
Таблица: изолированная / начальная / средняя / конечная форма

shape_arabic(input: &[u32], output: &mut [u32]) — выбор формы

Интеграция в рендер-пайплайн перед выводом в VGA

Минимальный набор: 28 основных арабских букв × 4 формы

2.4 — Динамический Glyph Cache (VGA Plane 2)
LRU Glyph Cache реализован в vga/vga_unicode.rs (через static mut массивы)

codepoint_to_vga_byte(cp: u32) -> Option<u8> — единая точка диспетчеризации (устраняет дублирование из vga_buffer и shell)

Поддержка широких символов (16×16): два соседних слота

Египетские иероглифы (U+13000–U+1342F)

Шумерская клинопись (U+12000–U+123FF)

2.5 — Рендер-пайплайн
text
KernelEvent
  → локализованный шаблон (kernel_messages.rs)
  → подстановка параметров (адрес, код ошибки)
  → замена цифр по NumeralSystem
  → Arabic shaping (если RTL)
  → направление текста (LTR/RTL)
  → Glyph Cache lookup (codepoint_to_vga_byte)
  → запись в VGA 0xB8000
Фаза 3: INTENT ENGINE (Мультиязычный парсер команд)
Цель: Команды на любом языке → единый Intent

3.1 — UTF-32 Intent Vectors
lookup_intent(buffer) — поиск по [u32] буферу (fonts/unicode.rs)

Таблица интентов с 80+ синонимами на RU/EN/AR/грузинском/китайском/японском/персидском

Fuzzy matching: расстояние Хэмминга между IntentVector'ами

Fallback: Intent::Unknown для нераспознанных команд

3.2 — GIGA_DICT (расширяемый словарь)
Статический словарь GIGA_DICT: &[([u32; 16], Intent)], 80+ записей

Поиск O(n) по совпадению + оптимизация по первому кодпоинту

Позднее: хэш-таблица или trie

Фаза NHS: ФОРМАТ .NHS — Самоустанавливающиеся App-пакеты (Концепт + Реализация)
APK для bare-metal. Без heap. Без ОС. Только железо и магия байтов.

Что это
.nhs — двойной формат:

Hybrid System — самоустанавливающийся бинарный пакет (app bundle)

NeroShizaScript — встроенный скриптовый язык с x87 FPU как виртуальной машиной

text
.nhs файл
  ├── [MANIFEST]  имя, автор, версия, флаги
  ├── [CODE]      скомпилированный Rust/asm (бинарный app)
  └── [SCR]       NeroShizaScript байткод (x87 VM скрипт)
Один и тот же файл может быть и приложением и скриптом одновременно — лончер запускает то, что есть.

Структура файла .nhs
text
┌──────────────────────────────────────────┐  offset 0
│  MAGIC:  4E 48 53 1A  ("NHS\x1A")        │  4 bytes   ← как NES: \x1A = Ctrl+Z, убивает тексторедакторы
│  VERSION: u16 major + u16 minor           │  4 bytes
│  FLAGS: u32  (bits: needs_fpu|needs_ps2)  │  4 bytes
│  ENTRY_POINT: u32  (offset от начала)     │  4 bytes
│  CODE_OFFSET: u32 + CODE_SIZE: u32        │  8 bytes
│  RODATA_OFFSET: u32 + RODATA_SIZE: u32    │  8 bytes
│  DATA_OFFSET: u32 + DATA_SIZE: u32        │  8 bytes
│  BSS_SIZE: u32  (нули при загрузке)       │  4 bytes
│  LUMP_DIR_OFFSET: u32 + LUMP_COUNT: u32   │  8 bytes   ← WAD-style lump directory
│  MANIFEST_OFFSET: u32 + MANIFEST_SIZE:u32 │  8 bytes
│  CRC32: u32  (checksum всего файла)       │  4 bytes
├──────────────────────────────────────────┤  offset 64
│  CODE SECTION  (position-independent)    │
├──────────────────────────────────────────┤
│  RODATA SECTION  (строки, таблицы)       │
├──────────────────────────────────────────┤
│  LUMP DIRECTORY  (WAD-style)             │
│  Каждый lump: 16 bytes                   │
│    [4] type_tag ("TEX","SND","CFG","GFX") │
│    [4] offset (от начала файла)          │
│    [4] size                              │
│    [4] name_hash (CRC32 имени)           │
├──────────────────────────────────────────┤
│  MANIFEST  (фиксированные 128 bytes)     │
│    [32] app_name  (UTF-8, zero-padded)   │
│    [32] author    (UTF-8)                │
│    [4]  version   (major.minor.patch.0)  │
│    [4]  min_os_version                   │
│    [4]  required_flags                   │
│    [4]  stack_size (сколько стека нужно) │
│    [48] reserved                         │
├──────────────────────────────────────────┤
│  LUMP DATA (TEX, SND, CFG, GFX блоки)   │
└──────────────────────────────────────────┘  EOF
Как грузится NHS-пакет (без heap!)
text
NhsLoader::load(blob: &[u8]) -> Result<AppHandle, NhsError>
  1. Проверяем magic[0..4] == "NHS\x1A"
  2. Читаем header (64 bytes) как &NhsHeader (repr(C), packed)
  3. CRC32 всего blob (table-lookup, 256-entry таблица в .rodata)
  4. Находим lump directory по lump_dir_offset
  5. Итерируемся по lump'ам (фиксированный счётчик, без vec!)
  6. Находим MANIFEST лумп → читаем имя, версию, флаги
  7. Проверяем required_flags против HW возможностей ядра
  8. entry_fn = blob.as_ptr().add(header.entry_point) as fn() -> AppResult
  9. Возвращаем AppHandle { run: entry_fn, name, stack_size }

ActivityManager::launch(handle: AppHandle)
  1. Записываем в ActivityStack
  2. Вызываем handle.run() → возвращает AppResult
  3. Обрабатываем Exit/Push/Replace как обычно
Встраивание в ядро (Фаза 1: статически)
Пока нет FS — пакеты встраиваем через include_bytes!():

rust
// apps/registry.rs
static GAME_BYTE_DODGE: &[u8] = include_bytes!("../packages/byte_dodge.nhs");
static DOOM_FIRE:        &[u8] = include_bytes!("../packages/doom_fire.nhs");
static JACKAL:           &[u8] = include_bytes!("../packages/jackal.nhs");
Лончер читает APP_REGISTRY, грузит манифест каждого → показывает имя/автора/тип в меню (динамически, вместо текущего хардкода ENTRIES).

Динамическая загрузка (Фаза 2: из RAM-диска)
Когда появится RAM-disk или TFTP загрузка — NHS Loader читает из буфера, не из include_bytes!.

Сравнение источников вдохновения
Идея	Откуда взяли	Зачем
Magic "NHS\x1A"	NES ROM ("NES\x1A")	Убивает текст-редакторы, идентифицируем мгновенно
64-byte fixed header	ELF (52/64 bytes)	Читаем одним ptr::read_volatile, без итераций
Lump directory	WAD (Doom)	Случайный доступ к ресурсам без heap-аллокатора
MANIFEST section	Android APK manifest	Имя, автор, версия, требования к железу
CRC32 table-lookup	ZIP/PNG	Верификация без temp buffer, без malloc
entry_point offset	WebAssembly export	Позволяет грузить по любому адресу
required_flags	NES mapper bits	Ядро знает, поддерживает ли железо этот app
stack_size в манифесте	SNES SRAM size field	ActivityManager резервирует стек заранее
NeroShizaScript — язык программирования на x87 FPU
"Мы не интерпретируем. Мы скармливаем байткод регистровому стеку сопроцессора."

Ключевое открытие: x87 FPU — это уже готовая стековая виртуальная машина:

8 регистров ST(0)–ST(7) = стек глубиной 8

fld = push, fstp = pop

fadd, fmul, fsub, fdiv — арифметика на стеке

fsin, fcos, fsqrt, flog2, f2xm1 — встроенные функции — native hardware!

80-битная точность (Extended Precision) — лучше, чем Python float64

NeroShizaScript — стековый язык поверх x87:

nhs
# Пример: вычислить sin(π/4) * √2 и напечатать
PUSH 3.14159265
PUSH 4.0
DIV          # x87: fdiv → ST(0) = π/4
SIN          # x87: fsin → ST(0) = sin(π/4) = 0.7071...
SQRT2        # x87: fld1 + fsqrt → push √2
MUL          # x87: fmul → ST(0) = 1.0
PRINT        # вывод ST(0) через vga_buffer

# Вероятностный оператор (через RDRAND)
PUSH 0.3
P_IF         # выполнить следующий блок с вероятностью 30%
  PRINT_STR "МАТРИЦА НЕСТАБИЛЬНА"
P_END

# Энтропия Шеннона встроена
PUSH_STR "NeroShiza"
ENTROPY      # Shannon entropy → ST(0)
PRINT
Таблица опкодов NeroShizaScript → x87:

Опкод NHS	x87 инструкция	Описание
PUSH f	fld [mem]	Положить константу на стек
POP	fstp [mem]	Снять со стека
ADD	fadd	ST(1) = ST(0) + ST(1), pop
SUB	fsub	ST(1) = ST(1) - ST(0), pop
MUL	fmul	ST(1) = ST(0) × ST(1), pop
DIV	fdiv	ST(1) = ST(1) / ST(0), pop
SIN	fsin	ST(0) = sin(ST(0))
COS	fcos	ST(0) = cos(ST(0))
SQRT	fsqrt	ST(0) = √ST(0)
LOG2	fld1; fyl2x	ST(0) = log₂(ST(0))
POW2	f2xm1	ST(0) = 2^ST(0) - 1
ABS	fabs	ST(0) =	ST(0)	
NEG	fchs	ST(0) = -ST(0)
DUP	fld ST(0)	Дублировать вершину стека
SWAP	fxch ST(1)	Поменять ST(0) ↔ ST(1)
PI	fldpi	Push π
E	fldl2e; ...	Push e
SQRT2	fldl2t; ...	Push √2
RND	RDRAND	Push случайное f64 ∈ [0,1)
ENTROPY	Shannon alg	Pop строку → push Shannon H
P_IF	RDRAND + cmp	Вероятностный if (Voodoo-режим)
PRINT	VGA out	Напечатать ST(0)
PRINT_STR	VGA out	Напечатать строку из пула
HALT	hlt	Завершить скрипт
Специальные операторы (уникальные для NHS):

nhs
# Байесовский оракул (встроен через VoodooEngine)
VOODOO_ASK "Это хорошее решение?"
# → ST(0) = вероятность "да" по матрице персонажей

# Клеточный автомат
CELLULAR 3      # 3 итерации Rule 110 на экране
DISPLAY_MATRIX  # отрисовать x87 стек как 8×1 гистограмму

# Цикл
PUSH 10.0       # счётчик
LOOP_START      # while ST(0) > 0
  PUSH 1.0
  SUB           # счётчик -= 1
  DUP
  PRINT
LOOP_END
NHS Script VM в Rust:

rust
// apps/nhs/vm.rs
pub struct NhsVm {
    // x87 стек: используем реальный FPU, не эмулируем!
    str_pool: [u8; 1024],   // пул строк скрипта (без heap)
    str_count: usize,
    pc: usize,              // program counter в байткоде
}

impl NhsVm {
    pub fn run(&mut self, bytecode: &[u8]) -> AppIntent {
        // decode opcode → dispatch → x87 inline asm
        // HALT → AppIntent::Pop
    }
}
Интеграция с ActivityManager:
Скрипт запускается как AppKind::Script(id), где id — индекс в SCRIPT_REGISTRY.
dispatch_update() для Script вызывает NhsVm::run(&script_bytecode).
Скрипт работает в цикле: каждый опкод — одна итерация, HALT возвращает ActivityIntent::Pop.

Формат байткода SCR-лампа:

text
[1 byte]  opcode
[varies]  операнды:
  PUSH:     [8 bytes] f64 little-endian
  PUSH_STR: [1 byte] длина + [N bytes] UTF-8
  P_IF:     [4 bytes] offset к P_END (если пропускаем блок)
  LOOP_END: [4 bytes] offset назад к LOOP_START
Почему это работает на bare-metal без OS:

x87 инициализируется в fpu::init() в фазе 2 загрузки

Строки в статическом пуле — не нужен heap

RDRAND — уже есть в moduls/rng.rs

Shannon entropy — уже есть в demo/fpu.rs

VoodooEngine — уже есть в moduls/voodoo_math.rs

Всё уже написано — нужен только интерпретатор!

Задачи для реализации NHS
Формат и загрузчик:

apps/nhs/mod.rs — pub mod header, manifest, lump, loader, crc32, vm

apps/nhs/header.rs — #[repr(C, packed)] struct NhsHeader (64 bytes)

apps/nhs/manifest.rs — #[repr(C, packed)] struct NhsManifest (128 bytes)

apps/nhs/lump.rs — #[repr(C, packed)] struct LumpEntry (16 bytes) + LumpType enum

apps/nhs/loader.rs — NhsLoader::load(&[u8]) -> Result<AppHandle, NhsError>

apps/nhs/crc32.rs — CRC32 table-lookup (без heap, 1KB таблица в .rodata)

NeroShizaScript VM:

apps/nhs/vm.rs — NhsVm::run(bytecode) — диспетчер опкодов

apps/nhs/opcodes.rs — enum Opcode + таблица декодирования

apps/nhs/x87_ops.rs — unsafe inline asm для каждого x87 опкода

apps/nhs/str_pool.rs — статический пул строк [u8; 1024] (без heap)

Вероятностный P_IF через rng::rdrand() + порог

Интеграция VOODOO_ASK → voodoo_math::bayesian_step()

Интеграция ENTROPY → fpu::shannon_entropy()

Экосистема:

apps/registry.rs — статический реестр встроенных NHS пакетов (include_bytes!)

tools/nhs_builder.py — Python: берёт байты + манифест → собирает .nhs с CRC32

Лончер читает манифесты из реестра → показывает имя+автора+тип(app/script) в TUI меню

Интеграция с ActivityManager: launch_nhs(blob) → определяет тип → push на стек

Фаза 4: PORT FIREWALL И БЕЗОПАСНОСТЬ
Цель: Защита от случайных обращений к опасным портам

4.1 — safe_outb()
Белый список разрешённых портов:

VGA: 0x3C0-0x3DF, 0x01CE-0x01CF

Speaker: 0x42, 0x43, 0x61

PS/2: 0x60, 0x64

Serial: 0x3F8-0x3FF

PIC: 0x20, 0x21, 0xA0, 0xA1

CMOS: 0x70, 0x71

PIT: 0x40-0x43

Запретить: 0x1F0-0x1F7 (HDD), 0x170-0x177 (IDE), 0xCF8-0xCFF (PCI)

При обращении к запрещённому порту → АБСОЛЮТНЫЙ АБАНДОН

Логирование в serial все обращения к портам (debug mode)

4.2 — Анти-Сансара (детектор зацикливания)
Счётчик итераций в Voodoo Engine (connection_drops)

Порог: > 10 одинаковых вопросов → connection_drop + reset_matrix

Статистика обрывов: connection_drops (уже есть)

Лор-сообщение: "ОБРЫВ СВЯЗИ С ОРАКУЛОМ. МАТРИЦА ОБНУЛЕНА."

АБАНДОН СИСТЕМА (3 уровня критических сбоев)
"Три уровня капитуляции. Выбирай с умом."

Уровень	Имя	Триггер	Реакция
FATAL_ABANDON	Абсолютный Абандон	Kernel panic, double fault, page fault	loop { hlt() } + перезагрузка матрицы (warm reset)
MODULE_ABANDON	Модульный Абандон	Драйвер не ответил, порт недоступен, таймаут	Выгрузить модуль, продолжить работу
TASK_ABANDON	Задачный Абандон	Команда не распознана, буфер переполнен	Вернуть управление в shell, напечатать лор-сообщение
rust
// core/abandon.rs (концепт)
pub enum AbandonLevel {
    Fatal,   // → panic_handler → hlt loop
    Module,  // → unload_module() → continue
    Task,    // → shell prompt → lore message
}
Реализация:

FATAL_ABANDON — уже реализован как #[panic_handler] с "АБСОЛЮТНЫЙ АБАНДОН" + hlt

MODULE_ABANDON — добавить в validator.rs для probe_* функций

TASK_ABANDON — уже частично: shell выводит лор при неизвестной команде

WATCHDOG BAF-КОДЫ (Код укуса сторожевого таймера)
"Каждый укус — это диагноз."

Два вида кодов: KernelBiteCode (ядро, ring 0) и AppBiteCode (приложения, ring 3+).

KernelBiteCode (0xBAF_...)
Константа	Значение	Смысл
BITE_CLASSIC	0xBAFBAF	Стандартный таймаут — hlt-loop, CPU завис
BITE_ORACLE_LOOP	0xBAF001	Voodoo Engine завис в бесконечном вопросе
BITE_DEMONIC	0xBAF666	Демонический сбой — паника без причины
BITE_DEAD	0xBAFDEAD	Ядро мертво — Double Fault + Abort
BITE_BASE_415	0xBAFBACE415	База 415 — катастрофический сбой памяти
BITE_ELITE	0xBAF1337	LEET-укус — статус элитного сбоя
BITE_INFINITY	0xBAF88	Бесконечный цикл обнаружен (88 = ∞)
BITE_EMERGENCY	0xBAF911	Экстренный сигнал — НЕ ПАНИКОВАТЬ
AppBiteCode (0xBAD_... / 0xBED_...)
Константа	Значение	Смысл
BITE_BAD_BOY	0xBADBAF	Плохой мальчик — app вышел за пределы
BITE_POISON	0xBADF00D	Отравленные данные — BAD FOOD
BITE_SLEEPY	0xBEDBAF	Приложение "уснуло" (BED = кровать)
BITE_BEDA_BEDA	0xBEDABEDA	Беда-беда — двойной сбой в app
rust
// core/watchdog.rs (концепт)
pub const BITE_CLASSIC:     u64 = 0xBAFBAF;
pub const BITE_ORACLE_LOOP: u64 = 0xBAF001;
pub const BITE_DEMONIC:     u64 = 0xBAF666;
pub const BITE_DEAD:        u64 = 0xBAFDEAD;
pub const BITE_BASE_415:    u64 = 0xBAFBACE415;
pub const BITE_ELITE:       u64 = 0xBAF1337;
pub const BITE_INFINITY:    u64 = 0xBAF88;
pub const BITE_EMERGENCY:   u64 = 0xBAF911;
pub const BITE_BAD_BOY:     u64 = 0xBADBAF;
pub const BITE_POISON:      u64 = 0xBADF00D;
pub const BITE_SLEEPY:      u64 = 0xBEDBAF;
pub const BITE_BEDA_BEDA:   u64 = 0xBEDABEDA;

pub fn watchdog_bite(code: u64) -> ! {
    // Записать код в serial + VGA
    // Уйти в hlt-loop
}
Задачи:

core/watchdog.rs — реализовать BAF-коды как константы + watchdog_bite(code)

Heartbeat на PIT-таймере: каждые N тиков ядро должно сбросить счётчик

Если счётчик не сброшен → watchdog_bite(BITE_CLASSIC)

Интеграция с Port Firewall: обращение к запрещённому порту → BITE_DEMONIC

Фаза 5: VOODOO ENGINE 2.0 (Развитие ИИ)
Цель: Расширение Байесовского движка

5.1 — Расширение базы знаний
Больше персонажей (настраиваемый CHAR_COUNT)

Больше вопросов (настраиваемый Q_COUNT)

Динамическая KNOWLEDGE_BASE (загрузка из бинарного блоба)

5.2 — Многоконтекстный движок
Режимы: Акинатор, Детектив файлов, Предсказатель байтов

Переключение контекста KNOWLEDGE_BASE по режиму

Общий интерфейс: predict(context, observation) -> (answer, probability)

5.3 — Обучение на кракозябрах (File Carving)
Матрица переходов байтов: [[f32; 256]; 256]

train_on_dump(data) — обновление весов по паре (current_byte, next_byte)

Предсказание типа файла по magic bytes:

text
PK\x03\x04 → ZIP/XLSX/DOCX
\x7FELF → ELF binary
MZ → DOS/PE executable
GGUF → нейросетевые веса
\xFF\xD8\xFF → JPEG
\x89PNG → PNG
Замер энтропии Шеннона блока → "сжатые данные" vs "текст"

Фаза 6: НЕЙРОКОМПРЕССИЯ (Экспериментальная)
Цель: Арифметическое кодирование на 80-битных x87 регистрах

6.1 — Предсказательная модель
Контекстная модель Маркова: вероятность P(byte | prev_N_bytes)

Контекст: 1-3 байта (order-1, order-2, order-3)

Смешивание контекстов через Байесовское взвешивание

6.2 — Арифметический кодировщик
80-битная арифметика x87 (Extended Precision)

Отрезок [low, high] сужается по каждому предсказанию

Результат: одно дробное число = весь файл

Декодер: обратный процесс с той же моделью

6.3 — Семантическое сжатие (концепт)
Распознавание типа файла по magic bytes

Замена известных структур на Intent-коды

Генерация недостающего контента по шаблону

Фаза 7: GGUF И ГОЛОЕ ЖЕЛЕЗО (Долгосрочная)
Цель: Загрузка и инференс нейросетевых моделей без ОС

7.1 — Парсер GGUF
Чтение заголовка: magic "GGUF", version, tensor_count, kv_count

Парсинг метаданных (key-value пары)

Извлечение тензоров (f16/f32/q4_0/q4_1)

Bump-аллокатор для хранения весов в RAM

7.2 — Матричные операции на x87
Скалярное произведение (dot product) через fmul/fadd

Активация: ReLU (сравнение + условный переход)

Softmax: fldl2e + f2xm1 для exp(), нормализация суммы

Batch-размер = 1 (online inference)

7.3 — Генерация текста
Подача токена → forward pass → logits

Сэмплирование: «Порог Шизы» (Top-P) + «Нейро-тиски» (Top-K)

Вывод символов в VGA через Glyph Cache

Ограничение: модели < 100MB (RAM constraint)

Фаза 8: DOOM ПОЛНОЦЕННЫЙ (Факультативно)
Цель: Играбельный уровень на bare-metal

8.1 — BSP рендеринг
Парсинг LINEDEFS, SIDEDEFS, SECTORS, SEGS, SSECTORS, NODES

BSP-обход дерева (front-to-back)

Рендеринг стен (column-based projection)

High/low textures, floor/ceiling flats

8.2 — Игровая логика
Движение игрока (forward/back/strafe/turn)

Коллизия со стенами

Двери и лифты (sector specials)

Вещи (things): декорации, враги, предметы

8.3 — Звук
PC Speaker: простые эффекты (выстрел, дверь, смерть)

Приоритетная очередь звуков

🏛️ АРХИТЕКТУРНЫЕ ПРИНЦИПЫ
6 Аксиом NeroShiza OS (троичный гибрид)
Настоящая мощь ПК скрыта под абстракциями — мы обращаемся к портам, регистрам и MSR напрямую

Команды на любом языке — Intent Engine понимает русский, английский, арабский

Забытые datasheets воскресают через ИИ — VGA, PIT, CMOS, x87 используются на полную

Статистика и ИИ на уровне железа — Байес + Шеннон + x87 FPU = VoodooEngine

Свой язык с вероятностями — стековый язык с комбинаторикой, x87 как VM

Три режима — одно железо — x86-64 (ядро) + x86-32 (Mode 13h) + x87 FPU (VM) работают в унисон

Три слоя системы (Hybrid OS = ядро + менеджер активностей + приложения)
Слой	Где живёт	Что содержит
Ядро	core/, vga/, moduls/rtc.rs	Прерывания, память, порты, регистры — не знает об apps вообще
Терминал	shell/	Shell REPL (interrupt-driven), Intent Engine, команды
Приложения	apps/	Activity Manager + все apps — полностью изолированы от ядра
Android-style ActivityStack
text
Kernel boots
  └─► shell (interrupt-driven, hlt_loop)
        └─► user types "apps"
              └─► apps::activity::run_activity_manager()
                    └─► ActivityStack: [Launcher]
                          ├─► Enter → Push(Games)   → [Launcher, Games]
                          │     └─► Esc → Exit       → [Launcher]
                          ├─► Enter → Push(Doom)    → [Launcher, Doom]
                          │     └─► Esc → Exit       → [Launcher]
                          └─► Esc → Exit             → stack empty → shell
AppIntent — что возвращает каждый app:

Continue — оставаться активным (не менять стек)

Pop — закрыть текущий activity (Esc / кнопка назад)

Push(AppKind) — запустить новый activity поверх текущего

Replace(AppKind) — заменить текущий без записи в back-stack

ActivityStack — [Option<Slot>; 16] статический массив, без heap. Глубина до 16 уровней.

Жизненный цикл Activity:

on_start() — при первом запуске (Push)

on_pause() — когда поверх открывают другой activity

on_resume() — когда верхний activity закрыт и текущий снова активен (должен перерисовать экран)

on_destroy() — при удалении из стека (Pop или Replace)

Структура каталогов (актуальная)
text
core/           — ЯДРО (чистое, без UI): загрузка, прерывания, память, GDT, PS/2, serial
vga/            — видеоподсистема: буфер, регистры, Unicode-маппинг
fonts/          — шрифты, локаль, Unicode-таблицы
moduls/         — системные модули: chronos, menger, rng, rtc, voodoo_math
demo/           — демонстрации: beeper, fpu

apps/           — ПРИКЛАДНОЙ СЛОЙ (отдельно от ядра)
  activity.rs   — ActivityManager, ActivityStack, AppIntent, lifecycle dispatch
  launcher.rs   — TUI-меню с подсветкой курсора (↑↓ Enter Esc), точка входа
  mod.rs        — pub-экспорты (games, jackal, doom re-export)
  doom/         — Doom Fire / Mode 13h / WAD parser
    mod.rs, chaos.rs, input.rs, vga_graphics.rs, wad.rs, watchdog.rs
  games/        — 5-in-1 игры
    mod.rs, launcher.rs, common_hw.rs
    byte_dodge.rs, clicker_game.rs, debil_card_game.rs
    match_grab_and_leave.rs, absurd_loading_screen.rs, module_demo.rs
  jackal/       — Анализатор + архиватор (WAD/DOCX/ZIP)
    mod.rs, shell.rs, analyzer.rs, encoder.rs
    tools/      mod.rs, pipeline.rs, archive.rs, extractor.rs, validate.rs, replace.rs
    scripts/    pipeline.py, pack.py, unpack.py, validate.py ...

shell/          — Интерактивный терминал (interrupt-driven, Intent Engine)
  shell.rs, logo.rs, mod.rs

tests/          — интеграционные тесты
Архитектурное разделение
Слой	Модули	Что содержит
Ядро	core/, vga/, fonts/, moduls/, demo/	Железо, прерывания, память, шрифты
Приложения	apps/	ActivityManager, TUI-меню, игры, doom, jackal — изолированы от ядра
Терминал	shell/	Shell REPL, Intent Engine, команды
Правило: apps/ никогда не вызывает shell/ напрямую. Shell запускает Apps через crate::apps::activity::run_activity_manager(). Apps возвращают управление через AppIntent::Pop.

📊 ПРИОРИТЕТНОСТЬ (уточнённая)
text
[КРИТИЧНО]  Интеграция shell → ActivityManager (замкнуть цикл)
[ВЫСОКО]    Дописать lifecycle для Games/Doom/Jackal (on_pause/on_resume)
[ВЫСОКО]    Фаза NHS: Loader (header, crc32, loader) — фундамент для .nhs
[ВЫСОКО]    Фаза NHS: NeroShizaScript VM (первые 5 опкодов: PUSH, ADD, PRINT)
[СРЕДНЕ]    Рефакторинг R1 (уже сделан?) — codepoint_to_vga_byte()
[СРЕДНЕ]    Фаза 2 — Многоязычность (USP проекта)
[СРЕДНЕ]    Фаза 3 — Intent Engine (команды на всех языках)
[СРЕДНЕ]    Фаза 4 — Port Firewall (безопасность VoodooEngine)
[НИЗКО]     Фаза 5 — Voodoo 2.0 (расширение ИИ)
[ЭКСПЕР.]   Фаза 6 — Нейрокомпрессия (R&D)
[ЭКСПЕР.]   Фаза 7 — GGUF инференс (R&D)
[ФАКУЛЬТ.]  Фаза 8 — Doom полный (когда всё остальное готово)
🎭 ФИРМЕННЫЕ СИСТЕМНЫЕ СООБЩЕНИЯ
Panic
RU Lore: АБСОЛЮТНЫЙ АБАНДОН. ЯДРО ПОКИНУЛО ЭТУ ВЕТКУ РЕАЛЬНОСТИ.

EN Lore: ABSOLUTE ABANDON. THE KERNEL HAS LEFT THIS BRANCH OF REALITY.

AR Lore: .تخلٍّ مطلق. غادرت النواة هذا الفرع من الواقع

Watchdog Bite
RU Lore: ВРЕМЯ ВЫШЛО. WATCHDOG УКУСИЛ ЯДРО. ПЕРЕЗАГРУЗКА МАТРИЦЫ...

EN Lore: TIME IS UP. THE WATCHDOG BIT THE KERNEL. REBOOTING THE MATRIX...

Код укуса: см. BAF-коды выше (BITE_CLASSIC = 0xBAFBAF по умолчанию)

Page Fault
RU Lore: СОЗНАНИЕ КОСНУЛОСЬ НЕСУЩЕСТВУЮЩЕЙ СТРАНИЦЫ ПАМЯТИ.

EN Lore: CONSCIOUSNESS TOUCHED A PAGE THAT DOES NOT EXIST.

Double Fault
RU Lore: ДВОЙНОЙ СБОЙ. МАТРИЦА НЕ УСПЕЛА ПОНЯТЬ ПЕРВУЮ ОШИБКУ.

EN Lore: DOUBLE FAULT. THE MATRIX DID NOT FINISH UNDERSTANDING THE FIRST ERROR.

Out of Memory
RU Lore: ПАМЯТЬ ИСЧЕРПАНА. ПУСТОТА ПОГЛОТИЛА ПОСЛЕДНИЙ АЛЛОКАТОР.

EN Lore: OUT OF MEMORY. THE VOID HAS CONSUMED THE LAST ALLOCATOR.

Invalid Opcode
RU Lore: ПРОЦЕССОР УСЛЫШАЛ ЗАКЛИНАНИЕ, КОТОРОГО НЕ БЫЛО В ЕГО СВИТКАХ.

EN Lore: THE PROCESSOR HEARD AN INSTRUCTION NOT WRITTEN IN ITS SCROLLS.

Keyboard Controller Timeout
RU Lore: КЛАВИАТУРНЫЙ СТРАЖ НЕ ОТВЕТИЛ. СИГНАЛ ПОТЕРЯЛСЯ В ПРОВОДАХ.

EN Lore: THE KEYBOARD WARDEN DID NOT ANSWER. SIGNAL LOST IN THE WIRES.

whoami (команда — 5 резидентов)
Бросает кубик через x87 FPU / RDRAND, выдаёт одного из резидентов:

text
Текущий сеанс: Петрович (Король жижи)
Текущий сеанс: Группа К.А.Ф.И.Д.Р.А. (Анализ аномалий)
Текущий сеанс: Банановый Турист
Текущий сеанс: Dr. Bred
Текущий сеанс: ДедушкаВКрутую
Реализация: rng::random_range(0, 5) → индекс в CHARACTERS из voodoo_math.rs

manifest (команда — философия NERO & SHIZA)
Выводит философский текст лейбла NeroShiza Records.

Слово NERO — неоново-синим 0x09 (Bright Blue on Black)

Слово SHIZA — кислотно-малиновым 0x0D (Bright Magenta on Black)

Заклинание готовности (Intent::Status)
Иероглифы 𓋹𓊽𓋾𓎟𓉐 из Egyptian Hieroglyphs (U+1300x) — команда-статус системы.
Через Dynamic Glyph Cache в VGA Plane 2 выводится в загрузочном экране.

🔨 РЕФАКТОРИНГ — ПЛАН (v0.2 → v0.3)
Составлено на основе реального кода, без галлюцинаций.

Главная проблема (решена в R1)
Логика «codepoint → VGA byte» была написана три раза в vga_buffer.rs, shell.rs и vga_unicode.rs.
Решение: единая функция codepoint_to_vga_byte() в vga/vga_unicode.rs.

rust
// vga/vga_unicode.rs — единственное место
pub fn codepoint_to_vga_byte(cp: u32) -> Option<u8> {
    if cp < 128 { return Some(cp as u8); }                      // ASCII
    if let Some(v) = cyrillic_to_vga(cp) { return Some(v); }   // слоты 128-191
    ensure_glyph_cached(cp)                                      // LRU 192-255
}
Теперь vga_buffer.rs и shell.rs вызывают одну функцию вместо трёх веток.

Рефакторинг-фазы
Фаза R1 — создать codepoint_to_vga_byte() в vga_unicode.rs, убрать дублирование

Фаза R2 — перенести font.rs + arabic_font.rs → секции внутри vga_unicode.rs, удалить файлы

Фаза R3 — добавить Writer::write_codepoint(u32) для прямой записи из shell без format!

Фаза R4 — убрать мёртвый код: try_90x30() в vga_hw.rs, неиспользуемые импорты

Что НЕ делаем (и почему)
Предложение	Причина отказа
InputHandler trait	ISR-контекст несовместим с &mut self через Mutex — дедлок
VgaMode enum с конфигами	FontMode ≠ режим VGA, это временное переключение plane
GlyphSystem с Vec<Box<dyn FontSource>>	heap + vtable на bare-metal = overhead без пользы
Убрать locale.rs	Он делает больше (locale state, badge, RTL, panic screen)
Переделать write_str trait	Нужен для println!() макроса
🔧 КОМАНДЫ СБОРКИ
batch
# Сборка ядра
cargo build --bin blog_os

# Создание BIOS-образа
.\run.bat --build-only

# Запуск в QEMU
.\run.bat

# Тесты
cargo test
📚 ССЫЛКИ И ИСТОЧНИКИ
VGA Registers: FreeVGA / OSDev Wiki

x87 FPU: Intel® 64 and IA-32 Software Developer Manual, Vol. 1 Ch. 8

RDRAND/RDSEED: Intel® DRNG Software Implementation Guide

PIT 8254: OSDev Wiki / Intel 82C54 Datasheet

CMOS/RTC: Motorola MC146818A Datasheet

WAD Format: Doom Wiki / id Software specs

GGUF Format: llama.cpp documentation

Dynamic Glyph Caching: ReactOS + OSDev VGA text mode tricks

Arabic Shaping: Unicode Standard Annex #9 (Bidi) + UAX #53

🏛️ И.Б.И.П. — ЛОР И МЕТА-ВСЕЛЕННАЯ
Институт Бреда и Помешательства (И.Б.И.П.)
Юрисдикция: Психотаун
Статус: Официальная операционная система Института
Лейбл: NeroShiza Records — первый мультивселенский лейбл, точка сборки мультивселенной

Два слоя системы:

Слой	Описание
Инженерный	Холодный, строгий: #![no_std], RDRAND, x87 FPU, BAF-коды, порты VGA
Сакральный	Лор, персонажи, тексты: Психотаун, И.Б.И.П., NeroShiza Records
Принцип: "Хаос должен быть в именах и строках, не в UI и структуре кода."
Код — строгий. Лор — безумный.

Психотаунский Календарь
Единица	Значение
1 месяц	100 дней
1 год	1200 дней (12 месяцев × 100 дней)
Стандартное время	HEX-время через chronos.rs
Резиденты мультивселенной (персонажи whoami)
Персонаж	Роль	В коде
Петрович	Король жижи — герой нашего времени	CHARACTERS[0] в voodoo_math.rs
Группа К.А.Ф.И.Д.Р.А.	Изучает аномалии рабочих будней	CHARACTERS[1]
Банановый Турист	Ищет истину	CHARACTERS[2]
Dr. Bred	Выписывает рецепты от скуки	CHARACTERS[3]
ДедушкаВКрутую	Музыкальный след в мультивселенной	CHARACTERS[4]
(Архитектор Хаоса = сам разработчик, не в списке whoami)

Философия NERO & SHIZA
NERO (Нейро): строгая математика x87 FPU, 64-битная логика, борьба с галлюцинациями алгоритмов. Выжимаем чистый сигнал без помех.

SHIZA (Шиза): абсолютная творческая свобода. Психотаунский календарь. Загрузка через египетские иероглифы 𓋹𓊽𓋾𓎟𓉐. Полный отказ от std.

"NeroShiza Records. Мы открываем порталы. Вы просто нажимаете Play."

Реальное железо (цель деплоя)
Загрузчик: GRUB (или собственный bootloader)

Носитель: USB flash drive

Целевое железо: i5-12400F, 32GB RAM

Статус: Сейчас работает в QEMU; задача — запуск на реальном ПК

🔧 ИНЖЕНЕРНЫЕ ПРИНЦИПЫ (ТРОИЧНЫЙ ГИБРИД)
Настоящая мощь ПК скрыта под абстракциями — обращаемся к портам, регистрам и MSR напрямую

Команды на любом языке — Intent Engine: UTF-32 вектор, не строка

Забытые datasheets воскресают через ИИ — VGA 0x3C4, PIT 0x42, x87 fsin/fcos — всё используется

Статистика и ИИ на уровне железа — Байес + Шеннон + x87 FPU = VoodooEngine (Акинатор)

Свой язык с вероятностями — NeroShizaScript: x87 как VM, P_IF через RDRAND

Хаос в именах, порядок в коде — лор безумный, структура строгая

Три режима — один кристалл — x86-64 (ядро) + x86-32 (Mode 13h) + x87 FPU (VM)

Институт Бреда и Помешательства (И.Б.И.П.) — Юрисдикция Психотаун
NeroShiza Records — Лейбл цифрового абсурда