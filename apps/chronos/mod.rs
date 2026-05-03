// ============================================================
// ТРОЙНОЕ ВРЕМЯ + ТРИГОНОМЕТРИЧЕСКИЕ ЧАСЫ
// ============================================================
// 1. Человеческое — из RTC (уже есть в rtc.rs)
// 2. Шестнадцатеричное — сырые регистры CMOS в hex
// 3. Психотаунское (Мир 100) — 100 дней в месяце, 1200 дней в году
// 4. Тригочасы — время как угол на тригонометрическом круге
// ============================================================

use core::arch::asm;

// ============================================================
// ПСИХОТАУНСКИЙ КАЛЕНДАРЬ (Мир 100)
// ============================================================
// Эпоха: 1 января 2000 года (UTC).
// 1 месяц = 100 дней, 1 год = 12 месяцев = 1200 дней.
// ============================================================

/// Количество дней в месяце земного календаря
fn days_in_month(year: u16, month: u8) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Считает полные дни от 01.01.2000 до заданной даты
fn days_since_epoch(year: u16, month: u8, day: u8) -> u32 {
    let mut total: u32 = 0;
    // Полные годы
    for y in 2000..year {
        total += if (y % 4 == 0) && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
    }
    // Полные месяцы текущего года
    for m in 1..month {
        total += days_in_month(year, m);
    }
    // Дни текущего месяца (минус 1: день 1 = 0)
    if day > 0 {
        total += (day - 1) as u32;
    }
    total
}

/// Дата в Мире 100
pub struct PsychotownDate {
    pub day: u8,   // 0..99
    pub month: u8, // 1..12
    pub year: u32,
    pub total_days: u32,
}

/// Переводит земную дату в Мир 100
pub fn to_psychotown(year: u16, month: u8, day: u8) -> PsychotownDate {
    let total = days_since_epoch(year, month, day);
    let p_year = total / 1200;
    let rem = total % 1200;
    let p_month = (rem / 100) as u8 + 1;
    let p_day = (rem % 100) as u8;
    PsychotownDate {
        day: p_day,
        month: p_month,
        year: p_year,
        total_days: total,
    }
}

// ============================================================
// HEX TIME — время как его видит процессор (сырые BCD)
// ============================================================

unsafe fn cmos_read_raw(reg: u8) -> u8 {
    // Используем функцию без задержки для циклов ожидания
    unsafe { crate::validator::cmos_read_no_delay(reg) }
}

/// Выводит время в шестнадцатеричном формате (сырые регистры CMOS).
/// Правило: probe_cmos() перед любым чтением CMOS-порта.
pub fn display_hex_time() {
    // Сначала опрос: жив ли чип и батарейка?
    let report = crate::validator::probe_cmos();
    if !report.chip_alive {
        crate::locale::print_localized_line(
            crate::kernel_messages::current(crate::kernel_messages::UiText::ChronosHexNoChip),
            0x0C,
        );
        return;
    }
    if !report.battery_ok {
        crate::locale::print_localized_line(
            crate::kernel_messages::current(crate::kernel_messages::UiText::ChronosHexDeadBattery),
            0x0C,
        );
    }
    unsafe {
        // Ждём стабильного чтения
        while cmos_read_raw(0x0A) & 0x80 != 0 {}

        let sec = cmos_read_raw(0x00);
        let min = cmos_read_raw(0x02);
        let hour = cmos_read_raw(0x04);
        let day = cmos_read_raw(0x07);
        let month = cmos_read_raw(0x08);
        let year = cmos_read_raw(0x09);

        crate::locale::print_localized_fmt(
            0x0E,
            format_args!(
                "  0x{:02X}:0x{:02X}:0x{:02X}  0x{:02X}.0x{:02X}.0x20{:02X}",
                hour, min, sec, day, month, year
            ),
        );
    }
}

// ============================================================
// ТРИГОНОМЕТРИЧЕСКИЕ ЧАСЫ АРХИТЕКТОРА ХАОСА
// ============================================================
// Часы = угол на тригонометрическом круге.
// 12:00 = π/2 (90°), 3:00 = 0° (2π), 6:00 = 3π/2 (270°), 9:00 = π (180°).
// Минуты в корнях: sin/cos от текущего угла.
// ============================================================

/// Вычисляет sin через x87. Вход: градусы * 100, выход: значение * 10000
fn fpu_sin_deg(deg100: i64) -> i64 {
    let result: i64;
    unsafe {
        asm!(
            // Переводим градусы*100 в радианы: deg100 / 100 * pi / 180
            "push {d}",
            "fild qword ptr [rsp]",      // deg100
            "add rsp, 8",

            // Делим на 18000 (= 100 * 180) и умножаем на pi
            "push {div}",
            "fild qword ptr [rsp]",
            "add rsp, 8",
            "fdivp",                      // deg100 / 18000

            "fldpi",                      // pi
            "fmulp",                      // (deg100/18000) * pi = радианы

            "fsin",

            // Умножаем на 10000 для точности
            "push {scale}",
            "fild qword ptr [rsp]",
            "add rsp, 8",
            "fmulp",

            "push 0",
            "fistp qword ptr [rsp]",
            "pop {out}",

            d = in(reg) deg100,
            div = in(reg) 18000i64,
            scale = in(reg) 10000i64,
            out = out(reg) result,
        );
    }
    result
}

/// Вычисляет cos через x87. Вход: градусы * 100, выход: значение * 10000
fn fpu_cos_deg(deg100: i64) -> i64 {
    let result: i64;
    unsafe {
        asm!(
            "push {d}",
            "fild qword ptr [rsp]",
            "add rsp, 8",
            "push {div}",
            "fild qword ptr [rsp]",
            "add rsp, 8",
            "fdivp",
            "fldpi",
            "fmulp",
            "fcos",
            "push {scale}",
            "fild qword ptr [rsp]",
            "add rsp, 8",
            "fmulp",
            "push 0",
            "fistp qword ptr [rsp]",
            "pop {out}",
            d = in(reg) deg100,
            div = in(reg) 18000i64,
            scale = in(reg) 10000i64,
            out = out(reg) result,
        );
    }
    result
}

/// Определяет ближайшую «красивую» форму с корнями для значения sin/cos
/// Вход: value * 10000. Возвращает строку типа "√3/2" или "√2/2".
fn radical_name(val10000: i64) -> &'static str {
    // Точные значения * 10000:
    // 0     = 0
    // 1/2   = 5000
    // √2/2  = 7071
    // √3/2  = 8660
    // 1     = 10000
    let abs_val = if val10000 < 0 { -val10000 } else { val10000 };
    let neg = val10000 < 0;

    // Допуск ± 600 (0.06) — хватает для минутной погрешности
    if abs_val < 600 {
        return "0";
    }
    if abs_val > 600 && abs_val < 3500 {
        if neg {
            return "-(0..1/2)";
        } else {
            return "(0..1/2)";
        }
    }
    if abs_val >= 3500 && abs_val < 5600 {
        if neg { return "-1/2" } else { return "1/2" }
    }
    if abs_val >= 5600 && abs_val < 7700 {
        if neg { return "-V2/2" } else { return "V2/2" }
    }
    if abs_val >= 7700 && abs_val < 9300 {
        if neg { return "-V3/2" } else { return "V3/2" }
    }
    if abs_val >= 9300 {
        if neg { return "-1" } else { return "1" }
    }
    "~"
}

/// Выводит тригонометрические часы
pub fn display_trig_clock() {
    let dt = crate::apps::rtc::read_moscow_time();

    // Угол: α = 90° - 30° * (hours % 12) - 0.5° * minutes
    // В единицах deg*100:  α = 9000 - 3000 * (h%12) - 50 * min
    let h = (dt.hours % 12) as i64;
    let m = dt.minutes as i64;
    let _s = dt.seconds as i64;

    let alpha_deg100 = 9000 - 3000 * h - 50 * m;

    // Нормализуем в 0..36000
    let alpha_norm = ((alpha_deg100 % 36000) + 36000) % 36000;

    // Вычисляем sin и cos через x87
    let sin_val = fpu_sin_deg(alpha_deg100);
    let cos_val = fpu_cos_deg(alpha_deg100);

    // Определяем «корневую» форму
    let sin_radical = radical_name(sin_val);
    let cos_radical = radical_name(cos_val);

    // Градусы для отображения
    let deg_whole = alpha_norm / 100;
    let deg_frac = alpha_norm % 100;

    crate::kernel_messages::print_chronos_angle(deg_whole, deg_frac);
    let sin_sign = if sin_val < 0 { "-" } else { "" };
    let sin_abs = if sin_val < 0 { -sin_val } else { sin_val };
    crate::locale::print_localized_fmt(
        0x0E,
        format_args!(
            "  sin = {}{}.{:02}  [{}]",
            sin_sign,
            sin_abs / 10000,
            (sin_abs % 10000) / 100,
            sin_radical
        ),
    );

    let cos_sign = if cos_val < 0 { "-" } else { "" };
    let cos_abs = if cos_val < 0 { -cos_val } else { cos_val };
    crate::locale::print_localized_fmt(
        0x0E,
        format_args!(
            "  cos = {}{}.{:02}  [{}]",
            cos_sign,
            cos_abs / 10000,
            (cos_abs % 10000) / 100,
            cos_radical
        ),
    );
    crate::kernel_messages::print_chronos_roots(sin_radical, cos_radical);
}

// ============================================================
// ГЛАВНАЯ КОМАНДА: время / time
// ============================================================

/// Выводит время в трёх реальностях + тригочасы.
/// Правило: probe_cmos() перед чтением RTC.
pub fn display_triple_time() {
    let report = crate::validator::probe_cmos();
    if !report.chip_alive {
        crate::locale::print_localized_line(
            crate::kernel_messages::current(crate::kernel_messages::UiText::ChronosTimeNoChip),
            0x0C,
        );
        return;
    }
    if !report.battery_ok {
        crate::locale::print_localized_line(
            crate::kernel_messages::current(crate::kernel_messages::UiText::ChronosTimeDeadBattery),
            0x0C,
        );
    }
    let dt = crate::apps::rtc::read_moscow_time();

    // === Заголовок ===
    crate::locale::print_localized_line(
        crate::kernel_messages::current(crate::kernel_messages::UiText::ChronosHeader),
        0x0E,
    );

    // 1. Человеческое
    crate::locale::print_localized_line(
        crate::kernel_messages::current(crate::kernel_messages::UiText::ChronosHuman),
        0x0B,
    );
    crate::locale::print_localized_fmt(
        0x0E,
        format_args!(
            "  {:02}:{:02}:{:02}  {:02}.{:02}.{}",
            dt.hours, dt.minutes, dt.seconds, dt.day, dt.month, dt.year
        ),
    );

    // 2. Шестнадцатеричное (сырые регистры CMOS)
    crate::locale::print_localized_line(
        crate::kernel_messages::current(crate::kernel_messages::UiText::ChronosHex),
        0x0B,
    );
    display_hex_time();

    // 3. Психотаунское (Мир 100)
    let p = to_psychotown(dt.year, dt.month, dt.day);
    crate::locale::print_localized_line(
        crate::kernel_messages::current(crate::kernel_messages::UiText::ChronosPsychotown),
        0x0B,
    );
    crate::locale::print_localized_fmt(
        0x0E,
        format_args!(
            "  {:02}.{:02}.{:04}  ({} days since 2000 epoch)",
            p.day, p.month, p.year, p.total_days
        ),
    );
    crate::locale::print_localized_line(
        crate::kernel_messages::current(crate::kernel_messages::UiText::ChronosPsychotownHint),
        0x0E,
    );

    // 4. Тригонометрические часы
    crate::locale::print_localized_line(
        crate::kernel_messages::current(crate::kernel_messages::UiText::ChronosTrigClock),
        0x0B,
    );
    display_trig_clock();

    crate::locale::print_localized_line(
        crate::kernel_messages::current(crate::kernel_messages::UiText::ChronosFooter),
        0x0A,
    );
}
