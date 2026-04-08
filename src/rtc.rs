// CMOS RTC — часы реального времени и статус батарейки
// Читаем через порты 0x70 (индекс) и 0x71 (данные)

use x86_64::instructions::port::Port;

/// Читает байт из CMOS регистра
unsafe fn cmos_read(reg: u8) -> u8 {
    unsafe {
        let mut index: Port<u8> = Port::new(0x70);
        let mut data: Port<u8> = Port::new(0x71);
        index.write(reg);
        data.read()
    }
}

/// Ждём пока RTC не обновляется (бит 7 регистра 0x0A)
unsafe fn wait_rtc_ready() {
    unsafe {
        while cmos_read(0x0A) & 0x80 != 0 {}
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

/// Читает текущее время из RTC
pub fn read_rtc() -> DateTime {
    unsafe {
        // Ждём стабильные данные
        wait_rtc_ready();

        let seconds = cmos_read(0x00);
        let minutes = cmos_read(0x02);
        let hours = cmos_read(0x04);
        let day = cmos_read(0x07);
        let month = cmos_read(0x08);
        let year = cmos_read(0x09);

        // Проверяем формат: BCD или binary (регистр B, бит 2)
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

    // RTC обычно в UTC, добавляем +3 для Москвы
    dt.hours = dt.hours + 3;
    if dt.hours >= 24 {
        dt.hours -= 24;
        dt.day += 1;
        // Не заморачиваемся с переносом месяцев — для демо хватит
    }

    dt
}

/// Проверяет батарейку CMOS (регистр D, бит 7)
/// true = батарейка жива, false = сдохла
pub fn battery_ok() -> bool {
    unsafe { cmos_read(0x0D) & 0x80 != 0 }
}

/// Показывает часы и батарейку на экране
pub fn display_status() {
    let dt = read_moscow_time();
    let bat = battery_ok();

    crate::println!(
        "Время (МСК): {:02}:{:02}:{:02}  Дата: {:02}.{:02}.{}",
        dt.hours, dt.minutes, dt.seconds,
        dt.day, dt.month, dt.year
    );

    if bat {
        crate::println!("Батарейка CMOS: ОК (жива)");
    } else {
        crate::println!("Батарейка CMOS: СДОХЛА!");
    }
}
