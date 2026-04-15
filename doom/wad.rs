// ============================================================
// WAD PARSER — no_std WAD-ридер для NeroShiza Doom
// ============================================================
// Формат WAD (id Software):
//   Header  (12 байт): magic[4] | num_lumps: i32 | dir_offset: i32
//   Dir     (16 байт × N): filepos: i32 | size: i32 | name: [u8; 8]
//
// Используем &'static [u8] — WAD подключается через include_bytes!
// или передаётся как ссылка на статические данные.
// alloc::vec::Vec — для хранения директории в памяти при необходимости.
// ============================================================

extern crate alloc;
use alloc::vec::Vec;

// ============================================================
// Структуры
// ============================================================

/// Тип WAD-файла
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WadType {
    IWad, // Internal WAD (doom1.wad, doom2.wad)
    PWad, // Patch WAD (уровни, моды)
}

/// Заголовок WAD-файла (12 байт)
#[derive(Debug, Clone, Copy)]
pub struct WadHeader {
    pub wad_type:   WadType,
    pub num_lumps:  u32,
    pub dir_offset: u32,
}

/// Запись в директории WAD (16 байт)
#[derive(Debug, Clone, Copy)]
pub struct LumpInfo {
    pub filepos: u32,
    pub size:    u32,
    pub name:    [u8; 8],
}

impl LumpInfo {
    /// Возвращает имя lump как ASCII-строку (без нулевых байтов)
    pub fn name_str(&self) -> &[u8] {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(8);
        &self.name[..end]
    }

    /// Сравнивает имя lump с ASCII-строкой (case-insensitive)
    pub fn name_matches(&self, s: &[u8]) -> bool {
        if s.len() > 8 { return false; }
        for (i, &b) in s.iter().enumerate() {
            let a = self.name[i].to_ascii_uppercase();
            let b = b.to_ascii_uppercase();
            if a != b { return false; }
        }
        // Остаток имени lump должен быть нулём или совпадать
        for i in s.len()..8 {
            if self.name[i] != 0 { return false; }
        }
        true
    }
}

// ============================================================
// WAD Reader
// ============================================================

/// Разбирает WAD-данные из сырого байтового среза.
pub struct WadReader {
    data:   &'static [u8],
    header: WadHeader,
    lumps:  Vec<LumpInfo>,
}

impl WadReader {
    /// Создаёт WadReader из &'static [u8].
    /// Возвращает None если данные не являются валидным WAD.
    pub fn new(data: &'static [u8]) -> Option<WadReader> {
        if data.len() < 12 {
            return None;
        }

        // Парсим magic (4 байта)
        let wad_type = match &data[0..4] {
            b"IWAD" => WadType::IWad,
            b"PWAD" => WadType::PWad,
            _       => return None,
        };

        let num_lumps  = read_u32_le(data, 4);
        let dir_offset = read_u32_le(data, 8);

        // Проверяем что директория помещается в файл
        let dir_end = dir_offset as usize + num_lumps as usize * 16;
        if dir_end > data.len() {
            return None;
        }

        let header = WadHeader { wad_type, num_lumps, dir_offset };

        // Читаем директорию в Vec
        let mut lumps = Vec::with_capacity(num_lumps as usize);
        for i in 0..num_lumps as usize {
            let base = dir_offset as usize + i * 16;
            let filepos = read_u32_le(data, base);
            let size    = read_u32_le(data, base + 4);

            let mut name = [0u8; 8];
            name.copy_from_slice(&data[base + 8..base + 16]);

            lumps.push(LumpInfo { filepos, size, name });
        }

        Some(WadReader { data, header, lumps })
    }

    /// Тип WAD (IWAD / PWAD)
    pub fn wad_type(&self) -> WadType { self.header.wad_type }

    /// Количество lump-ов
    pub fn num_lumps(&self) -> usize { self.lumps.len() }

    /// Находит индекс первого lump с заданным именем.
    pub fn find_lump(&self, name: &[u8]) -> Option<usize> {
        self.lumps.iter().position(|l| l.name_matches(name))
    }

    /// Возвращает данные lump по индексу.
    pub fn lump_data(&self, index: usize) -> Option<&'static [u8]> {
        let info = self.lumps.get(index)?;
        let start = info.filepos as usize;
        let end   = start + info.size as usize;
        if end > self.data.len() { return None; }
        Some(&self.data[start..end])
    }

    /// Возвращает данные lump по имени.
    pub fn lump_by_name(&self, name: &[u8]) -> Option<&'static [u8]> {
        let idx = self.find_lump(name)?;
        self.lump_data(idx)
    }

    /// Метаданные lump по индексу.
    pub fn lump_info(&self, index: usize) -> Option<&LumpInfo> {
        self.lumps.get(index)
    }

    /// Список всех lump (для отладки / статуса)
    pub fn lumps(&self) -> &[LumpInfo] { self.lumps.as_slice() }
}

// ============================================================
// Вспомогательные функции
// ============================================================

/// Little-endian u32 из байтового среза
#[inline]
fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    let b = &data[offset..offset + 4];
    (b[0] as u32)
        | ((b[1] as u32) << 8)
        | ((b[2] as u32) << 16)
        | ((b[3] as u32) << 24)
}

// ============================================================
// Глобальный WAD — загружается один раз через init()
// ============================================================

static mut WAD_DATA: Option<&'static [u8]> = None;

/// Возвращает ссылку на загруженные WAD-данные.
pub fn wad_data() -> Option<&'static [u8]> {
    // SAFETY: WAD_DATA — static mut Option<&'static [u8]>. Инициализируется
    // один раз через load() до вызова doom::run(); только читается после.
    // Bare-metal однопоточное ядро — гонок данных нет.
    unsafe { WAD_DATA }
}

/// Инициализирует глобальный WAD.
pub fn load(data: &'static [u8]) {
    // SAFETY: вызывается один раз из doom::init() до start game loop.
    // &'static гарантирует что данные живут до конца программы.
    unsafe { WAD_DATA = Some(data); }
}

/// Возвращает WadReader если WAD загружен.
pub fn reader() -> Option<WadReader> {
    // SAFETY: аналогично wad_data() — только читаем WAD_DATA.
    // Каждый вызов аллоцирует новый Vec через bump-аллокатор;
    // для многократных вызовов предпочтительно кешировать результат.
    unsafe { WAD_DATA.and_then(WadReader::new) }
}
