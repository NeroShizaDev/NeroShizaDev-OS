use super::game::GameState;
use super::gurps::Outcome;
use super::tribe::flags;

pub(super) fn adjust_action_target(state: &GameState, action: usize, stupid: bool, base: u8) -> u8 {
    let mut target = base as i16;

    if state.tribe.has_flag(flags::РЕПУТАЦИЯ_ДУХОВ) {
        target += 1;
    }

    match action {
        0 => {
            if state.tribe.has_flag(flags::КУЛЬТ_БЕССМЕРТНОГО) {
                target -= if stupid { 1 } else { 2 };
            }
        }
        3 => {
            if state.tribe.has_flag(flags::ВРАГИ_ЗНАЮТ) {
                target -= 2;
            }
            if state.tribe.has_flag(flags::ОКРУЖЕНЫ) {
                target -= if stupid { 2 } else { 4 };
            }
            if state.tribe.has_flag(flags::ПЕРВЫЙ_ВОЛК) {
                target += 2;
            }
            if stupid && state.tribe.has_flag(flags::СОЮЗ_ТУПЫХ) {
                target += 1;
            }
        }
        _ => {}
    }

    target.clamp(3, 18) as u8
}

pub(super) fn after_action(state: &mut GameState, action: usize, stupid: bool, outcome: Outcome) {
    if action == 2 && state.tribe.has_flag(flags::ОБЩЕЕ_ВИДЕНИЕ) {
        state.tribe.add_knowledge(4);
        state.tribe.clear_flag(flags::ОБЩЕЕ_ВИДЕНИЕ);
        state
            .tribe
            .log
            .push("Общее видение ещё свежо: размышления сложились в ясную мысль.");
    }

    if action == 0 && state.tribe.has_flag(flags::КУЛЬТ_БЕССМЕРТНОГО) {
        match outcome {
            Outcome::CritFail => {
                state.hunt_penalty_turns = state.hunt_penalty_turns.max(2);
                state.tribe.morale = (state.tribe.morale - 1).clamp(-10, 10);
                state.tribe.authority = (state.tribe.authority - 1).clamp(-10, 10);
                state.tribe.log.push(
                    "Зверь снова ушёл невредимым. Культ требует прекратить охоту, вождю не верят.",
                );
            }
            Outcome::Fail => {
                state.hunt_penalty_turns = state.hunt_penalty_turns.max(1);
                state.tribe.morale = (state.tribe.morale - 1).clamp(-10, 10);
                state
                    .tribe
                    .log
                    .push("Культ бессмертного зверя сорвал охоту: охотники спорят, добыча уходит.");
            }
            _ => {}
        }
    }

    if action == 3 && state.tribe.has_flag(flags::ПЕРВЫЙ_ВОЛК) {
        match outcome {
            Outcome::CritSuccess => {
                state.tribe.add_knowledge(4);
                state.tribe.set_flag(flags::ПРЕДУПРЕЖДЕНИЕ);
                state
                    .tribe
                    .log
                    .push("Волчонок почуял опасность раньше людей: путь разведки стал яснее.");
            }
            Outcome::Success => {
                state.tribe.add_knowledge(2);
                state.tribe.log.push("Первый волк вывел разведчиков по запаху и шуму. Следов найдено больше обычного.");
            }
            _ => {}
        }
    }

    if action == 3 && state.tribe.has_flag(flags::ВРАГИ_ЗНАЮТ) {
        match outcome {
            Outcome::CritSuccess => {
                state.tribe.pending_raid_in = state.tribe.pending_raid_in.saturating_sub(1);
                state.tribe.add_knowledge(3);
                state
                    .tribe
                    .log
                    .push("Разведчики запутали следы: врагам стало сложнее ударить быстро.");
            }
            Outcome::CritFail => {
                state.tribe.pending_raid_in = state.tribe.pending_raid_in.max(2);
                state.tribe.morale = (state.tribe.morale - 1).clamp(-10, 10);
                state
                    .tribe
                    .log
                    .push("Враги не просто знают о племени, теперь они знают и привычки дозора.");
            }
            _ => {}
        }
    }

    if action == 3 && state.tribe.has_flag(flags::ОКРУЖЕНЫ) {
        match outcome {
            Outcome::Success | Outcome::CritSuccess => {
                state.tribe.add_knowledge(3);
                state
                    .tribe
                    .log
                    .push("Разведка нашла щель в кольце врагов: пусть узкую, но настоящую.");
            }
            Outcome::Fail | Outcome::CritFail => {
                state.tribe.pending_raid_in = state.tribe.pending_raid_in.max(1);
                state.tribe.morale = (state.tribe.morale - 1).clamp(-10, 10);
                state.tribe.log.push("Кольцо вокруг племени оказалось теснее, чем думали. Возвращаться стало тяжелее.");
            }
        }
    }

    if stupid && action == 3 && state.tribe.has_flag(flags::СОЮЗ_ТУПЫХ) {
        match outcome {
            Outcome::CritSuccess | Outcome::Success => {
                state.tribe.add_knowledge(2);
                state
                    .tribe
                    .log
                    .push("Союз тупых сработал нелепо, но полезно: чужие шумели не в ту сторону.");
            }
            Outcome::CritFail => {
                state.tribe.pending_raid_in = state.tribe.pending_raid_in.saturating_sub(1);
                state.tribe.morale = (state.tribe.morale + 1).clamp(-10, 10);
                state.tribe.log.push(
                    "Тупые союзники случайно отвлекли врагов. Провал выглядел как хитрый план.",
                );
            }
            Outcome::Fail => {}
        }
    }
}

pub(super) fn apply_turn_links(state: &mut GameState) {
    if state.rules_applied_turn == state.tribe.turn {
        return;
    }
    state.rules_applied_turn = state.tribe.turn;

    if state.tribe.has_flag(flags::ПРОКЛЯТЫЕ_ЛИСТЬЯ) {
        if state.tribe.knowledge >= 50 {
            state.tribe.clear_flag(flags::ПРОКЛЯТЫЕ_ЛИСТЬЯ);
            state
                .tribe
                .log
                .push("Шаман наконец понял, что проклятые листья можно просто выбросить.");
        } else if state.tribe.turn % 3 == 0 {
            state.tribe.morale = (state.tribe.morale - 1).clamp(-10, 10);
            state
                .tribe
                .log
                .push("Проклятые листья у входа снова портят настроение племени.");
        }
    }

    if state.tribe.has_flag(flags::СМЕНА_ВОЖДЯ) {
        state.tribe.authority = 0;
        state.tribe.clear_flag(flags::СМЕНА_ВОЖДЯ);
        state
            .tribe
            .log
            .push("Новый вождь сел у костра. Старый авторитет сгорел вместе с прошлым порядком.");
    }

    if state.tribe.has_flag(flags::ДОЛГ_КРОВИ) && state.tribe.turn % 4 == 0 {
        state.tribe.morale = (state.tribe.morale - 1).clamp(-10, 10);
        state.tribe.pending_raid_in = state.tribe.pending_raid_in.max(2);
        state
            .tribe
            .log
            .push("Долг крови не забыт: в лагере снова ждут шагов чужаков.");
    }

    if state.tribe.has_flag(flags::МЕДВЕДЬ_ПОМНИТ) && state.tribe.turn % 5 == 0 {
        state.hunt_penalty_turns = state.hunt_penalty_turns.max(1);
        state
            .tribe
            .log
            .push("Где-то рядом помнят старого медведя. На охоту идут без прежней уверенности.");
    }

    if state.tribe.has_flag(flags::НАС_ИЗУЧАЮТ) && state.tribe.food < 5 {
        state.tribe.authority = (state.tribe.authority - 1).clamp(-10, 10);
        state.tribe.pending_conflict_in = state.tribe.pending_conflict_in.max(2);
        state.tribe.log.push(
            "Соседи увидели слабость: еды мало, и скоро они придут уже не смотреть, а требовать.",
        );
    }

    if state.tribe.has_flag(flags::ВРАГИ_ЗНАЮТ) && state.tribe.turn % 6 == 0 {
        state.tribe.pending_conflict_in = state.tribe.pending_conflict_in.max(2);
        state.tribe.pending_raid_in = state.tribe.pending_raid_in.max(4);
        state.tribe.log.push(
            "Враги знают, где стоит лагерь. Сначала придут с требованиями, потом уже с копьями.",
        );
    }

    if state.tribe.has_flag(flags::ОКРУЖЕНЫ) {
        state.ap_penalty_turns = state.ap_penalty_turns.max(1);
        state.tribe.pending_conflict_in = state.tribe.pending_conflict_in.max(3);
        state
            .tribe
            .log
            .push("Племя живёт в кольце угроз: на этот ход приказов меньше.");
    }

    if state.tribe.has_flag(flags::СОЮЗ_ТУПЫХ)
        && state.tribe.pending_raid_in > 1
        && state.tribe.turn % 5 == 0
    {
        state.tribe.pending_raid_in = state.tribe.pending_raid_in.saturating_sub(1);
        state.tribe.log.push(
            "Союз тупых опять помог не тем способом, но вовремя: враги запутались в чужом шуме.",
        );
    }

    if state.tribe.has_flag(flags::ПЕРВЫЙ_ВОЛК)
        && (state.tribe.has_flag(flags::ВРАГИ_ЗНАЮТ) || state.tribe.has_flag(flags::ОКРУЖЕНЫ))
        && state.tribe.turn % 3 == 0
    {
        state.tribe.set_flag(flags::ПРЕДУПРЕЖДЕНИЕ);
        state.tribe.add_knowledge(1);
        state
            .tribe
            .log
            .push("Волчонок рычит раньше дозора: племя успевает чуть лучше понять угрозу.");
    }

    if state.tribe.food == 0 {
        state.tribe.morale = (state.tribe.morale - 1).clamp(-10, 10);
        state
            .tribe
            .log
            .push("Пустые запасы давят на племя: в лагере стало заметно тише.");
    }

    if state.tribe.morale <= -6 {
        state.tribe.authority = (state.tribe.authority - 1).clamp(-10, 10);
        state
            .tribe
            .log
            .push("Плохое настроение разъедает власть вождя: приказы слушают всё хуже.");
    }

    if state.tribe.morale >= 6 && state.tribe.has_flag(flags::ФИЛОСОФИЯ) {
        state.tribe.add_knowledge(2);
        state.tribe.log.push("Воодушевлённое племя спорит и учится быстрее: философия понемногу приносит новое знание.");
    }
}
