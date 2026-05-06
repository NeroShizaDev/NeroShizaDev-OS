use super::commands::handle_shell_commands;
use super::state::{
    ALT_HELD, CTRL_HELD, HIST_NAV, SCROLL_MODE, SCROLL_OFFSET, SHELL, SHIFT_HELD, redraw_input,
    reset_buffer,
};
use crate::print;
use crate::{kernel_messages, locale, unicode};
use core::sync::atomic::Ordering;

enum FKeyMessage {
    Saved(usize),
    RecordPrompt,
    Empty,
}

fn print_fkey_message(slot: usize, message: FKeyMessage) {
    match (locale::get_locale(), message) {
        (kernel_messages::Locale::RuRu, FKeyMessage::Saved(len)) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: Принято! ({} симв.)]", slot + 1, len),
        ),
        (kernel_messages::Locale::EnUs, FKeyMessage::Saved(len)) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: Saved! ({} chars)]", slot + 1, len),
        ),
        (kernel_messages::Locale::ArEg, FKeyMessage::Saved(len)) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: تم الحفظ! ({} رمز)]", slot + 1, len),
        ),
        (kernel_messages::Locale::RuRu, FKeyMessage::RecordPrompt) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: Введи команду, потом Alt+F{}]", slot + 1, slot + 1),
        ),
        (kernel_messages::Locale::EnUs, FKeyMessage::RecordPrompt) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: Type command, then Alt+F{}]", slot + 1, slot + 1),
        ),
        (kernel_messages::Locale::ArEg, FKeyMessage::RecordPrompt) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: اكتب الأمر ثم Alt+F{}]", slot + 1, slot + 1),
        ),
        (kernel_messages::Locale::RuRu, FKeyMessage::Empty) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: Пусто. Alt+F{} для записи]", slot + 1, slot + 1),
        ),
        (kernel_messages::Locale::EnUs, FKeyMessage::Empty) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: Empty. Alt+F{} to record]", slot + 1, slot + 1),
        ),
        (kernel_messages::Locale::ArEg, FKeyMessage::Empty) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: فارغ. Alt+F{} للتسجيل]", slot + 1, slot + 1),
        ),
    }
}

// ============================================================
// УДАЛЕНИЕ ВЫДЕЛЕНИЯ
// ============================================================
fn delete_selection() {
    // SAFETY: чтение/запись глобальных статиков только из обработчика прерывания (однопоточно).
    unsafe {
        if !SHELL.sel_active {
            return;
        }
        let start = SHELL.sel_start;
        let end = SHELL.sel_end;
        let _idx2 = SHELL.buffer_len; // локальная копия для debug_assert (Rust 2024: no &static_mut)
        debug_assert!(
            start <= end,
            "invalid selection: start={} end={}",
            start,
            end
        );
        debug_assert!(
            end <= _idx2,
            "selection end past INDEX: end={} index={}",
            end,
            _idx2
        );
        let len = end - start;
        if len == 0 {
            SHELL.sel_active = false;
            return;
        }

        remove_buffer_range(start, len);
        SHELL.cursor = start;
        SHELL.sel_active = false;
    }
}

// ============================================================
// CLIPBOARD: copy / paste / cut / select_all
// ============================================================
fn do_copy() {
    unsafe {
        if SHELL.sel_active && SHELL.sel_end > SHELL.sel_start {
            let len = SHELL.sel_end - SHELL.sel_start;
            copy_buffer_range_to(&raw mut SHELL.clipboard, SHELL.sel_start, len);
            SHELL.clipboard_len = len;
        } else if SHELL.buffer_len > 0 {
            copy_buffer_range_to(&raw mut SHELL.clipboard, 0, SHELL.buffer_len);
            SHELL.clipboard_len = SHELL.buffer_len;
        }
    }
}

fn do_paste() {
    // SAFETY: однопоточный доступ из обработчика прерывания клавиатуры.
    unsafe {
        if SHELL.clipboard_len == 0 {
            return;
        }
        let _cl = SHELL.clipboard_len;
        let _idx3 = SHELL.buffer_len;
        let _cur3 = SHELL.cursor;
        debug_assert!(_cl <= 64, "CLIP_LEN overflow: {}", _cl);
        debug_assert!(_idx3 <= 63, "INDEX overflow: {}", _idx3);
        debug_assert!(
            _cur3 <= _idx3,
            "CURSOR past INDEX: cursor={} index={}",
            _cur3,
            _idx3
        );

        if SHELL.sel_active {
            delete_selection();
        }

        let space = 63 - SHELL.buffer_len;
        let paste_len = SHELL.clipboard_len.min(space);
        if paste_len == 0 {
            return;
        }

        shift_buffer_right_from_cursor(paste_len);
        for i in 0..paste_len {
            SHELL.buffer[SHELL.cursor + i] = SHELL.clipboard[i];
        }
        SHELL.buffer_len += paste_len;
        SHELL.cursor += paste_len;
        redraw_input();
    }
}

fn do_cut() {
    unsafe {
        if SHELL.sel_active && SHELL.sel_end > SHELL.sel_start {
            do_copy();
            delete_selection();
            redraw_input();
        }
    }
}

fn do_select_all() {
    unsafe {
        if SHELL.buffer_len > 0 {
            SHELL.sel_active = true;
            SHELL.sel_start = 0;
            SHELL.sel_end = SHELL.buffer_len;
            redraw_input();
        }
    }
}

fn copy_buffer_range_to(dst: *mut [u32; 64], start: usize, len: usize) {
    unsafe {
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!(SHELL.buffer[start]),
            (&mut *dst).as_mut_ptr(),
            len,
        );
    }
}

fn shift_buffer_right_from_cursor(count: usize) {
    unsafe {
        core::ptr::copy(
            core::ptr::addr_of!(SHELL.buffer[SHELL.cursor]),
            core::ptr::addr_of_mut!(SHELL.buffer[SHELL.cursor + count]),
            SHELL.buffer_len - SHELL.cursor,
        );
    }
}

fn remove_buffer_range(start: usize, len: usize) {
    unsafe {
        if len == 0 {
            return;
        }
        core::ptr::copy(
            core::ptr::addr_of!(SHELL.buffer[start + len]),
            core::ptr::addr_of_mut!(SHELL.buffer[start]),
            SHELL.buffer_len - start - len,
        );
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!(SHELL.buffer[SHELL.buffer_len - len]),
            0,
            len,
        );
        SHELL.buffer_len -= len;
    }
}

fn delete_char_before_cursor() {
    unsafe {
        if SHELL.cursor == 0 {
            return;
        }
        SHELL.cursor -= 1;
        remove_buffer_range(SHELL.cursor, 1);
    }
}

fn delete_char_at_cursor() {
    unsafe {
        if SHELL.cursor >= SHELL.buffer_len {
            return;
        }
        remove_buffer_range(SHELL.cursor, 1);
    }
}

fn insert_codepoint_at_cursor(cp: u32) {
    unsafe {
        shift_buffer_right_from_cursor(1);
        SHELL.buffer[SHELL.cursor] = cp;
        SHELL.buffer_len += 1;
        SHELL.cursor += 1;
    }
}

fn save_buffer_to_history_slot(slot: usize) {
    unsafe {
        let len = SHELL.buffer_len;
        for i in 0..len {
            SHELL.history[slot][i] = SHELL.buffer[i];
        }
        for i in len..64 {
            SHELL.history[slot][i] = 0;
        }
        SHELL.history_lens[slot] = len;
    }
}

fn restore_buffer_from_history_slot(slot: usize) {
    unsafe {
        let len = SHELL.history_lens[slot];
        for i in 0..len {
            SHELL.buffer[i] = core::ptr::read_volatile(&raw const SHELL.history[slot][i]);
        }
        for i in len..64 {
            SHELL.buffer[i] = 0;
        }
        SHELL.buffer_len = len;
        SHELL.cursor = len;
    }
}

// ============================================================
// ИСТОРИЯ КОМАНД
// ============================================================
fn push_history() {
    unsafe {
        if SHELL.buffer_len == 0 {
            return;
        }
        let idx = SHELL.history_idx;
        save_buffer_to_history_slot(idx);
        SHELL.history_idx = (idx + 1) % 32;
        if SHELL.history_count < 32 {
            SHELL.history_count += 1;
        }
        SHELL.history_nav = -1;
    }
}

fn history_up() {
    unsafe {
        if SHELL.history_count == 0 {
            return;
        }
        if SHELL.history_nav == -1 {
            // Сохраняем текущий ввод в history[32]
            save_buffer_to_history_slot(32);
            SHELL.cursor = SHELL.buffer_len;
            SHELL.history_nav = 0;
        } else {
            // Переходим к предыдущей команде в истории
            SHELL.history_nav += 1;
            if SHELL.history_nav as usize >= SHELL.history_count {
                // ничего не делаем
            }
            let ring_idx = (SHELL.history_idx + 32 - 1 - SHELL.history_nav as usize) % 32;
            restore_buffer_from_history_slot(ring_idx);
        }
        SHELL.sel_active = false;
        redraw_input();
    }
}

fn history_down() {
    unsafe {
        if SHELL.history_nav < 0 {
            return;
        }
        SHELL.history_nav -= 1;
        if SHELL.history_nav < 0 {
            // Восстанавливаем сохранённый ввод из history[32]
            restore_buffer_from_history_slot(32);
        } else {
            let ring_idx = (SHELL.history_idx + 32 - 1 - SHELL.history_nav as usize) % 32;
            restore_buffer_from_history_slot(ring_idx);
        }
        SHELL.sel_active = false;
        redraw_input();
    }
}

// ============================================================
// СКРОЛЛБЭК — PageUp/PageDown
// ============================================================
fn enter_scroll_mode() {
    unsafe {
        if SCROLL_MODE {
            return;
        }
        let total = crate::fb_buffer::scroll_total();
        if total == 0 {
            return;
        }
        // Только снимок экрана для восстановления по Esc.
        // В кольцо НЕ пишем — строки там уже есть от new_line().
        crate::fb_buffer::save_screen_snapshot();
        SCROLL_MODE = true;
        SCROLL_OFFSET = total.min(12);
        crate::fb_buffer::show_scrollback(SCROLL_OFFSET);
    }
}

fn scroll_page_up() {
    unsafe {
        let total = crate::fb_buffer::scroll_total();
        if SCROLL_OFFSET + 12 < total {
            SCROLL_OFFSET += 12;
        } else if total > 0 {
            SCROLL_OFFSET = total - 1;
        }
        crate::fb_buffer::show_scrollback(SCROLL_OFFSET);
    }
}

fn scroll_page_down() {
    unsafe {
        if SCROLL_OFFSET >= 12 {
            SCROLL_OFFSET -= 12;
            crate::fb_buffer::show_scrollback(SCROLL_OFFSET);
        } else {
            exit_scroll_mode();
        }
    }
}

fn exit_scroll_mode() {
    unsafe {
        if !SCROLL_MODE {
            return;
        }
        SCROLL_MODE = false;
        // Восстанавливаем экран из снимка (не из scrollback — чтобы вернуть точный вид)
        crate::fb_buffer::restore_saved_screen();
        redraw_input();
    }
}

fn handle_fkey(slot: usize) {
    debug_assert!(slot < 12, "F-key slot out of range: {}", slot);
    unsafe {
        if ALT_HELD.load(Ordering::Acquire) {
            if let Some(s) = SHELL.recording_slot {
                if s == slot {
                    let len = SHELL.buffer_len.min(64);
                    copy_buffer_range_to(&raw mut SHELL.hotkeys[slot], 0, len);
                    core::ptr::write_volatile(&raw mut SHELL.hotkey_lens[slot], len);
                    SHELL.recording_slot = None;
                    locale::print_localized_line("", 0x0E);
                    print_fkey_message(slot, FKeyMessage::Saved(len));
                    reset_buffer();
                    print!("> ");
                    return;
                }
            }
            SHELL.recording_slot = Some(slot);
            reset_buffer();
            locale::print_localized_line("", 0x0E);
            print_fkey_message(slot, FKeyMessage::RecordPrompt);
            print!("F{}> ", slot + 1);
        } else {
            let rec = core::ptr::read_volatile(&raw const SHELL.recording_slot);
            if rec.is_some() {
                return;
            }
            let len = core::ptr::read_volatile(&raw const SHELL.hotkey_lens[slot]);
            if len == 0 {
                locale::print_localized_line("", 0x0E);
                print_fkey_message(slot, FKeyMessage::Empty);
                print!("> ");
                return;
            }
            for i in 0..len {
                let val = core::ptr::read_volatile(&raw const SHELL.hotkeys[slot][i]);
                SHELL.buffer[i] = val;
            }
            SHELL.buffer_len = len;
            SHELL.cursor = len;
            print!("[F{}] ", slot + 1);
            handle_keyboard_input('\n');
        }
    }
}

// ============================================================
// ОБРАБОТКА RawKey (CapsLock, ScrollLock, PauseBreak, стрелки, Home/End)
// ============================================================
pub fn handle_raw_key(key: pc_keyboard::KeyCode) {
    use pc_keyboard::KeyCode;
    unsafe {
        // Скроллбэк: только PgUp/PgDn, остальное — выход
        if SCROLL_MODE {
            match key {
                KeyCode::PageUp => {
                    scroll_page_up();
                    return;
                }
                KeyCode::PageDown => {
                    scroll_page_down();
                    return;
                }
                _ => {
                    exit_scroll_mode();
                    return;
                }
            }
        }

        match key {
            KeyCode::CapsLock => {
                if SHELL.buffer_len > 0 && !SHELL.confirm_pending {
                    SHELL.confirm_pending = true;
                    locale::print_localized_line("", 0x0E);
                    locale::render_event_auto(kernel_messages::KernelEvent::ShellConfirmPrompt);
                }
            }
            KeyCode::ScrollLock => {
                SHELL.lang_rus = !SHELL.lang_rus;
                redraw_input();
            }
            KeyCode::PauseBreak => {
                if SHELL.confirm_pending {
                    SHELL.confirm_pending = false;
                    locale::render_event_auto(kernel_messages::KernelEvent::ShellCanceled);
                    reset_buffer();
                    print!("> ");
                }
            }
            // ← → стрелки + Shift-выделение
            KeyCode::ArrowLeft => {
                if SHELL.cursor > 0 {
                    if SHIFT_HELD.load(Ordering::Acquire) {
                        if !SHELL.sel_active {
                            SHELL.sel_active = true;
                            SHELL.sel_start = SHELL.cursor - 1;
                            SHELL.sel_end = SHELL.cursor;
                        } else if SHELL.sel_start == SHELL.cursor {
                            SHELL.sel_start = SHELL.cursor - 1;
                        } else if SHELL.sel_end == SHELL.cursor {
                            SHELL.sel_end = SHELL.cursor - 1;
                            if SHELL.sel_start == SHELL.sel_end {
                                SHELL.sel_active = false;
                            }
                        }
                    } else {
                        SHELL.sel_active = false;
                    }
                    SHELL.cursor -= 1;
                    redraw_input();
                }
            }
            KeyCode::ArrowRight => {
                if SHELL.cursor < SHELL.buffer_len {
                    if SHIFT_HELD.load(Ordering::Acquire) {
                        if !SHELL.sel_active {
                            SHELL.sel_active = true;
                            SHELL.sel_start = SHELL.cursor;
                            SHELL.sel_end = SHELL.cursor + 1;
                        } else if SHELL.sel_end == SHELL.cursor {
                            SHELL.sel_end = SHELL.cursor + 1;
                        } else if SHELL.sel_start == SHELL.cursor {
                            SHELL.sel_start = SHELL.cursor + 1;
                            if SHELL.sel_start == SHELL.sel_end {
                                SHELL.sel_active = false;
                            }
                        }
                    } else {
                        SHELL.sel_active = false;
                    }
                    SHELL.cursor += 1;
                    redraw_input();
                }
            }
            KeyCode::Home => {
                if SHELL.cursor > 0 {
                    if SHIFT_HELD.load(Ordering::Acquire) {
                        if !SHELL.sel_active {
                            SHELL.sel_active = true;
                            SHELL.sel_start = 0;
                            SHELL.sel_end = SHELL.cursor;
                        } else {
                            SHELL.sel_start = 0;
                        }
                    } else {
                        SHELL.sel_active = false;
                    }
                    SHELL.cursor = 0;
                    redraw_input();
                }
            }
            KeyCode::End => {
                if SHELL.cursor < SHELL.buffer_len {
                    if SHIFT_HELD.load(Ordering::Acquire) {
                        if !SHELL.sel_active {
                            SHELL.sel_active = true;
                            SHELL.sel_start = SHELL.cursor;
                            SHELL.sel_end = SHELL.buffer_len;
                        } else {
                            SHELL.sel_end = SHELL.buffer_len;
                        }
                    } else {
                        SHELL.sel_active = false;
                    }
                    SHELL.cursor = SHELL.buffer_len;
                    redraw_input();
                }
            }
            // ↑↓ история команд
            KeyCode::ArrowUp => {
                history_up();
            }
            KeyCode::ArrowDown => {
                history_down();
            }
            // PageUp/PageDown — прокрутка экрана
            KeyCode::PageUp => {
                enter_scroll_mode();
            }
            KeyCode::PageDown => {} // вне scroll mode — ничего
            // Delete — удалить символ ПОД курсором
            KeyCode::Delete => {
                if SHELL.sel_active {
                    delete_selection();
                    HIST_NAV = -1;
                    redraw_input();
                } else if SHELL.cursor < SHELL.buffer_len {
                    delete_char_at_cursor();
                    HIST_NAV = -1;
                    redraw_input();
                }
            }
            // F1-F12: Alt+Fn = запись, Fn = выполнение
            KeyCode::F1 => handle_fkey(0),
            KeyCode::F2 => handle_fkey(1),
            KeyCode::F3 => handle_fkey(2),
            KeyCode::F4 => handle_fkey(3),
            KeyCode::F5 => handle_fkey(4),
            KeyCode::F6 => handle_fkey(5),
            KeyCode::F7 => handle_fkey(6),
            KeyCode::F8 => handle_fkey(7),
            KeyCode::F9 => handle_fkey(8),
            KeyCode::F10 => handle_fkey(9),
            KeyCode::F11 => handle_fkey(10),
            KeyCode::F12 => handle_fkey(11),
            _ => {}
        }
    }
}

pub fn show_shell_prompt() {
    reset_buffer();
    print!("> ");
    redraw_input();
}

// ============================================================
// ГЛАВНАЯ ЛОГИКА — Unicode Intent Engine
// Ctrl+C/V/X/A, cursor-aware insert/delete, redraw_input
// ============================================================

pub fn handle_keyboard_input(c: char) {
    unsafe {
        // Выход из скроллбэка на любую клавишу
        if SCROLL_MODE {
            exit_scroll_mode();
            if c == '\x1B' {
                return;
            }
        }

        // === Ctrl+key: clipboard ===
        if CTRL_HELD.load(Ordering::Acquire) {
            match c {
                'c' => {
                    do_copy();
                    return;
                }
                'v' => {
                    do_paste();
                    return;
                }
                'x' => {
                    do_cut();
                    return;
                }
                'a' => {
                    do_select_all();
                    return;
                }
                'l' => {
                    crate::fb_buffer::clear_screen();
                    locale::draw_locale_badge();
                    print!("> ");
                    redraw_input();
                    return;
                }
                _ => {
                    return;
                }
            }
        }

        // === Escape: сброс буфера / отмена подтверждения ===
        if c == '\x1B' {
            if SHELL.confirm_pending {
                SHELL.confirm_pending = false;
                locale::render_event_auto(kernel_messages::KernelEvent::ShellCanceled);
            }
            reset_buffer();
            print!("> ");
            redraw_input();
            return;
        }

        if c == '\n' {
            if SHELL.confirm_pending {
                SHELL.confirm_pending = false;
            }
            SHELL.sel_active = false;
            locale::print_localized_line("", 0x0E);

            if SHELL.buffer_len > 0 {
                push_history();
                if handle_shell_commands() {
                    reset_buffer();
                    print!("> ");
                    redraw_input();
                    return;
                }
            }

            reset_buffer();
            print!("> ");
            redraw_input();
        } else if c == '\x08' {
            // Backspace
            HIST_NAV = -1;
            if SHELL.confirm_pending {
                SHELL.confirm_pending = false;
                locale::render_event_auto(kernel_messages::KernelEvent::ShellCanceled);
                reset_buffer();
                print!("> ");
                redraw_input();
            } else if SHELL.sel_active {
                // Удаляем выделение
                delete_selection();
                redraw_input();
            } else if SHELL.cursor > 0 {
                delete_char_before_cursor();
                redraw_input();
            }
        } else if SHELL.buffer_len < 63 {
            if SHELL.confirm_pending {
                return;
            }
            HIST_NAV = -1;

            // Удаляем выделение если есть
            if SHELL.sel_active {
                delete_selection();
            }

            // Применяем языковую раскладку
            let mapped = locale::map_shell_input_char(c, SHELL.lang_rus);
            let cp = unicode::char_to_codepoint(mapped);
            insert_codepoint_at_cursor(cp);
            redraw_input();
        }
    }
}
