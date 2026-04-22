// ============================================================
// JACKAL SHELL COMMAND — вывод Report в консоль
// ============================================================
// Использование из shell.rs:
//   match cmd {
//       "jackal" | "jac" => jackal::shell::run_on_slice(data),
//       ...
//   }
//
// Пока модуль работает на &[u8] — позже, когда будет ramdisk или
// FAT12, появится `run_on_file(path: &str)`.
// ============================================================

#![allow(dead_code)]

use super::analyzer::{self, FileKind, Report, AUTOCORR_PERIODS};
use crate::apps::jackal::encoder;
use crate::apps::jackal::tools::archive;
use crate::apps::jackal::tools::validate;

// ============================================================
// Статические буферы для DEMO — ни байта на стеке
// ============================================================
// EncodedBlock = 256 КБ, Archive = 16×64 КБ = ~1 МБ.
// Всё это убивает ядровой стек — переносим в BSS.
static mut S_BLOCK:  encoder::EncodedBlock = encoder::EncodedBlock::new();
static mut S_JKL:    [u8; encoder::JKL_HEADER_SIZE + encoder::MAX_ENCODED]
                   = [0u8; encoder::JKL_HEADER_SIZE + encoder::MAX_ENCODED];
static mut S_ARCH:   archive::Archive = archive::Archive::new();
static mut S_UNPACK: archive::Archive = archive::Archive::new();
static mut S_PACKED: [u8; 8 * 1024] = [0u8; 8 * 1024]; // был на стеке → DF

/// Главный вход: принимает сырые байты, печатает отчёт.
pub fn run_on_slice(data: &[u8], label: &str) {
    crate::user_messages::print_jackal_header(label, data.len() as u64);

    let report = analyzer::analyze(data);

    print_magic(&report);
    print_stats(&report);
    print_block_profile(&report);
    print_autocorr(&report);
    print_classification(&report);
    print_voodoo_bridge(&report);

    crate::user_messages::print_jackal_footer();
}

// ============================================================
// Секции отчёта
// ============================================================

fn print_magic(report: &Report) {
    match report.magic {
        Some(m) => {
            // TECH: «[MAGIC] MZ @0x00 → Executable»
            // LORE: «Первая сигнатура узнана: MZ — это исполняемое тело.»
            crate::user_messages::print_jackal_magic(m.offset, m.name, m.kind_hint.short());
        }
        None => {
            crate::user_messages::print_jackal_no_magic();
        }
    }
}

fn print_stats(report: &Report) {
    let h_int = report.global_h_milli / 1000;
    let h_frac = report.global_h_milli % 1000;
    crate::user_messages::print_jackal_entropy(h_int, h_frac);
    crate::user_messages::print_jackal_histogram_stats(
        report.unique,
        report.printable_bp,
        report.zero_bp,
    );
}

fn print_block_profile(report: &Report) {
    if report.profile.block_count == 0 { return; }
    let (mn, mx) = report.profile.range();
    let trans = report.profile.transition_count();
    crate::user_messages::print_jackal_profile_summary(
        report.profile.block_count as u32,
        mn, mx, trans,
    );

    // Рисуем sparkline высотой 8 строк (ниже — плотнее)
    // Символы от «пусто» до «полно»: . , - + * # @ █
    // В кодовой базе возможно лучше через спец-глифы VGA,
    // но для простоты пока ASCII.
    let mut line: [u8; MAX_BLOCKS_VIEW] = [b' '; MAX_BLOCKS_VIEW];
    let shown = report.profile.block_count.min(MAX_BLOCKS_VIEW);
    for i in 0..shown {
        let v = report.profile.milli[i];
        // 0..8000 → 0..7 индекс
        let idx = (v / 1000).min(7) as usize;
        line[i] = BAR_CHARS[idx];
    }
    crate::user_messages::print_jackal_sparkline(&line[..shown]);
}

fn print_autocorr(report: &Report) {
    let (best_p, best_v) = report.autocorr.strongest();
    // best_v в basis points 0..10000. Для печати: %.2f = bp/100
    let pct_int = best_v / 100;
    let pct_frac = best_v % 100;
    crate::user_messages::print_jackal_autocorr_summary(
        best_p as u32, pct_int as u32, pct_frac as u32,
    );

    // Полная таблица периодов
    for (i, &p) in AUTOCORR_PERIODS.iter().enumerate() {
        let v = report.autocorr.match_bp[i];
        crate::user_messages::print_jackal_autocorr_row(
            p as u32, (v / 100) as u32, (v % 100) as u32,
        );
    }
}

fn print_classification(report: &Report) {
    let kind_name = match report.classification {
        FileKind::Text       => "TEXT",
        FileKind::Executable => "EXECUTABLE",
        FileKind::Compressed => "COMPRESSED",
        FileKind::Random     => "RANDOM",
        FileKind::Structured => "STRUCTURED",
        FileKind::LossyMedia => "LOSSY MEDIA",
        FileKind::Unknown    => "UNKNOWN",
    };
    crate::user_messages::print_jackal_classification(
        kind_name,
        report.confidence_milli,
    );
}

fn print_voodoo_bridge(report: &Report) {
    let priors = analyzer::to_voodoo_priors(report);
    // Печатаем 6 приоров в промилле (× 1000 → 0..1000)
    let milli: [u32; 6] = [
        (priors[0] * 1000.0) as u32,
        (priors[1] * 1000.0) as u32,
        (priors[2] * 1000.0) as u32,
        (priors[3] * 1000.0) as u32,
        (priors[4] * 1000.0) as u32,
        (priors[5] * 1000.0) as u32,
    ];
    crate::user_messages::print_jackal_voodoo_priors(&milli);
}

// ============================================================
// Константы вывода
// ============================================================
const MAX_BLOCKS_VIEW: usize = 64;
const BAR_CHARS: [u8; 8] = [b'.', b',', b'-', b'+', b'*', b'#', b'%', b'@'];

// ============================================================
// DEMO — запускается из APPS-меню по пункту «Jackal»
// ============================================================
// Анализирует встроенный тестовый образец и демонстрирует
// возможности analyzer + encoder прямо в VGA.
// Выход: любая клавиша.
// ============================================================

/// Тестовые данные: ASCII-текст + блок нулей + «магия» PNG
static DEMO_DATA: &[u8] = b"\
Hello NeroShizaDev-OS! This is a test slice for Jackal analyzer.\n\
The quick brown fox jumps over the lazy dog. 1234567890\n\
\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\
\x00\x00\x00\x00\x00\x00\x00\x00\x08\x02\x00\x00\x00\
\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\
";

pub fn run_demo() {
    crate::trace::record("jackal demo entered");
    crate::vga_buffer::clear_screen();
    crate::locale::print_localized_line("=== JACKAL ANALYZER DEMO ===", 0x0E);
    crate::locale::print_localized_line("Analysing built-in test slice...", 0x07);

    crate::trace::record("jackal run_on_slice start");
    run_on_slice(DEMO_DATA, "demo-slice");
    crate::trace::record("jackal run_on_slice done");

    crate::trace::record("jackal analyze for encode start");
    let report = analyzer::analyze(DEMO_DATA);
    crate::trace::record("jackal analyze for encode done");
    unsafe {
        crate::trace::record("jackal encode_into start");
        encoder::encode_into(DEMO_DATA, &report,
            &mut *core::ptr::addr_of_mut!(S_BLOCK));
        crate::trace::record("jackal encode_into done");
    }

    crate::locale::print_localized_line("", 0x07);
    unsafe { print_encoder_result(&*core::ptr::addr_of!(S_BLOCK)); }

    let (header_len, payload_len) = unsafe {
        crate::trace::record("jackal jkl header build start");
        let block = &*core::ptr::addr_of!(S_BLOCK);
        let jkl   = &mut *core::ptr::addr_of_mut!(S_JKL);
        let hl = encoder::write_jkl_header(jkl, block);
        let pl = block.encoded_size as usize;
        if hl > 0 && hl + pl <= jkl.len() {
            core::ptr::copy_nonoverlapping(
                block.buf.as_ptr(),
                jkl[hl..].as_mut_ptr(),
                pl,
            );
        }
        crate::trace::record("jackal jkl header build done");
        (hl, pl)
    };

    const JKL_MAX: usize = encoder::JKL_HEADER_SIZE + encoder::MAX_ENCODED;
    if header_len > 0 && header_len + payload_len <= JKL_MAX {
        crate::trace::record("jackal validate jkl start");
        let status = unsafe {
            let jkl = &*core::ptr::addr_of!(S_JKL);
            validate::validate(&jkl[..header_len + payload_len])
        };
        crate::trace::record("jackal validate jkl done");
        crate::locale::print_localized_fmt(0x0B, format_args!("[VALIDATE] {}", status.description()));
        unsafe {
            crate::trace::record("jackal archive roundtrip start");
            let jkl = &*core::ptr::addr_of!(S_JKL);
            demo_archive_roundtrip(&jkl[..header_len + payload_len]);
            crate::trace::record("jackal archive roundtrip done");
        }
    } else {
        crate::locale::print_localized_line("[VALIDATE] skipped: buffer overflow", 0x0C);
    }

    crate::locale::print_localized_line("", 0x07);
    crate::locale::print_localized_line("Press any key to return to APPS menu...", 0x08);
    crate::trace::record("jackal wait_key start");
    wait_key();
    crate::trace::record("jackal wait_key done");
}

unsafe fn demo_archive_roundtrip(jkl: &[u8]) {
    crate::trace::record("jackal arch reset start");
    let arch = &mut *core::ptr::addr_of_mut!(S_ARCH);
    arch.count = 0;
    crate::trace::record("jackal arch add blocks start");
    let ok_main = arch.add_jkl_block(b"demo.jkl", jkl);
    let ok_note = arch.add(b"note.txt", b"Jackal archive demo payload");
    crate::trace::record("jackal arch add blocks done");

    if !ok_main || !ok_note {
        crate::locale::print_localized_line("[ARCH] add block failed", 0x0C);
        return;
    }

    let packed_len = archive::pack(&*core::ptr::addr_of!(S_ARCH), &mut *core::ptr::addr_of_mut!(S_PACKED));
    crate::trace::record("jackal arch pack done");
    if packed_len == 0 {
        crate::locale::print_localized_line("[ARCH] pack failed", 0x0C);
        return;
    }

    let packed_slice = &(&(*core::ptr::addr_of!(S_PACKED)))[..packed_len];
    let status = validate::validate(packed_slice);
    crate::locale::print_localized_fmt(
        0x0B,
        format_args!("[ARCH] validate={} bytes={}", status.description(), packed_len),
    );

    match archive::unpack_into(packed_slice, &mut *core::ptr::addr_of_mut!(S_UNPACK)) {
        Ok(()) => {
            crate::trace::record("jackal arch unpack ok");
            let unpacked = &*core::ptr::addr_of!(S_UNPACK);
            crate::locale::print_localized_fmt(0x0A, format_args!("[ARCH] blocks={}", unpacked.count));
            for i in 0..unpacked.count {
                let b = &unpacked.blocks[i];
                crate::locale::print_localized_fmt(
                    0x07,
                    format_args!(
                        "[ARCH] {} alg={} raw={} enc={}",
                        b.name_str(), b.algorithm, b.original_size, b.data_len
                    ),
                );
            }
        }
        Err(e) => {
            crate::trace::record("jackal arch unpack err");
            crate::locale::print_localized_fmt(0x0C, format_args!("[ARCH] unpack failed: {}", e));
        }
    }
}

fn print_encoder_result(block: &encoder::EncodedBlock) {
    let alg_name = match block.algorithm {
        encoder::ALG_STORE => "STORE",
        encoder::ALG_RLE   => "RLE",
        encoder::ALG_DELTA => "DELTA",
        _                  => "?",
    };
    crate::locale::print_localized_line("--- ENCODER RESULT ---", 0x0B);
    crate::locale::print_localized_fmt(0x0F, format_args!("[ENC] Algorithm : {}", alg_name));
    crate::locale::print_localized_fmt(0x07, format_args!("[ENC] Original  : {} bytes", block.original_size));
    crate::locale::print_localized_fmt(0x07, format_args!("[ENC] Encoded   : {} bytes", block.encoded_size));
    let ratio_int  = block.ratio_milli / 10;
    let ratio_frac = block.ratio_milli % 10;
    crate::locale::print_localized_fmt(0x0A, format_args!("[ENC] Ratio     : {}.{}%", ratio_int, ratio_frac));
}

fn wait_key() {
    use x86_64::instructions::hlt;
    unsafe {
        loop {
            if crate::ps2::has_scancode() {
                let sc = crate::ps2::read_scancode();
                if sc & 0x80 == 0 { break; }
            }
            hlt();
        }
    }
}


