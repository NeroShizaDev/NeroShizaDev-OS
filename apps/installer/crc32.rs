// ============================================================
// CRC32 — табличная реализация без heap
// ============================================================
// 256-entry lookup table (1KB) в .rodata — компилятор кладёт
// прямо в образ ядра, не нужен ни alloc, ни BSS.
//
// Полином: 0xEDB88320 (IEEE 802.3, тот же что в ZIP/PNG/PE).
// Аналог: Windows CRC32 из kernel32.dll, но без DLL.
// ============================================================

/// CRC32 lookup table (IEEE polynomial, reflected).
/// Предвычислена в compile-time через const fn.
static CRC32_TABLE: [u32; 256] = generate_table();

/// Генерирует CRC32 таблицу в compile-time.
const fn generate_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0u32;
    while i < 256 {
        let mut crc = i;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i as usize] = crc;
        i += 1;
    }
    table
}

/// Вычисляет CRC32 среза байтов.
///
/// Используется для верификации .nhs пакета при установке.
/// Аналог того, как Windows проверяет PE checksum.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        let idx = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLE[idx];
    }
    !crc
}

/// Проверяет CRC32: вычисляет и сравнивает с ожидаемым.
pub fn verify(data: &[u8], expected: u32) -> bool {
    crc32(data) == expected
}
