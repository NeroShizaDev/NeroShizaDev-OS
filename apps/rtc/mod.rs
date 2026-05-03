// CMOS RTC — часы реального времени и статус батарейки
// Использует общие функции из validator.rs для устранения дублирования

use core::arch::asm;

/// Читает CMOS регистр (использует общую функцию из validator)
unsafe fn cmos_read(reg: u8) -> u8 {
    crate::validator::cmos_read(reg)
}

/// Ждём пока RTC не обновляется (бит 7 регистра 0x0A).
/// На современном железе (Alder Lake + мёртвая CMOS-батарейка) UIP может
/// зависнуть навсегда — ограничиваем 100 000 итераций (~100 мс).
unsafe fn wait_rtc_ready() {
    let mut retries: u32 = 0;
    while crate::validator::cmos_probe_reg(0x0A) & 0x80 != 0 {
        retries += 1;
        if retries >= 100_000 {
            crate::serial_println!("[RTC] wait_rtc_ready: timeout (UIP stuck), proceeding anyway");
            break;
        }
        core::hint::spin_loop();
    }
}

/// Конвертирует BCD в обычное число
fn bcd_to_bin(bcd: u8) -> u8 {
    (bcd & 0x0F) + ((bcd >> 4) * 10)
}

/// Структура времени
pub struct DateTime {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

#[inline]
fn utc_to_moscow_hours(hours: u8) -> (u8, bool) {
    let shifted = hours.wrapping_add(3);
    (shifted % 24, shifted < hours)
}

/// Читает текущее время из RTC
pub fn read_rtc() -> DateTime {
    unsafe {
        wait_rtc_ready();

        let seconds = cmos_read(0x00);
        let minutes = cmos_read(0x02);
        let hours = cmos_read(0x04);
        let day = cmos_read(0x07);
        let month = cmos_read(0x08);
        let year = cmos_read(0x09);

        let reg_b = cmos_read(0x0B);
        let is_bcd = reg_b & 0x04 == 0;

        let (s, m, h, d, mo, y) = if is_bcd {
            (
                bcd_to_bin(seconds),
                bcd_to_bin(minutes),
                bcd_to_bin(hours),
                bcd_to_bin(day),
                bcd_to_bin(month),
                bcd_to_bin(year) as u16,
            )
        } else {
            (seconds, minutes, hours, day, month, year as u16)
        };

        DateTime {
            seconds: s,
            minutes: m,
            hours: h,
            day: d,
            month: mo,
            year: 2000 + y,
        }
    }
}

/// Возвращает время по Москве (UTC+3)
pub fn read_moscow_time() -> DateTime {
    let mut dt = read_rtc();
    let (hours, day_overflow) = utc_to_moscow_hours(dt.hours);
    dt.hours = hours;
    if day_overflow {
        dt.day = dt.day.wrapping_add(1);
    }
    dt
}

/// Проверяет батарейку CMOS (регистр D, бит 7)
/// true = батарейка жива, false = сдохла
pub fn battery_ok() -> bool {
    unsafe { cmos_read(0x0D) & 0x80 != 0 }
}

/// Показывает часы и батарейку на экране.
/// Сначала опрашивает порт через validator::probe_cmos(),
/// затем читает время только если чип жив и батарейка ОК.
pub fn display_status() {
    // Шаг 1: опрос порта — жив ли чип вообще
    let report = crate::validator::probe_cmos();
    crate::validator::display_cmos_probe(&report);

    // Шаг 2: читаем время только если данные достоверны
    if !report.chip_alive {
        crate::locale::print_localized_line(
            crate::kernel_messages::current(crate::kernel_messages::UiText::RtcNoChip),
            0x0C,
        );
        return;
    }

    if !report.battery_ok {
        crate::locale::print_localized_line(
            crate::kernel_messages::current(crate::kernel_messages::UiText::RtcDeadBattery),
            0x0C,
        );
        // Всё равно показываем время — но с предупреждением
    }

    if !report.rtc_ready {
        crate::locale::print_localized_line(
            crate::kernel_messages::current(crate::kernel_messages::UiText::RtcUipStuck),
            0x0C,
        );
        return;
    }

    let dt = read_moscow_time();

    if crate::locale::get_locale() == crate::kernel_messages::Locale::ArEg {
        crate::locale::print_localized_fmt(
            0x0E,
            format_args!(
                "الوقت (مصر): {:02}:{:02}:{:02}  التاريخ: {:02}.{:02}.{}",
                dt.hours, dt.minutes, dt.seconds, dt.day, dt.month, dt.year
            ),
        );
        return;
    }

    crate::kernel_messages::print_rtc_line(
        dt.hours, dt.minutes, dt.seconds, dt.day, dt.month, dt.year,
    );
}

/// Читает IA32_THERM_STATUS (MSR 0x19C) и выводит температуру CPU.
/// Биты 22:16 = отступ в °C до порога троттлинга (TjMax).
/// Бит 4 = PROCHOT log (троттлинг уже был зафиксирован).
/// В QEMU MSR 0x19C возвращает 0 → margin=0, temp=TjMax — нормально.
fn thermal_status_supported() -> bool {
    let mut max_leaf: u32;
    let mut eax6: u32;

    unsafe {
        asm!(
            "push rbx",
            "xor ecx, ecx",
            "xor edx, edx",
            "mov eax, 0",
            "cpuid",
            "mov {max_leaf:e}, eax",
            "pop rbx",
            max_leaf = lateout(reg) max_leaf,
            out("eax") _,
            out("ecx") _,
            out("edx") _,
        );
    }

    if max_leaf < 6 {
        return false;
    }

    unsafe {
        asm!(
            "push rbx",
            "xor ecx, ecx",
            "mov eax, 6",
            "cpuid",
            "mov {eax6:e}, eax",
            "pop rbx",
            eax6 = lateout(reg) eax6,
            out("eax") _,
            out("ecx") _,
            out("edx") _,
        );
    }

    eax6 & 1 != 0
}

pub struct ThermalProbe {
    pub throttle_logged: bool,
    pub margin_c_to_tjmax: u32,
    pub estimated_temp_c: u32,
}

/// Пытается прочитать IA32_THERM_STATUS и вернуть компактный снимок.
/// None = MSR недоступен на текущем CPU.
pub fn probe_thermal() -> Option<ThermalProbe> {
    if !thermal_status_supported() {
        return None;
    }

    let eax: u32;
    let _edx: u32;

    // SAFETY: rdmsr — привилегированная инструкция, доступна на ring-0.
    // MSR 0x19C = IA32_THERM_STATUS, поддерживается на Intel Core/Xeon.
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") 0x19cu32,
            out("eax") eax,
            out("edx") _edx,
            options(nomem, nostack),
        );
    }

    let throttle_logged = (eax >> 4) & 1 != 0; // бит 4: PROCHOT# log
    let margin = (eax >> 16) & 0x7F; // биты 22:16: °C до TjMax
    let tj_max = 100u32; // типовой TjMax Intel
    let temp = tj_max.saturating_sub(margin);

    Some(ThermalProbe {
        throttle_logged,
        margin_c_to_tjmax: margin,
        estimated_temp_c: temp,
    })
}

pub fn display_thermal() {
    let Some(probe) = probe_thermal() else {
        match crate::locale::get_locale() {
            crate::kernel_messages::Locale::RuRu => {
                crate::locale::print_localized_line("[TEMP] Термодатчик CPU недоступен", 0x0C)
            }
            crate::kernel_messages::Locale::EnUs => {
                crate::locale::print_localized_line("[TEMP] CPU thermal sensor unavailable", 0x0C)
            }
            crate::kernel_messages::Locale::ArEg => {
                crate::locale::print_localized_line("[TEMP] حساس حرارة المعالج غير متاح", 0x0C)
            }
        }
        return;
    };

    if probe.throttle_logged {
        crate::kernel_messages::print_rtc_throttle(probe.estimated_temp_c, probe.margin_c_to_tjmax);
    } else {
        crate::kernel_messages::print_rtc_temp(probe.estimated_temp_c, probe.margin_c_to_tjmax);
    }
}
