// tribe.rs — состояние игры

use super::gurps::skill_target;
use super::x87_rng::X87Rng;

// ═══════════════════════════════════════════════════════════
// ФЛАГИ — 128 бит = 128 уникальных состояний мира
// ═══════════════════════════════════════════════════════════
pub mod flags {
    // Охота
    pub const МЕТОД_ТЕРПЕНИЯ: u128 = 1 << 0;
    pub const ТРОПА_ЗВЕРЕЙ: u128 = 1 << 1;
    pub const ЛОВУШКИ: u128 = 1 << 2;
    pub const МЕТОД_ШКУРЫ: u128 = 1 << 3;
    pub const РЕПУТАЦИЯ_ДУХОВ: u128 = 1 << 4;
    pub const ПЕРВОЕ_СТАДО: u128 = 1 << 5;
    pub const ДИПЛОМАТИЯ_ПРИРОДЫ: u128 = 1 << 6;
    pub const КУЛЬТ_БЕССМЕРТНОГО: u128 = 1 << 7;
    pub const МЕДВЕДЬ_ПОМНИТ: u128 = 1 << 8;

    // Сбор
    pub const ТАЙНАЯ_ПОЛЯНА: u128 = 1 << 9;
    pub const ПЕРВЫЙ_ТРАВНИК: u128 = 1 << 10;
    pub const ХРАНЕНИЕ_ЕДЫ: u128 = 1 << 11;
    pub const КОРЗИНЫ: u128 = 1 << 12;
    pub const ЛЕЧЕБНАЯ_ЯГОДА: u128 = 1 << 13;
    pub const ПРОКЛЯТЫЕ_ЛИСТЬЯ: u128 = 1 << 14;
    pub const НАС_ИЗУЧАЮТ: u128 = 1 << 15;
    pub const ПЕРВОЕ_ПИСЬМО: u128 = 1 << 16;
    pub const СЧЁТ: u128 = 1 << 17;
    pub const ХРАНИЛИЩЕ: u128 = 1 << 18;
    pub const ПОСУДА: u128 = 1 << 19;
    pub const ОБЩЕЕ_ВИДЕНИЕ: u128 = 1 << 20;
    pub const ДОЛГ_КРОВИ: u128 = 1 << 21;
    pub const ВЕЧНЫЙ_КУСТ: u128 = 1 << 22;

    // Размышление
    pub const ОГОНЬ: u128 = 1 << 23;
    pub const СТРОИТЕЛЬСТВО: u128 = 1 << 24;
    pub const ПИСЬМО_30: u128 = 1 << 25; // ускоряет Письменность на 30%
    pub const ЯЗЫК_РАЗВИВАЕТСЯ: u128 = 1 << 26;
    pub const ПОЭЗИЯ: u128 = 1 << 27;
    pub const ЛЕТОПИСЬ: u128 = 1 << 28;
    pub const ФИЛОСОФИЯ: u128 = 1 << 29;
    pub const НАУКА: u128 = 1 << 30;
    pub const ТРАДИЦИЯ_ЗАГАДОК: u128 = 1 << 31;
    pub const СМЕНА_ВОЖДЯ: u128 = 1 << 32;
    pub const ЭПОС_НАРОДА: u128 = 1 << 33;
    pub const ЧУЖАК_ПОБЕДИТЕЛЬ: u128 = 1 << 34;
    pub const КОСМОЛОГИЯ: u128 = 1 << 35;
    pub const КУЛИНАРИЯ: u128 = 1 << 36;
    pub const РИТУАЛ_ПОГРЕБЕНИЯ: u128 = 1 << 37;

    // Разведка
    pub const НОВАЯ_ТЕРРИТОРИЯ: u128 = 1 << 38;
    pub const РЕКА_НАЙДЕНА: u128 = 1 << 39;
    pub const ТЕРРИТОРИЯ_КАРТА: u128 = 1 << 40;
    pub const ПРЕДУПРЕЖДЕНИЕ: u128 = 1 << 41;
    pub const ПЕЩЕРА_НАЙДЕНА: u128 = 1 << 42;
    pub const ВРАГИ_ЗНАЮТ: u128 = 1 << 43;
    pub const ОКРУЖЕНЫ: u128 = 1 << 44;
    pub const МЕТОД_ЗАПАХА: u128 = 1 << 45;
    pub const ЗАГАДОЧНЫЙ_РИТУАЛ: u128 = 1 << 46;
    pub const АКУСТИКА: u128 = 1 << 47;
    pub const СОЮЗ_ТУПЫХ: u128 = 1 << 48;
    pub const ГОРА_НАЙДЕНА: u128 = 1 << 49;
    pub const СВОБОДНАЯ_ЗЕМЛЯ: u128 = 1 << 50;
    pub const ЗВЕНЯЩИЙ_МЕТОД: u128 = 1 << 51;

    // Отдых / прочее
    pub const ПЕРВЫЙ_ВОЛК: u128 = 1 << 52;
    pub const ВИДЕНИЕ_ВО_СНЕ: u128 = 1 << 53;
    pub const ДЕЗЕРТИР: u128 = 1 << 54;
    pub const НАС_ИЩУТ: u128 = 1 << 55;
    pub const КОНФЛИКТ_ИЗ_ЗА_ЯГОД: u128 = 1 << 56;
    pub const ТЕРРИТОРИЯ_ПОТЕРЯНА: u128 = 1 << 57;
    pub const КАБАН_ПЕТРОВИЧ: u128 = 1 << 58;
    pub const РИТУАЛ_КОПЬЯ: u128 = 1 << 59;
    pub const ОПАСНОСТЬ_РЯДОМ: u128 = 1 << 60;
    pub const ИСКУССТВО: u128 = 1 << 61;
    pub const КРАСКА: u128 = 1 << 62;

    // === ГЛАВНАЯ ЦЕЛЬ ИГРЫ ===
    pub const ПИСЬМЕННОСТЬ: u128 = 1 << 127;
}

// ═══════════════════════════════════════════════════════════
// АТРИБУТЫ
// ═══════════════════════════════════════════════════════════
#[derive(Clone, Copy)]
pub struct Attributes {
    pub strength: u8,     // СИЛ 3..18
    pub dexterity: u8,    // ЛОВ
    pub intelligence: u8, // ИНТ
    pub health: u8,       // ЗДР
}

impl Attributes {
    /// 3d6 на каждый атрибут, сумма не больше 44, минимум 6
    pub fn roll(rng: &mut X87Rng) -> Self {
        let mut a = Self {
            strength: rng.roll_3d6(),
            dexterity: rng.roll_3d6(),
            intelligence: rng.roll_3d6(),
            health: rng.roll_3d6(),
        };
        // Антиманчкин: срезаем максимум в минимум пока сумма > 44
        loop {
            let sum =
                a.strength as u16 + a.dexterity as u16 + a.intelligence as u16 + a.health as u16;
            if sum <= 44 {
                break;
            }

            let vals = [a.strength, a.dexterity, a.intelligence, a.health];
            let max_i = vals
                .iter()
                .enumerate()
                .max_by_key(|&(_, v)| v)
                .map(|(i, _)| i)
                .unwrap_or(0);
            let min_i = vals
                .iter()
                .enumerate()
                .min_by_key(|&(_, v)| v)
                .map(|(i, _)| i)
                .unwrap_or(0);

            // Срезаем с максимума
            match max_i {
                0 => a.strength = a.strength.saturating_sub(1),
                1 => a.dexterity = a.dexterity.saturating_sub(1),
                2 => a.intelligence = a.intelligence.saturating_sub(1),
                3 => a.health = a.health.saturating_sub(1),
                _ => {}
            }
            // Добавляем к минимуму если он < 6
            let min_val = [a.strength, a.dexterity, a.intelligence, a.health][min_i];
            if min_val < 6 {
                match min_i {
                    0 => a.strength += 1,
                    1 => a.dexterity += 1,
                    2 => a.intelligence += 1,
                    3 => a.health += 1,
                    _ => {}
                }
            }
        }
        // Минимум 3 везде (GURPS позволяет)
        a.strength = a.strength.max(3);
        a.dexterity = a.dexterity.max(3);
        a.intelligence = a.intelligence.max(3);
        a.health = a.health.max(3);
        a
    }
}

// ═══════════════════════════════════════════════════════════
// НАВЫКИ
// ═══════════════════════════════════════════════════════════
#[derive(Clone, Copy, Default)]
pub struct Skills {
    pub hunting: u8, // 0=нет 1=новичок 2=профи
    pub gathering: u8,
    pub thinking: u8,
    pub scouting: u8,
    pub combat: u8,
    pub medicine: u8,
    pub building: u8,
}

// ═══════════════════════════════════════════════════════════
// ЛОГ ИСТОРИИ (кольцевой буфер)
// ═══════════════════════════════════════════════════════════
pub const LOG_SIZE: usize = 64;
pub const MSG_LEN: usize = 64;

pub struct HistoryLog {
    pub buffer: [[u32; MSG_LEN]; LOG_SIZE],
    pub head: usize, // следующая позиция для записи
}

impl HistoryLog {
    pub const fn new() -> Self {
        Self {
            buffer: [[0; MSG_LEN]; LOG_SIZE],
            head: 0,
        }
    }

    pub fn push(&mut self, msg: &str) {
        let idx = self.head % LOG_SIZE;
        let mut len = 0;
        for ch in msg.chars().take(MSG_LEN) {
            self.buffer[idx][len] = ch as u32;
            len += 1;
        }
        for i in len..MSG_LEN {
            self.buffer[idx][i] = 0;
        }
        self.head = self.head.wrapping_add(1);
    }

    /// Получить N последних записей (для отображения)
    pub fn get_recent(&self, n: usize) -> impl Iterator<Item = &[u32; MSG_LEN]> {
        let count = n.min(self.head.min(LOG_SIZE));
        let start = if self.head >= count {
            self.head - count
        } else {
            0
        };
        (start..self.head).map(move |i| &self.buffer[i % LOG_SIZE])
    }
}

// ═══════════════════════════════════════════════════════════
// TRIBE — основная структура ~120 байт + лог
// ═══════════════════════════════════════════════════════════
pub struct Tribe {
    pub name: [u8; 16],
    pub attrs: Attributes,
    pub skills: Skills,
    pub population: u32,
    pub food: i32,
    pub knowledge: u32,
    pub xp: u32,
    pub flags: u128,
    pub turn: u32,
    pub authority: i8, // авторитет вождя -10..+10
    pub morale: i8,    // настроение племени -10..+10
    pub log: HistoryLog,

    // Отложенные события (через N ходов что-то случится)
    pub pending_raid_in: u8, // 0 = нет, N = через N ходов набег
    pub pending_guests_in: u8,
    pub pending_conflict_in: u8,
}

impl Tribe {
    pub fn new(name: &[u8], rng: &mut X87Rng) -> Self {
        let mut n = [b' '; 16];
        let len = name.len().min(16);
        n[..len].copy_from_slice(&name[..len]);

        Self {
            name: n,
            attrs: Attributes::roll(rng),
            skills: Skills::default(),
            population: 10,
            food: 20,
            knowledge: 0,
            xp: 0,
            flags: 0,
            turn: 1,
            authority: 0,
            morale: 0,
            log: HistoryLog::new(),
            pending_raid_in: 0,
            pending_guests_in: 0,
            pending_conflict_in: 0,
        }
    }

    // --- Флаги ---
    pub fn set_flag(&mut self, f: u128) {
        self.flags |= f;
    }
    pub fn has_flag(&self, f: u128) -> bool {
        self.flags & f != 0
    }
    pub fn clear_flag(&mut self, f: u128) {
        self.flags &= !f;
    }

    // --- Вторичные параметры (GURPS) ---

    /// Максимум ОД на ход
    pub fn max_ap(&self) -> u8 {
        let base = 2u8;
        let bonus = (self.attrs.intelligence / 5).min(3);
        let morale_bonus = if self.morale >= 3 { 1u8 } else { 0 };
        base + bonus + morale_bonus
    }

    /// Базовая скорость/инициатива (DX+HT)/4
    pub fn speed(&self) -> u8 {
        (self.attrs.dexterity as u16 + self.attrs.health as u16) as u8 / 4
    }

    /// Сколько еды потребляет племя за ход
    pub fn food_per_turn(&self) -> i32 {
        let mut cost = self.population as i32;
        // Кабан Петрович ест тоже
        if self.has_flag(flags::КАБАН_ПЕТРОВИЧ) {
            cost += 1;
        }
        cost
    }

    /// Цель броска для навыка охоты
    pub fn hunt_target(&self) -> u8 {
        let mut t = skill_target(self.attrs.dexterity, self.skills.hunting, -8);
        // Бонусы от флагов
        if self.has_flag(flags::ТРОПА_ЗВЕРЕЙ) {
            t = t.saturating_add(3);
        }
        if self.has_flag(flags::ЛОВУШКИ) {
            t = t.saturating_add(2);
        }
        if self.has_flag(flags::РИТУАЛ_КОПЬЯ) {
            t = t.saturating_add(1);
        }
        t.min(18)
    }

    pub fn gather_target(&self) -> u8 {
        let mut t = skill_target(self.attrs.dexterity, self.skills.gathering, -5);
        if self.has_flag(flags::ТАЙНАЯ_ПОЛЯНА) {
            t = t.saturating_add(2);
        }
        if self.has_flag(flags::КОРЗИНЫ) {
            t = t.saturating_add(1);
        }
        t.min(18)
    }

    pub fn think_target(&self) -> u8 {
        let mut t = skill_target(self.attrs.intelligence, self.skills.thinking, -5);
        if self.has_flag(flags::ФИЛОСОФИЯ) {
            t = t.saturating_add(1);
        }
        if self.has_flag(flags::ТРАДИЦИЯ_ЗАГАДОК) {
            t = t.saturating_add(1);
        }
        t.min(18)
    }

    pub fn scout_target(&self) -> u8 {
        let mut t = skill_target(self.attrs.dexterity, self.skills.scouting, -6);
        if self.has_flag(flags::ТЕРРИТОРИЯ_КАРТА) {
            t = t.saturating_add(2);
        }
        t.min(18)
    }

    // --- Применение эффектов ---

    pub fn add_food(&mut self, amount: i32) {
        self.food += amount;
        // Бонус корзин: +20% к сбору
        if self.has_flag(flags::КОРЗИНЫ) && amount > 0 {
            self.food += amount / 5;
        }
    }

    pub fn add_knowledge(&mut self, amount: u32) {
        self.knowledge = self.knowledge.saturating_add(amount);
        self.xp = self.xp.saturating_add(amount / 2);
    }

    pub fn lose_population(&mut self, amount: u32) {
        self.population = self.population.saturating_sub(amount);
    }

    /// Конец хода: еда съедена, проверки
    pub fn end_of_turn(&mut self) -> TurnResult {
        let consumption = self.food_per_turn();
        self.food -= consumption;

        let raid_due = self.pending_raid_in == 1;
        let guests_due = self.pending_guests_in == 1;
        let conflict_due = self.pending_conflict_in == 1;

        // Уменьшаем счётчики отложенных событий
        if self.pending_raid_in > 0 {
            self.pending_raid_in -= 1;
        }
        if self.pending_guests_in > 0 {
            self.pending_guests_in -= 1;
        }
        if self.pending_conflict_in > 0 {
            self.pending_conflict_in -= 1;
        }

        // Кабан Петрович даёт +2 еды
        if self.has_flag(flags::КАБАН_ПЕТРОВИЧ) {
            self.food += 2;
        }

        // Вечный куст +3 еды
        if self.has_flag(flags::ВЕЧНЫЙ_КУСТ) {
            self.food += 3;
        }

        self.turn += 1;

        if self.population == 0 {
            return TurnResult::Extinction;
        }
        if self.food < 0 {
            // Голод: теряем население
            let starved = ((-self.food) / 5 + 1).min(self.population as i32) as u32;
            self.lose_population(starved);
            self.food = 0;
            self.log.push("ГОЛОД! Племя теряет людей.");
            if self.population == 0 {
                return TurnResult::Extinction;
            }
            return TurnResult::Famine;
        }
        if conflict_due {
            return TurnResult::Conflict;
        }
        if guests_due {
            return TurnResult::Guests;
        }
        if raid_due {
            return TurnResult::Raid;
        }

        TurnResult::Ok
    }

    /// Проверка открытия Письменности
    pub fn check_writing_unlock(&mut self) -> bool {
        let mut cost = 200u32;
        // Флаги ускоряют открытие
        if self.has_flag(flags::ПЕРВОЕ_ПИСЬМО) {
            cost = cost * 7 / 10;
        } // -30%
        if self.has_flag(flags::ПИСЬМО_30) {
            cost = cost * 7 / 10;
        } // ещё -30%
        if self.has_flag(flags::СЧЁТ) {
            cost = cost * 9 / 10;
        } // -10%
        if self.has_flag(flags::ЛЕТОПИСЬ) {
            cost = cost * 9 / 10;
        }
        if self.has_flag(flags::ЯЗЫК_РАЗВИВАЕТСЯ) {
            cost = cost * 9 / 10;
        }

        if self.knowledge >= cost && !self.has_flag(flags::ПИСЬМЕННОСТЬ) {
            self.set_flag(flags::ПИСЬМЕННОСТЬ);
            return true;
        }
        false
    }
}

pub enum TurnResult {
    Ok,
    Famine,
    Guests,
    Conflict,
    Raid,
    Extinction,
}
