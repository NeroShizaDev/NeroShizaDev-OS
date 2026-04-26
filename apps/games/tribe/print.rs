// print.rs — вывод текста игры через vga_unicode.rs
//
// Вся цепочка:
//   &'static str (UTF-8 кириллица в db.rs)
//     → chars() → char as u32 (codepoint)
//       → vga_unicode::codepoint_to_vga_byte(cp)
//         → write_vga_cell(x, y, byte, color)
//
// Никаких аллокаций. Никаких String. Только стек.

use crate::vga_unicode::print_char;

// ═══════════════════════════════════════════════════════════
// ЦВЕТА (стандартные VGA атрибуты)
// ═══════════════════════════════════════════════════════════

pub const ЦВЕТ_ОБЫЧНЫЙ: u8 = 0x07; // светло-серый на чёрном
pub const ЦВЕТ_ЗАГОЛОВОК: u8 = 0x0F; // ярко-белый на чёрном
pub const ЦВЕТ_УСПЕХ: u8 = 0x0A; // ярко-зелёный
pub const ЦВЕТ_ПРОВАЛ: u8 = 0x0C; // ярко-красный
pub const ЦВЕТ_КРИТ: u8 = 0x0E; // жёлтый
pub const ЦВЕТ_ПОДСКАЗКА: u8 = 0x08; // тёмно-серый
pub const ЦВЕТ_РАМКА: u8 = 0x09; // синий

// ═══════════════════════════════════════════════════════════
// СОСТОЯНИЕ КУРСОРА
// ═══════════════════════════════════════════════════════════

static mut CURSOR_X: usize = 0;
static mut CURSOR_Y: usize = 0;

pub fn cursor_reset() {
    unsafe {
        CURSOR_X = 0;
        CURSOR_Y = 0;
    }
}

pub fn cursor_pos() -> (usize, usize) {
    unsafe { (CURSOR_X, CURSOR_Y) }
}

pub fn cursor_set(x: usize, y: usize) {
    unsafe {
        CURSOR_X = x;
        CURSOR_Y = y;
    }
}

fn cols() -> usize {
    crate::vga_hw::get_columns()
}
fn rows() -> usize {
    crate::vga_hw::get_rows()
}

// ═══════════════════════════════════════════════════════════
// БАЗОВЫЙ ВЫВОД
// ═══════════════════════════════════════════════════════════

/// Вывести один символ с переносом строк и скроллингом
pub fn print_char_adv(c: char, color: u8) {
    unsafe {
        match c {
            '\n' => {
                CURSOR_X = 0;
                CURSOR_Y += 1;
                if CURSOR_Y >= rows() {
                    scroll_up();
                    CURSOR_Y = rows() - 1;
                }
            }
            '\r' => {
                CURSOR_X = 0;
            }
            _ => {
                let consumed = print_char(c as u32, CURSOR_X, CURSOR_Y, color).max(1);

                CURSOR_X += consumed;
                if CURSOR_X >= cols() {
                    CURSOR_X = 0;
                    CURSOR_Y += 1;
                    if CURSOR_Y >= rows() {
                        scroll_up();
                        CURSOR_Y = rows() - 1;
                    }
                }
            }
        }
    }
}

/// Вывести &str на текущей позиции курсора
pub fn print(s: &str, color: u8) {
    for c in s.chars() {
        print_char_adv(c, color);
    }
}

/// Вывести &str + перенос строки
pub fn println(s: &str, color: u8) {
    print(s, color);
    print_char_adv('\n', color);
}

/// Вывести &str в конкретной позиции без изменения курсора
pub fn print_at(s: &str, x: usize, y: usize, color: u8) {
    let mut cx = x;
    for c in s.chars() {
        if cx >= cols() {
            break;
        }
        let consumed = print_char(c as u32, cx, y, color).max(1);
        cx += consumed;
    }
}

/// Вывести число u32 как текст
pub fn print_num(mut n: u32, color: u8) {
    if n == 0 {
        print_char_adv('0', color);
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 10;
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    for &b in &buf[i..] {
        print_char_adv(b as char, color);
    }
}

/// Вывести число i32 (с минусом если отрицательное)
pub fn print_inum(n: i32, color: u8) {
    if n < 0 {
        print_char_adv('-', color);
        print_num((-n) as u32, color);
    } else {
        print_num(n as u32, color);
    }
}

// ═══════════════════════════════════════════════════════════
// СКРОЛЛИНГ
// ═══════════════════════════════════════════════════════════

/// Сдвигаем всё содержимое экрана на 1 строку вверх
fn scroll_up() {
    unsafe {
        let base = 0xB8000 as *mut u16;
        let cols = cols();
        let rows = rows();
        // Копируем строки 1..rows в строки 0..rows-1
        for y in 0..rows - 1 {
            for x in 0..cols {
                let src = (y + 1) * cols + x;
                let dst = y * cols + x;
                let val = core::ptr::read_volatile(base.add(src));
                core::ptr::write_volatile(base.add(dst), val);
            }
        }
        // Очищаем последнюю строку
        let blank: u16 = (ЦВЕТ_ОБЫЧНЫЙ as u16) << 8 | b' ' as u16;
        for x in 0..cols {
            let dst = (rows - 1) * cols + x;
            core::ptr::write_volatile(base.add(dst), blank);
        }
    }
}

/// Очистить весь экран
pub fn clear_screen() {
    unsafe {
        let base = 0xB8000 as *mut u16;
        let blank: u16 = (ЦВЕТ_ОБЫЧНЫЙ as u16) << 8 | b' ' as u16;
        for i in 0..cols() * rows() {
            core::ptr::write_volatile(base.add(i), blank);
        }
        CURSOR_X = 0;
        CURSOR_Y = 0;
    }
}

/// Очистить одну строку
pub fn clear_line(y: usize) {
    unsafe {
        let base = 0xB8000 as *mut u16;
        let blank: u16 = (ЦВЕТ_ОБЫЧНЫЙ as u16) << 8 | b' ' as u16;
        for x in 0..cols() {
            core::ptr::write_volatile(base.add(y * cols() + x), blank);
        }
    }
}

// ═══════════════════════════════════════════════════════════
// УТИЛИТЫ ДЛЯ ИГРЫ
// ═══════════════════════════════════════════════════════════

/// Нарисовать горизонтальную линию из символов '═'
pub fn print_separator(color: u8) {
    let w = cols().saturating_sub(2);
    for _ in 0..w {
        print_char_adv('=', color);
    }
    print_char_adv('\n', color);
}

/// Напечатать результат броска кубиков в стиле игры:
/// "Бросок: 3d6 → 9   Цель: 11 > 9  ✓ Успех"
pub fn print_roll_result(roll: u8, target: u8, outcome_symbol: &str, outcome_text: &str) {
    print("Бросок: 3d6 -> ", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(roll as u32, ЦВЕТ_КРИТ);
    print("   Цель: ", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(target as u32, ЦВЕТ_ОБЫЧНЫЙ);
    print(" ", ЦВЕТ_ОБЫЧНЫЙ);

    // знак > или <
    let sign_color = if roll <= target {
        ЦВЕТ_УСПЕХ
    } else {
        ЦВЕТ_ПРОВАЛ
    };
    print(outcome_symbol, sign_color);
    print(" ", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(roll as u32, sign_color);
    print("  ", ЦВЕТ_ОБЫЧНЫЙ);
    println(outcome_text, sign_color);
}

/// Напечатать текст события с паузой
/// (в реальной ОС — ждём нажатия клавиши через keyboard::read_scancode)
pub fn print_event_text(text: &str) {
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println(text, ЦВЕТ_ОБЫЧНЫЙ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    print("[Любая клавиша]", ЦВЕТ_ПОДСКАЗКА);
}

/// Заголовок игры с племенем и статистикой
pub fn print_game_header(
    name: &str,
    turn: u32,
    population: u32,
    food: i32,
    knowledge: u32,
    ap_left: u8,
    ap_max: u8,
) {
    print_separator(ЦВЕТ_РАМКА);
    print("  Племя: ", ЦВЕТ_ОБЫЧНЫЙ);
    print(name, ЦВЕТ_ЗАГОЛОВОК);
    print("  |  Ход: ", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(turn, ЦВЕТ_ЗАГОЛОВОК);
    println("", ЦВЕТ_ОБЫЧНЫЙ);

    print("  Народ: ", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(population, ЦВЕТ_КРИТ);
    print("  Еда: ", ЦВЕТ_ОБЫЧНЫЙ);
    print_inum(
        food,
        if food < 5 {
            ЦВЕТ_ПРОВАЛ
        } else {
            ЦВЕТ_УСПЕХ
        },
    );
    print("  Знания: ", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(knowledge, ЦВЕТ_ОБЫЧНЫЙ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);

    print("  Приказы: ", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(ap_left as u32, ЦВЕТ_КРИТ);
    print("/", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(ap_max as u32, ЦВЕТ_ОБЫЧНЫЙ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    print_separator(ЦВЕТ_РАМКА);
}

/// Главное меню действий
pub fn print_main_menu(ap_left: u8) {
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println("  Что прикажет вождь?", ЦВЕТ_ЗАГОЛОВОК);
    println("", ЦВЕТ_ОБЫЧНЫЙ);

    let dim = |cost: u8| {
        if ap_left >= cost {
            ЦВЕТ_ОБЫЧНЫЙ
        } else {
            ЦВЕТ_ПОДСКАЗКА
        }
    };

    print("  [1] ОХОТА     ", dim(2));
    println(
        "(2 приказа) — загнать зверя, ловушки. Риск: средний",
        dim(2),
    );

    print("  [2] СОБИРАТЬ  ", dim(1));
    println(
        "(1 приказ)  — ягоды, коренья у лагеря. Риск: низкий",
        dim(1),
    );

    print("  [3] ДУМАТЬ    ", dim(2));
    println("(2 приказа) — шаман, костёр, знания. Риск: низкий", dim(2));

    print("  [4] РАЗВЕДКА  ", dim(1));
    println("(1 приказ)  — осмотреть округу. Риск: средний", dim(1));

    print("  [5] НИЧЕГО    ", ЦВЕТ_ОБЫЧНЫЙ);
    println("(0 приказов) — день отдыха. Мир не отдыхает.", ЦВЕТ_ОБЫЧНЫЙ);

    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println("  [H] История ходов", ЦВЕТ_ПОДСКАЗКА);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
}

/// Подменю действия
pub fn print_action_submenu(action_name: &str, description: &str, cost: u8) {
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    print_separator(ЦВЕТ_РАМКА);
    print("  ", ЦВЕТ_ОБЫЧНЫЙ);
    print(action_name, ЦВЕТ_ЗАГОЛОВОК);
    print("  (", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(cost as u32, ЦВЕТ_КРИТ);
    println(" приказа)", ЦВЕТ_ОБЫЧНЫЙ);
    print("  ", ЦВЕТ_ОБЫЧНЫЙ);
    println(description, ЦВЕТ_ОБЫЧНЫЙ);
    print_separator(ЦВЕТ_РАМКА);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println("  [1] КАК ОБЫЧНО — проверенный способ", ЦВЕТ_ОБЫЧНЫЙ);
    println("  [2] ПО-ТУПОМУ  - ??? (система решит)", ЦВЕТ_КРИТ);
    println("  [3] НАЗАД", ЦВЕТ_ПОДСКАЗКА);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
}

/// Экран открытия Письменности
pub fn print_writing_unlock() {
    clear_screen();
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    print_separator(ЦВЕТ_ЗАГОЛОВОК);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println("  ПЛЕМЯ ИЗОБРЕЛО ПИСЬМЕННОСТЬ.", ЦВЕТ_ЗАГОЛОВОК);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println(
        "  Шаман нацарапал первые знаки на плоском камне.",
        ЦВЕТ_ОБЫЧНЫЙ,
    );
    println(
        "  Теперь знания не умрут вместе со стариками.",
        ЦВЕТ_ОБЫЧНЫЙ,
    );
    println("  Теперь история останется после вас.", ЦВЕТ_ОБЫЧНЫЙ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println("  Впервые с начала игры:", ЦВЕТ_ОБЫЧНЫЙ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println("  [S] СОХРАНИТЬ ПРОГРЕСС", ЦВЕТ_УСПЕХ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    print_separator(ЦВЕТ_ЗАГОЛОВОК);
}

/// Экран конца игры
pub fn print_game_over(tribe_name: &str, turns: u32) {
    clear_screen();
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    print_separator(ЦВЕТ_ПРОВАЛ);
    println("  ПЛЕМЯ ИСЧЕЗЛО.", ЦВЕТ_ПРОВАЛ);
    print_separator(ЦВЕТ_ПРОВАЛ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    print("  Племя ", ЦВЕТ_ОБЫЧНЫЙ);
    print(tribe_name, ЦВЕТ_ЗАГОЛОВОК);
    print(" просуществовало ", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(turns, ЦВЕТ_КРИТ);
    println(" ходов.", ЦВЕТ_ОБЫЧНЫЙ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println("  [Enter] Начало заново...", ЦВЕТ_ПОДСКАЗКА);
}

/// Экран создания племени
pub fn print_tribe_creation(str_: u8, dex: u8, int: u8, hlt: u8, max_ap: u8) {
    clear_screen();
    print_separator(ЦВЕТ_ЗАГОЛОВОК);
    println("  ПЛЕМЯ СОЗДАНО!", ЦВЕТ_ЗАГОЛОВОК);
    print_separator(ЦВЕТ_ЗАГОЛОВОК);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println(
        "  Боги (и сопроцессор x87) определили вашу судьбу:",
        ЦВЕТ_ОБЫЧНЫЙ,
    );
    println("", ЦВЕТ_ОБЫЧНЫЙ);

    print("  СИЛ (Сила)     : ", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(str_ as u32, ЦВЕТ_КРИТ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    print("  ЛОВ (Ловкость) : ", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(dex as u32, ЦВЕТ_КРИТ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    print("  ИНТ (Интеллект): ", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(int as u32, ЦВЕТ_КРИТ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    print("  ЗДР (Здоровье) : ", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(hlt as u32, ЦВЕТ_КРИТ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    print("  Макс. приказов : ", ЦВЕТ_ОБЫЧНЫЙ);
    print_num(max_ap as u32, ЦВЕТ_ЗАГОЛОВОК);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println("  Население: 10  |  Еда: 20  |  Знания: 0", ЦВЕТ_ОБЫЧНЫЙ);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    println("  Письменности нет. Сохранений нет.", ЦВЕТ_ПРОВАЛ);
    println("  Выживите. И может быть однажды...", ЦВЕТ_ПОДСКАЗКА);
    println("", ЦВЕТ_ОБЫЧНЫЙ);
    print_separator(ЦВЕТ_ЗАГОЛОВОК);
    print("  [Enter] Начало...", ЦВЕТ_ПОДСКАЗКА);
}
