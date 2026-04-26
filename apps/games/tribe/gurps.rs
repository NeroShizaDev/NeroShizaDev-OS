// gurps.rs — математика бросков GURPS Lite
// Правило: бросок 3d6 должен быть МЕНЬШЕ или РАВЕН цели = успех

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    CritSuccess, // 3-4   — всегда успех
    Success,     // <= цели
    Fail,        // > цели
    CritFail,    // 17-18 — всегда провал
}

impl Outcome {
    pub fn index(&self) -> usize {
        match self {
            Outcome::CritSuccess => 0,
            Outcome::Success => 1,
            Outcome::Fail => 2,
            Outcome::CritFail => 3,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Outcome::CritSuccess => ">> КРИТ УСПЕХ",
            Outcome::Success => ">  Успех",
            Outcome::Fail => "<  Провал",
            Outcome::CritFail => "<< КРИТ ПРОВАЛ",
        }
    }
}

/// Основной бросок GURPS
pub fn check(target: u8, roll: u8) -> Outcome {
    match roll {
        3..=4 => Outcome::CritSuccess,
        17..=18 => Outcome::CritFail,
        r if r <= target => Outcome::Success,
        _ => Outcome::Fail,
    }
}

/// Цель броска с учётом навыка или дефолтного штрафа
pub fn skill_target(attr: u8, level: u8, default_penalty: i8) -> u8 {
    let base = attr as i16;
    let result = if level > 0 {
        base + level as i16 - 1
    } else {
        base + default_penalty as i16
    };
    result.clamp(3, 18) as u8
}

/// Цель для "по-тупому" режима — всегда низкая
pub const STUPID_TARGET: u8 = 6;

/// Форматирование результата броска в буфер
/// Формат: "Цель 11 > 9  ✓  Успех" / "Цель 6 < 15  ✗✗ Крит провал"
pub fn format_roll(target: u8, roll: u8, outcome: Outcome, buf: &mut [u8; 48]) -> usize {
    let sign = match outcome {
        Outcome::CritSuccess | Outcome::Success => b'>',
        Outcome::Fail | Outcome::CritFail => b'<',
    };
    // "Бросок: RR > Цель TT" — простое форматирование без fmt
    let mut pos = 0;
    let prefix = "Бросок: ".as_bytes();
    buf[pos..pos + prefix.len()].copy_from_slice(prefix);
    pos += prefix.len();
    pos += write_u8(roll, &mut buf[pos..]);
    buf[pos] = b' ';
    pos += 1;
    buf[pos] = sign;
    pos += 1;
    buf[pos] = b' ';
    pos += 1;
    let target_label = "Цель ".as_bytes();
    buf[pos..pos + target_label.len()].copy_from_slice(target_label);
    pos += target_label.len();
    pos += write_u8(target, &mut buf[pos..]);
    pos
}

fn write_u8(n: u8, buf: &mut [u8]) -> usize {
    if n >= 100 {
        buf[0] = b'0' + n / 100;
        buf[1] = b'0' + (n % 100) / 10;
        buf[2] = b'0' + n % 10;
        3
    } else if n >= 10 {
        buf[0] = b'0' + n / 10;
        buf[1] = b'0' + n % 10;
        2
    } else {
        buf[0] = b'0' + n;
        1
    }
}
