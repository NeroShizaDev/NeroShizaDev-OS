/// Конвертирует char в u32 кодпоинт.
/// Тривиально: в Rust char уже IS Unicode scalar value.
#[inline(always)]
pub fn char_to_codepoint(c: char) -> u32 {
    c as u32
}

/// К какому блоку Unicode принадлежит кодпоинт.
/// Теперь использует ПОЛНУЮ таблицу из 346 блоков Unicode 17.0.
/// Бинарный поиск — O(log 346) ≈ 9 сравнений.
pub fn unicode_block_name(cp: u32) -> &'static str {
    match crate::unicode_blocks::find_block(cp) {
        Some(block) => block.name,
        None => "No Block",
    }
}

/// К какому скрипту (системе письменности) принадлежит кодпоинт.
/// 174 скрипта Unicode 17.0 — все языки человечества.
pub fn unicode_script_name(cp: u32) -> &'static str {
    crate::unicode_scripts::find_script(cp).name()
}

/// General Category кодпоинта (буква, цифра, пунктуация, etc.)
pub fn unicode_category(cp: u32) -> crate::unicode_categories::GeneralCategory {
    crate::unicode_categories::find_category(cp)
}

/// Это буква (любого языка)?
pub fn is_letter(cp: u32) -> bool {
    crate::unicode_categories::find_category(cp).is_letter()
}

/// Это цифра (любой системы счисления)?
pub fn is_number(cp: u32) -> bool {
    crate::unicode_categories::find_category(cp).is_number()
}
