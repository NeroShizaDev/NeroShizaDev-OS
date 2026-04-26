use x86_64::instructions::hlt;

use super::db::{self, Eff};
use super::db_ext;
use super::gurps::{self, Outcome, STUPID_TARGET};
use super::input::{self, KeyEvent};
use super::live_rules;
use super::print::{
    self, clear_screen, print_action_submenu, print_event_text, print_game_header, print_game_over,
    print_inum, print_main_menu, print_num, print_roll_result, print_separator,
    print_tribe_creation, print_writing_unlock,
};
use super::tribe::{Tribe, TurnResult};
use super::watchdog;
use super::x87_rng::X87Rng;

const ACTION_COSTS: [u8; 5] = [2, 1, 2, 1, 0];

pub(super) struct GameState {
    pub(super) tribe: Tribe,
    pub(super) rng: X87Rng,
    pub(super) ap_penalty_turns: u8,
    pub(super) hunt_penalty_turns: u8,
    pub(super) rules_applied_turn: u32,
}

pub fn run() {
    crate::ps2::clear_scancode_queue();
    input::reset();
    watchdog::init();

    let mut state = GameState::new(seed());
    show_creation(&state.tribe);
    if !wait_for_enter() {
        clear_screen();
        return;
    }

    while run_turn(&mut state) {}

    clear_screen();
}

impl GameState {
    fn new(seed: u64) -> Self {
        let mut rng = X87Rng::new(seed);
        let tribe = Tribe::new("Каменное племя".as_bytes(), &mut rng);
        Self {
            tribe,
            rng,
            ap_penalty_turns: 0,
            hunt_penalty_turns: 0,
            rules_applied_turn: 0,
        }
    }
}

fn seed() -> u64 {
    let ticks = unsafe { core::arch::x86_64::_rdtsc() };
    ticks ^ 0x5452_4942_4500_0001
}

fn run_turn(state: &mut GameState) -> bool {
    live_rules::apply_turn_links(state);

    if state.tribe.check_writing_unlock() {
        print_writing_unlock();
        wait_any_key();
    }

    let ap_penalized = state.ap_penalty_turns > 0;
    let hunt_penalized = state.hunt_penalty_turns > 0;
    if state.ap_penalty_turns > 0 {
        state.ap_penalty_turns -= 1;
    }
    if state.hunt_penalty_turns > 0 {
        state.hunt_penalty_turns -= 1;
    }

    let mut ap_left = turn_ap(state, ap_penalized);

    loop {
        render_turn_screen(state, ap_left, ap_penalized, hunt_penalized);

        let Some(key) = next_keydown() else {
            hlt();
            continue;
        };

        match key {
            KeyEvent::Esc => return false,
            KeyEvent::History => {
                show_history_screen(state);
            }
            KeyEvent::Action(action) => {
                let action = action as usize;
                let cost = ACTION_COSTS[action];
                if cost > ap_left {
                    show_notice("Не хватает приказов на это действие.");
                    continue;
                }

                let mut consumed_turn = false;
                if action == 4 {
                    execute_rest(state);
                    consumed_turn = true;
                } else if let Some(stupid) = choose_action_mode(action, cost) {
                    execute_action(state, action, stupid, hunt_penalized);
                } else {
                    continue;
                }

                if state.tribe.population == 0 {
                    show_game_over_screen(&state.tribe);
                    return false;
                }

                if consumed_turn {
                    ap_left = 0;
                } else {
                    ap_left = ap_left.saturating_sub(cost);
                }

                if ap_left == 0 {
                    break;
                }
            }
            KeyEvent::Enter => {}
        }
    }

    let result = state.tribe.end_of_turn();
    handle_turn_result(state, result);
    if state.tribe.population == 0 {
        show_game_over_screen(&state.tribe);
        return false;
    }

    maybe_system_event(state);
    if state.tribe.population == 0 {
        show_game_over_screen(&state.tribe);
        return false;
    }

    true
}

fn turn_ap(state: &GameState, ap_penalized: bool) -> u8 {
    let penalty = if ap_penalized { 1 } else { 0 };
    state.tribe.max_ap().saturating_sub(penalty)
}

fn render_turn_screen(state: &GameState, ap_left: u8, ap_penalized: bool, hunt_penalized: bool) {
    clear_screen();
    print_game_header(
        tribe_name(&state.tribe),
        state.tribe.turn,
        state.tribe.population,
        state.tribe.food,
        state.tribe.knowledge,
        ap_left,
        turn_ap(state, ap_penalized),
    );

    print::print("  Авторитет: ", print::ЦВЕТ_ОБЫЧНЫЙ);
    print_inum(state.tribe.authority as i32, print::ЦВЕТ_КРИТ);
    print::print("   Настроение: ", print::ЦВЕТ_ОБЫЧНЫЙ);
    print_inum(state.tribe.morale as i32, print::ЦВЕТ_КРИТ);
    print::print("   Письменность: ", print::ЦВЕТ_ОБЫЧНЫЙ);
    print_num(writing_cost_left(&state.tribe), print::ЦВЕТ_УСПЕХ);
    print::print(" зн", print::ЦВЕТ_ОБЫЧНЫЙ);
    print::println("", print::ЦВЕТ_ОБЫЧНЫЙ);

    if ap_penalized {
        print::println(
            "  Племя измотано: в этот ход на 1 приказ меньше.",
            print::ЦВЕТ_КРИТ,
        );
    }
    if hunt_penalized {
        print::println(
            "  Охота сорвана: в этот ход цель охоты ниже на 4.",
            print::ЦВЕТ_КРИТ,
        );
    }

    print_main_menu(ap_left);
    print::println("  Enter  выбрать режим действия", print::ЦВЕТ_ПОДСКАЗКА);
    print::println("  H      история племени", print::ЦВЕТ_ПОДСКАЗКА);
    print::println("  Esc    выйти в launcher", print::ЦВЕТ_ПОДСКАЗКА);
    print::println("", print::ЦВЕТ_ОБЫЧНЫЙ);
    print::println("  Последние события:", print::ЦВЕТ_ЗАГОЛОВОК);
    print_recent_history(&state.tribe, 4);
}

fn choose_action_mode(action: usize, cost: u8) -> Option<bool> {
    let (name, desc) = match action {
        0 => (
            "ОХОТА",
            "Охотники идут за мясом и рискуют встретить того, кто сильнее.",
        ),
        1 => (
            "СОБИРАТЬ",
            "Собиратели ищут ягоды, коренья, грибы и всё подозрительно съедобное.",
        ),
        2 => (
            "ДУМАТЬ",
            "Шаман, старейшины и бездельники пытаются превратить день в знание.",
        ),
        3 => (
            "РАЗВЕДКА",
            "Разведчики осматривают округу и иногда возвращаются с хорошими новостями.",
        ),
        _ => return None,
    };

    loop {
        clear_screen();
        print_action_submenu(name, desc, cost);
        match next_keydown() {
            Some(KeyEvent::Action(0)) => return Some(false),
            Some(KeyEvent::Action(1)) => return Some(true),
            Some(KeyEvent::Action(2) | KeyEvent::Esc) => return None,
            Some(_) => {}
            None => hlt(),
        }
    }
}

fn execute_action(state: &mut GameState, action: usize, stupid: bool, hunt_penalized: bool) {
    let target = action_target(state, action, stupid, hunt_penalized);
    let roll = state.rng.roll_3d6();
    let outcome = gurps::check(target, roll);
    let event_idx = state.rng.roll_1d6_minus1();

    let event = if stupid {
        let variant = state.rng.roll_index(db_ext::вариантов_тупых() as usize);
        db_ext::получить_тупое_ext(action, variant, outcome.index(), event_idx)
    } else {
        db::получить_обычное(action, outcome.index(), event_idx)
    };

    show_roll_and_event(roll, target, outcome, event.текст);
    apply_effects(state, &event.эфф);
    live_rules::after_action(state, action, stupid, outcome);
    state.tribe.log.push(event.текст);
}

fn execute_rest(state: &mut GameState) {
    let target = 10;
    let roll = state.rng.roll_3d6();
    let outcome = gurps::check(target, roll);
    let event_idx = state.rng.roll_1d6_minus1();
    let event = db::получить_тупое(4, 0, outcome.index(), event_idx);

    show_roll_and_event(roll, target, outcome, event.текст);
    apply_effects(state, &event.эфф);
    state.tribe.log.push(event.текст);
}

fn action_target(state: &GameState, action: usize, stupid: bool, hunt_penalized: bool) -> u8 {
    let base = if stupid {
        STUPID_TARGET
    } else {
        match action {
            0 => {
                let penalty = if hunt_penalized { 4 } else { 0 };
                state.tribe.hunt_target().saturating_sub(penalty)
            }
            1 => state.tribe.gather_target(),
            2 => state.tribe.think_target(),
            3 => state.tribe.scout_target(),
            _ => 10,
        }
    };

    live_rules::adjust_action_target(state, action, stupid, base)
}

fn show_roll_and_event(roll: u8, target: u8, outcome: Outcome, text: &str) {
    clear_screen();
    print_separator(print::ЦВЕТ_РАМКА);
    print::println("  РЕЗУЛЬТАТ ДЕЙСТВИЯ", print::ЦВЕТ_ЗАГОЛОВОК);
    print_separator(print::ЦВЕТ_РАМКА);
    print_roll_result(
        roll,
        target,
        if roll <= target { ">" } else { "<" },
        outcome.symbol(),
    );
    print_event_text(text);
    wait_any_key();
}

fn apply_effects(state: &mut GameState, effects: &[Eff; 4]) {
    for effect in effects {
        match *effect {
            Eff::Еда(amount) => state.tribe.add_food(amount),
            Eff::Знания(amount) => state.tribe.add_knowledge(amount),
            Eff::Народ(delta) => {
                if delta >= 0 {
                    state.tribe.population = state.tribe.population.saturating_add(delta as u32);
                } else {
                    state.tribe.lose_population((-delta) as u32);
                }
            }
            Eff::Флаг(flag) => state.tribe.set_flag(flag),
            Eff::Авторитет(delta) => {
                state.tribe.authority = (state.tribe.authority + delta).clamp(-10, 10);
            }
            Eff::Настроение(delta) => {
                state.tribe.morale = (state.tribe.morale + delta).clamp(-10, 10);
            }
            Eff::ОД(turns) => {
                state.ap_penalty_turns = state.ap_penalty_turns.max(turns);
            }
            Eff::Набег(turns) => {
                state.tribe.pending_raid_in = state.tribe.pending_raid_in.max(turns);
            }
            Eff::Гости(turns) => {
                state.tribe.pending_guests_in = state.tribe.pending_guests_in.max(turns);
            }
            Eff::Конфликт(turns) => {
                state.tribe.pending_conflict_in = state.tribe.pending_conflict_in.max(turns);
            }
            Eff::БросокЗДР => resolve_health_check(state),
            Eff::БросокЛОВ => resolve_dex_check(state),
            Eff::ЕдаПроц(percent) => apply_percent_food(&mut state.tribe, percent),
            Eff::ЗнанияПроц(percent) => {
                apply_percent_knowledge(&mut state.tribe, percent)
            }
            Eff::ШтрафОхоты(turns) => {
                state.hunt_penalty_turns = state.hunt_penalty_turns.max(turns);
            }
            Eff::Ничего => {}
        }
    }
}

fn resolve_health_check(state: &mut GameState) {
    let roll = state.rng.roll_3d6();
    let outcome = gurps::check(state.tribe.attrs.health, roll);
    let text = if matches!(outcome, Outcome::Fail | Outcome::CritFail) {
        state.tribe.lose_population(1);
        state.tribe.morale = (state.tribe.morale - 1).clamp(-10, 10);
        "Бросок ЗДР провален: племя теряет одного человека."
    } else {
        "Бросок ЗДР выдержан: племя пережило беду."
    };
    state.tribe.log.push(text);
}

fn resolve_dex_check(state: &mut GameState) {
    let roll = state.rng.roll_3d6();
    let outcome = gurps::check(state.tribe.attrs.dexterity, roll);
    let text = if matches!(outcome, Outcome::Fail | Outcome::CritFail) {
        state.tribe.add_food(-4);
        state.tribe.morale = (state.tribe.morale - 1).clamp(-10, 10);
        "Бросок ЛОВ провален: потеряны припасы и уверенность."
    } else {
        state.tribe.add_knowledge(2);
        "Бросок ЛОВ выдержан: племя выбралось и стало осторожнее."
    };
    state.tribe.log.push(text);
}

fn apply_percent_food(tribe: &mut Tribe, percent: i8) {
    let delta = tribe.food.saturating_mul(percent as i32) / 100;
    tribe.food += delta;
}

fn apply_percent_knowledge(tribe: &mut Tribe, percent: i8) {
    if percent >= 0 {
        let delta = tribe.knowledge.saturating_mul(percent as u32) / 100;
        tribe.knowledge = tribe.knowledge.saturating_add(delta);
    } else {
        let abs_delta = tribe.knowledge.saturating_mul((-percent) as u32) / 100;
        tribe.knowledge = tribe.knowledge.saturating_sub(abs_delta);
    }
}

fn handle_turn_result(state: &mut GameState, result: TurnResult) {
    match result {
        TurnResult::Ok => {}
        TurnResult::Famine => {
            show_notice("Голод ударил по племени. Часть людей не пережила этот ход.")
        }
        TurnResult::Guests => {
            if state.tribe.has_flag(super::tribe::flags::РЕПУТАЦИЯ_ДУХОВ) {
                state.tribe.add_food(4);
                state.tribe.add_knowledge(4);
                state
                    .tribe
                    .log
                    .push("Гости решили не злить духов леса и оставили дары у костра.");
                show_notice(
                    "К лагерю пришли гости, но репутация духов сработала: оставили еду, слухи и ушли мирно.",
                );
            } else if state.tribe.authority >= 2 {
                state.tribe.add_food(3);
                state.tribe.add_knowledge(5);
                state.tribe.log.push(
                    "Вождь удержал разговор в рамках торговли. Племя получило немного пользы.",
                );
                show_notice(
                    "Пришли гости. Вождь удержал разговор в рамках обмена: немного еды и знаний удалось выторговать.",
                );
            } else {
                state.tribe.add_food(-3);
                state.tribe.morale = (state.tribe.morale - 1).clamp(-10, 10);
                state.tribe.pending_conflict_in = state.tribe.pending_conflict_in.max(2);
                state
                    .tribe
                    .log
                    .push("Гости поели, осмотрелись и ушли недовольными. Теперь ждём ультиматума.");
                show_notice(
                    "Гости оказались не мирными: съели часть запасов, всё приметили и пообещали вернуться уже с требованиями.",
                );
            }
        }
        TurnResult::Conflict => {
            if state.tribe.has_flag(super::tribe::flags::ПРЕДУПРЕЖДЕНИЕ)
                || state.tribe.has_flag(super::tribe::flags::ПЕРВЫЙ_ВОЛК)
            {
                state.tribe.add_knowledge(3);
                state.tribe.authority = (state.tribe.authority + 1).clamp(-10, 10);
                state.tribe.log.push("К ультиматуму были готовы заранее: разговор вышел жёстким, но не катастрофическим.");
                show_notice(
                    "Пришёл ультиматум, но лагерь уже ждал гостей. Удалось выиграть время и не потерять лицо.",
                );
            } else if state.tribe.food >= 6 {
                state.tribe.add_food(-6);
                state.tribe.authority = (state.tribe.authority - 1).clamp(-10, 10);
                state.tribe.log.push(
                    "Племя откупилось запасами. Войны сегодня не будет, но слабость увидели все.",
                );
                show_notice(
                    "Чужие предъявили ультиматум. Пришлось откупиться запасами, чтобы не довести дело до крови.",
                );
            } else {
                state.tribe.morale = (state.tribe.morale - 2).clamp(-10, 10);
                state.tribe.pending_raid_in = state.tribe.pending_raid_in.max(1);
                state.tribe.log.push("Ультиматум отвергли или не смогли оплатить. Теперь дело идёт к открытому набегу.");
                show_notice(
                    "Чужие пришли с ультиматумом. Договориться не вышло: они уйдут собирать людей для удара.",
                );
            }
        }
        TurnResult::Raid => {
            let mut losses = state.rng.roll_1d3() as u32;
            if state.tribe.has_flag(super::tribe::flags::ПРЕДУПРЕЖДЕНИЕ) {
                losses = losses.saturating_sub(1);
            }
            state.tribe.lose_population(losses);
            state.tribe.add_food(-6);
            state.tribe.morale = (state.tribe.morale - 2).clamp(-10, 10);
            state
                .tribe
                .log
                .push("Набег! Враги ударили по лагерю и унесли часть запасов.");
            show_notice("Набег! Лагерь потрёпан, люди потеряны, запасы разграблены.");
        }
        TurnResult::Extinction => {}
    }
}

fn maybe_system_event(state: &mut GameState) {
    let roll = state.rng.roll_2d6();
    let category = match roll {
        8..=9 => Some(0),
        10..=11 => Some(1),
        12 => Some(2),
        _ => None,
    };

    if let Some(category) = category {
        let event = db::получить_системное(category, state.rng.roll_1d6_minus1());
        show_notice(event.текст);
        apply_effects(state, &event.эфф);
        state.tribe.log.push(event.текст);
    }
}

fn writing_cost_left(tribe: &Tribe) -> u32 {
    let mut cost = 200u32;
    if tribe.has_flag(super::tribe::flags::ПЕРВОЕ_ПИСЬМО) {
        cost = cost * 7 / 10;
    }
    if tribe.has_flag(super::tribe::flags::ПИСЬМО_30) {
        cost = cost * 7 / 10;
    }
    if tribe.has_flag(super::tribe::flags::СЧЁТ) {
        cost = cost * 9 / 10;
    }
    if tribe.has_flag(super::tribe::flags::ЛЕТОПИСЬ) {
        cost = cost * 9 / 10;
    }
    if tribe.has_flag(super::tribe::flags::ЯЗЫК_РАЗВИВАЕТСЯ) {
        cost = cost * 9 / 10;
    }
    cost.saturating_sub(tribe.knowledge)
}

fn show_creation(tribe: &Tribe) {
    print_tribe_creation(
        tribe.attrs.strength,
        tribe.attrs.dexterity,
        tribe.attrs.intelligence,
        tribe.attrs.health,
        tribe.max_ap(),
    );
}

fn show_history_screen(state: &GameState) {
    clear_screen();
    print_separator(print::ЦВЕТ_РАМКА);
    print::println("  ИСТОРИЯ ПЛЕМЕНИ", print::ЦВЕТ_ЗАГОЛОВОК);
    print_separator(print::ЦВЕТ_РАМКА);
    print_recent_history(&state.tribe, 12);
    print::println("", print::ЦВЕТ_ОБЫЧНЫЙ);
    print::println("  [Любая клавиша] назад", print::ЦВЕТ_ПОДСКАЗКА);
    wait_any_key();
}

fn print_recent_history(tribe: &Tribe, count: usize) {
    let mut had_any = false;
    for entry in tribe.log.get_recent(count) {
        let mut empty = true;
        for &cp in entry.iter() {
            if cp != 0 {
                empty = false;
                break;
            }
        }
        if empty {
            continue;
        }
        had_any = true;
        print::print("  * ", print::ЦВЕТ_ПОДСКАЗКА);
        for &cp in entry.iter() {
            if cp == 0 {
                break;
            }
            if let Some(ch) = char::from_u32(cp) {
                print::print_char_adv(ch, print::ЦВЕТ_ОБЫЧНЫЙ);
            }
        }
        print::println("", print::ЦВЕТ_ОБЫЧНЫЙ);
    }

    if !had_any {
        print::println("  * Пока ничего не произошло.", print::ЦВЕТ_ПОДСКАЗКА);
    }
}

fn tribe_name(tribe: &Tribe) -> &str {
    let end = tribe
        .name
        .iter()
        .rposition(|&b| b != b' ')
        .map(|idx| idx + 1)
        .unwrap_or(0);
    core::str::from_utf8(&tribe.name[..end]).unwrap_or("Племя")
}

fn show_notice(text: &str) {
    clear_screen();
    print_event_text(text);
    wait_any_key();
}

fn show_game_over_screen(tribe: &Tribe) {
    print_game_over(tribe_name(tribe), tribe.turn);
    wait_any_key();
}

fn wait_for_enter() -> bool {
    loop {
        match next_keydown() {
            Some(KeyEvent::Enter) => return true,
            Some(KeyEvent::Esc) => return false,
            Some(_) => {}
            None => hlt(),
        }
    }
}

fn wait_any_key() {
    loop {
        if next_keydown().is_some() {
            return;
        }
        hlt();
    }
}

fn next_keydown() -> Option<KeyEvent> {
    if watchdog::timed_out() {
        crate::serial_println!("[tribe] idle timeout ignored; waiting for input");
        watchdog::pet();
        return None;
    }

    input::poll();
    let key = input::pop_keydown();
    if key.is_some() {
        watchdog::pet();
    }
    key
}
