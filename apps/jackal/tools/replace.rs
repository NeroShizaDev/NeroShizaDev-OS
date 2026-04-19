// ============================================================
// REPLACE — Rust-версия replace_text.py
// ============================================================
// Python-оригинал: apps/jackal/scripts/replace_text.py
//
// Поиск и замена подстрок в байтовом срезе.
// В Python-скрипте входные правила берутся из JSON-файла.
// В ядре: передаём массив Rule структур напрямую.
//
// Пример:
//   let rules = &[
//       Rule { find: b"hello", replace: b"world" },
//       Rule { find: b"2024",  replace: b"2025" },
//   ];
//   let n = replace_all(src, dst, rules);
// ============================================================

#![allow(dead_code)]

/// Одно правило замены (аналог элемента JSON-массива в replace_text.py)
pub struct Rule<'a> {
    pub find:    &'a [u8],
    pub replace: &'a [u8],
}

/// Выполняет все правила замены по очереди.
/// `src` — исходные данные, `dst` — выходной буфер.
/// Возвращает количество байт записанных в `dst` и количество замен.
pub fn replace_all(src: &[u8], dst: &mut [u8], rules: &[Rule]) -> (usize, u32) {
    if rules.is_empty() {
        let n = src.len().min(dst.len());
        dst[..n].copy_from_slice(&src[..n]);
        return (n, 0);
    }

    // Для каждого правила применяем replace_one, используя ping-pong буфер.
    // Поскольку в ядре нет heap — используем единственный вспомогательный буфер.
    // Ограничение: суммарный размер не должен превышать dst.len().
    let mut total_replacements = 0u32;

    // Первый проход — пишем в dst
    let (written, reps) = replace_one(src, dst, &rules[0]);
    total_replacements += reps;

    if rules.len() == 1 {
        return (written, total_replacements);
    }

    // Для последующих правил нужен промежуточный буфер.
    // Ограничение in-kernel: статический буфер 8КБ.
    let mut aux: [u8; 8192] = [0u8; 8192];
    let mut current_len = written;

    for rule in &rules[1..] {
        // src → aux (используем dst[..current_len] как источник)
        let src_slice = &dst[..current_len];
        let (n, reps) = replace_one(src_slice, &mut aux, rule);
        total_replacements += reps;

        // aux → dst
        let copy = n.min(dst.len());
        dst[..copy].copy_from_slice(&aux[..copy]);
        current_len = copy;
    }

    (current_len, total_replacements)
}

/// Однократный проход замены одного паттерна.
fn replace_one(src: &[u8], dst: &mut [u8], rule: &Rule) -> (usize, u32) {
    let find    = rule.find;
    let replace = rule.replace;

    if find.is_empty() {
        let n = src.len().min(dst.len());
        dst[..n].copy_from_slice(&src[..n]);
        return (n, 0);
    }

    let mut src_pos = 0usize;
    let mut dst_pos = 0usize;
    let mut reps    = 0u32;

    while src_pos < src.len() && dst_pos < dst.len() {
        // Ищем совпадение начиная с src_pos
        if src_pos + find.len() <= src.len()
            && &src[src_pos..src_pos + find.len()] == find
        {
            // Копируем replace
            let copy = replace.len().min(dst.len() - dst_pos);
            dst[dst_pos..dst_pos + copy].copy_from_slice(&replace[..copy]);
            dst_pos += copy;
            src_pos += find.len();
            reps += 1;
        } else {
            dst[dst_pos] = src[src_pos];
            dst_pos += 1;
            src_pos += 1;
        }
    }

    (dst_pos, reps)
}

// ============================================================
// Удобная функция: поиск позиций паттерна в данных
// ============================================================
/// Аналог Python: [m.start() for m in re.finditer(pattern, text)]
/// Записывает до `max_hits` позиций в `out`. Возвращает количество найденных.
pub fn find_all(data: &[u8], pattern: &[u8], out: &mut [usize], max_hits: usize) -> usize {
    if pattern.is_empty() || data.len() < pattern.len() { return 0; }

    let mut count = 0usize;
    let mut i = 0usize;

    while i + pattern.len() <= data.len() && count < max_hits {
        if &data[i..i + pattern.len()] == pattern {
            out[count] = i;
            count += 1;
            i += pattern.len();
        } else {
            i += 1;
        }
    }

    count
}

