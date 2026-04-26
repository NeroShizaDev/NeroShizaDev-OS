// ============================================================
// Автосгенерировано из Unicode 17.0 UnicodeData.txt
// Всего символов: 299,382
// Сжатых диапазонов: 3,409
// Категорий: 29
// NeroShizaDev UCD Generator — Категории Всех Символов Мира
// ============================================================

/// General Category Unicode — тип символа
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum GeneralCategory {
    Cc = 0,  // Control
    Cf = 1,  // Format
    Co = 2,  // Private Use
    Cs = 3,  // Surrogate
    Ll = 4,  // Lowercase Letter
    Lm = 5,  // Modifier Letter
    Lo = 6,  // Other Letter
    Lt = 7,  // Titlecase Letter
    Lu = 8,  // Uppercase Letter
    Mc = 9,  // Spacing Mark
    Me = 10, // Enclosing Mark
    Mn = 11, // Nonspacing Mark
    Nd = 12, // Decimal Number
    Nl = 13, // Letter Number
    No = 14, // Other Number
    Pc = 15, // Connector Punctuation
    Pd = 16, // Dash Punctuation
    Pe = 17, // Close Punctuation
    Pf = 18, // Final Punctuation
    Pi = 19, // Initial Punctuation
    Po = 20, // Other Punctuation
    Ps = 21, // Open Punctuation
    Sc = 22, // Currency Symbol
    Sk = 23, // Modifier Symbol
    Sm = 24, // Math Symbol
    So = 25, // Other Symbol
    Zl = 26, // Line Separator
    Zp = 27, // Paragraph Separator
    Zs = 28, // Space Separator
    Cn = 29, // Unassigned
}

impl GeneralCategory {
    pub fn name(self) -> &'static str {
        match self {
            GeneralCategory::Cc => "Control",
            GeneralCategory::Cf => "Format",
            GeneralCategory::Co => "Private Use",
            GeneralCategory::Cs => "Surrogate",
            GeneralCategory::Ll => "Lowercase Letter",
            GeneralCategory::Lm => "Modifier Letter",
            GeneralCategory::Lo => "Other Letter",
            GeneralCategory::Lt => "Titlecase Letter",
            GeneralCategory::Lu => "Uppercase Letter",
            GeneralCategory::Mc => "Spacing Mark",
            GeneralCategory::Me => "Enclosing Mark",
            GeneralCategory::Mn => "Nonspacing Mark",
            GeneralCategory::Nd => "Decimal Number",
            GeneralCategory::Nl => "Letter Number",
            GeneralCategory::No => "Other Number",
            GeneralCategory::Pc => "Connector Punctuation",
            GeneralCategory::Pd => "Dash Punctuation",
            GeneralCategory::Pe => "Close Punctuation",
            GeneralCategory::Pf => "Final Punctuation",
            GeneralCategory::Pi => "Initial Punctuation",
            GeneralCategory::Po => "Other Punctuation",
            GeneralCategory::Ps => "Open Punctuation",
            GeneralCategory::Sc => "Currency Symbol",
            GeneralCategory::Sk => "Modifier Symbol",
            GeneralCategory::Sm => "Math Symbol",
            GeneralCategory::So => "Other Symbol",
            GeneralCategory::Zl => "Line Separator",
            GeneralCategory::Zp => "Paragraph Separator",
            GeneralCategory::Zs => "Space Separator",
            GeneralCategory::Cn => "Unassigned",
        }
    }

    /// Это буква?
    pub fn is_letter(self) -> bool {
        matches!(
            self,
            GeneralCategory::Lu
                | GeneralCategory::Ll
                | GeneralCategory::Lt
                | GeneralCategory::Lm
                | GeneralCategory::Lo
        )
    }

    /// Это цифра?
    pub fn is_number(self) -> bool {
        matches!(
            self,
            GeneralCategory::Nd | GeneralCategory::Nl | GeneralCategory::No
        )
    }

    /// Это пунктуация?
    pub fn is_punctuation(self) -> bool {
        matches!(
            self,
            GeneralCategory::Pc
                | GeneralCategory::Pd
                | GeneralCategory::Ps
                | GeneralCategory::Pe
                | GeneralCategory::Pi
                | GeneralCategory::Pf
                | GeneralCategory::Po
        )
    }
}

/// Сжатые диапазоны General Category (3409 записей)
static CATEGORY_RANGES: [(u32, u32, GeneralCategory); 3409] = [
    (0x0000, 0x001F, GeneralCategory::Cc), // 32 chars
    (0x0020, 0x0020, GeneralCategory::Zs),
    (0x0021, 0x0023, GeneralCategory::Po), // 3 chars
    (0x0024, 0x0024, GeneralCategory::Sc),
    (0x0025, 0x0027, GeneralCategory::Po), // 3 chars
    (0x0028, 0x0028, GeneralCategory::Ps),
    (0x0029, 0x0029, GeneralCategory::Pe),
    (0x002A, 0x002A, GeneralCategory::Po),
    (0x002B, 0x002B, GeneralCategory::Sm),
    (0x002C, 0x002C, GeneralCategory::Po),
    (0x002D, 0x002D, GeneralCategory::Pd),
    (0x002E, 0x002F, GeneralCategory::Po), // 2 chars
    (0x0030, 0x0039, GeneralCategory::Nd), // 10 chars
    (0x003A, 0x003B, GeneralCategory::Po), // 2 chars
    (0x003C, 0x003E, GeneralCategory::Sm), // 3 chars
    (0x003F, 0x0040, GeneralCategory::Po), // 2 chars
    (0x0041, 0x005A, GeneralCategory::Lu), // 26 chars
    (0x005B, 0x005B, GeneralCategory::Ps),
    (0x005C, 0x005C, GeneralCategory::Po),
    (0x005D, 0x005D, GeneralCategory::Pe),
    (0x005E, 0x005E, GeneralCategory::Sk),
    (0x005F, 0x005F, GeneralCategory::Pc),
    (0x0060, 0x0060, GeneralCategory::Sk),
    (0x0061, 0x007A, GeneralCategory::Ll), // 26 chars
    (0x007B, 0x007B, GeneralCategory::Ps),
    (0x007C, 0x007C, GeneralCategory::Sm),
    (0x007D, 0x007D, GeneralCategory::Pe),
    (0x007E, 0x007E, GeneralCategory::Sm),
    (0x007F, 0x009F, GeneralCategory::Cc), // 33 chars
    (0x00A0, 0x00A0, GeneralCategory::Zs),
    (0x00A1, 0x00A1, GeneralCategory::Po),
    (0x00A2, 0x00A5, GeneralCategory::Sc), // 4 chars
    (0x00A6, 0x00A6, GeneralCategory::So),
    (0x00A7, 0x00A7, GeneralCategory::Po),
    (0x00A8, 0x00A8, GeneralCategory::Sk),
    (0x00A9, 0x00A9, GeneralCategory::So),
    (0x00AA, 0x00AA, GeneralCategory::Lo),
    (0x00AB, 0x00AB, GeneralCategory::Pi),
    (0x00AC, 0x00AC, GeneralCategory::Sm),
    (0x00AD, 0x00AD, GeneralCategory::Cf),
    (0x00AE, 0x00AE, GeneralCategory::So),
    (0x00AF, 0x00AF, GeneralCategory::Sk),
    (0x00B0, 0x00B0, GeneralCategory::So),
    (0x00B1, 0x00B1, GeneralCategory::Sm),
    (0x00B2, 0x00B3, GeneralCategory::No), // 2 chars
    (0x00B4, 0x00B4, GeneralCategory::Sk),
    (0x00B5, 0x00B5, GeneralCategory::Ll),
    (0x00B6, 0x00B7, GeneralCategory::Po), // 2 chars
    (0x00B8, 0x00B8, GeneralCategory::Sk),
    (0x00B9, 0x00B9, GeneralCategory::No),
    (0x00BA, 0x00BA, GeneralCategory::Lo),
    (0x00BB, 0x00BB, GeneralCategory::Pf),
    (0x00BC, 0x00BE, GeneralCategory::No), // 3 chars
    (0x00BF, 0x00BF, GeneralCategory::Po),
    (0x00C0, 0x00D6, GeneralCategory::Lu), // 23 chars
    (0x00D7, 0x00D7, GeneralCategory::Sm),
    (0x00D8, 0x00DE, GeneralCategory::Lu), // 7 chars
    (0x00DF, 0x00F6, GeneralCategory::Ll), // 24 chars
    (0x00F7, 0x00F7, GeneralCategory::Sm),
    (0x00F8, 0x00FF, GeneralCategory::Ll), // 8 chars
    (0x0100, 0x0100, GeneralCategory::Lu),
    (0x0101, 0x0101, GeneralCategory::Ll),
    (0x0102, 0x0102, GeneralCategory::Lu),
    (0x0103, 0x0103, GeneralCategory::Ll),
    (0x0104, 0x0104, GeneralCategory::Lu),
    (0x0105, 0x0105, GeneralCategory::Ll),
    (0x0106, 0x0106, GeneralCategory::Lu),
    (0x0107, 0x0107, GeneralCategory::Ll),
    (0x0108, 0x0108, GeneralCategory::Lu),
    (0x0109, 0x0109, GeneralCategory::Ll),
    (0x010A, 0x010A, GeneralCategory::Lu),
    (0x010B, 0x010B, GeneralCategory::Ll),
    (0x010C, 0x010C, GeneralCategory::Lu),
    (0x010D, 0x010D, GeneralCategory::Ll),
    (0x010E, 0x010E, GeneralCategory::Lu),
    (0x010F, 0x010F, GeneralCategory::Ll),
    (0x0110, 0x0110, GeneralCategory::Lu),
    (0x0111, 0x0111, GeneralCategory::Ll),
    (0x0112, 0x0112, GeneralCategory::Lu),
    (0x0113, 0x0113, GeneralCategory::Ll),
    (0x0114, 0x0114, GeneralCategory::Lu),
    (0x0115, 0x0115, GeneralCategory::Ll),
    (0x0116, 0x0116, GeneralCategory::Lu),
    (0x0117, 0x0117, GeneralCategory::Ll),
    (0x0118, 0x0118, GeneralCategory::Lu),
    (0x0119, 0x0119, GeneralCategory::Ll),
    (0x011A, 0x011A, GeneralCategory::Lu),
    (0x011B, 0x011B, GeneralCategory::Ll),
    (0x011C, 0x011C, GeneralCategory::Lu),
    (0x011D, 0x011D, GeneralCategory::Ll),
    (0x011E, 0x011E, GeneralCategory::Lu),
    (0x011F, 0x011F, GeneralCategory::Ll),
    (0x0120, 0x0120, GeneralCategory::Lu),
    (0x0121, 0x0121, GeneralCategory::Ll),
    (0x0122, 0x0122, GeneralCategory::Lu),
    (0x0123, 0x0123, GeneralCategory::Ll),
    (0x0124, 0x0124, GeneralCategory::Lu),
    (0x0125, 0x0125, GeneralCategory::Ll),
    (0x0126, 0x0126, GeneralCategory::Lu),
    (0x0127, 0x0127, GeneralCategory::Ll),
    (0x0128, 0x0128, GeneralCategory::Lu),
    (0x0129, 0x0129, GeneralCategory::Ll),
    (0x012A, 0x012A, GeneralCategory::Lu),
    (0x012B, 0x012B, GeneralCategory::Ll),
    (0x012C, 0x012C, GeneralCategory::Lu),
    (0x012D, 0x012D, GeneralCategory::Ll),
    (0x012E, 0x012E, GeneralCategory::Lu),
    (0x012F, 0x012F, GeneralCategory::Ll),
    (0x0130, 0x0130, GeneralCategory::Lu),
    (0x0131, 0x0131, GeneralCategory::Ll),
    (0x0132, 0x0132, GeneralCategory::Lu),
    (0x0133, 0x0133, GeneralCategory::Ll),
    (0x0134, 0x0134, GeneralCategory::Lu),
    (0x0135, 0x0135, GeneralCategory::Ll),
    (0x0136, 0x0136, GeneralCategory::Lu),
    (0x0137, 0x0138, GeneralCategory::Ll), // 2 chars
    (0x0139, 0x0139, GeneralCategory::Lu),
    (0x013A, 0x013A, GeneralCategory::Ll),
    (0x013B, 0x013B, GeneralCategory::Lu),
    (0x013C, 0x013C, GeneralCategory::Ll),
    (0x013D, 0x013D, GeneralCategory::Lu),
    (0x013E, 0x013E, GeneralCategory::Ll),
    (0x013F, 0x013F, GeneralCategory::Lu),
    (0x0140, 0x0140, GeneralCategory::Ll),
    (0x0141, 0x0141, GeneralCategory::Lu),
    (0x0142, 0x0142, GeneralCategory::Ll),
    (0x0143, 0x0143, GeneralCategory::Lu),
    (0x0144, 0x0144, GeneralCategory::Ll),
    (0x0145, 0x0145, GeneralCategory::Lu),
    (0x0146, 0x0146, GeneralCategory::Ll),
    (0x0147, 0x0147, GeneralCategory::Lu),
    (0x0148, 0x0149, GeneralCategory::Ll), // 2 chars
    (0x014A, 0x014A, GeneralCategory::Lu),
    (0x014B, 0x014B, GeneralCategory::Ll),
    (0x014C, 0x014C, GeneralCategory::Lu),
    (0x014D, 0x014D, GeneralCategory::Ll),
    (0x014E, 0x014E, GeneralCategory::Lu),
    (0x014F, 0x014F, GeneralCategory::Ll),
    (0x0150, 0x0150, GeneralCategory::Lu),
    (0x0151, 0x0151, GeneralCategory::Ll),
    (0x0152, 0x0152, GeneralCategory::Lu),
    (0x0153, 0x0153, GeneralCategory::Ll),
    (0x0154, 0x0154, GeneralCategory::Lu),
    (0x0155, 0x0155, GeneralCategory::Ll),
    (0x0156, 0x0156, GeneralCategory::Lu),
    (0x0157, 0x0157, GeneralCategory::Ll),
    (0x0158, 0x0158, GeneralCategory::Lu),
    (0x0159, 0x0159, GeneralCategory::Ll),
    (0x015A, 0x015A, GeneralCategory::Lu),
    (0x015B, 0x015B, GeneralCategory::Ll),
    (0x015C, 0x015C, GeneralCategory::Lu),
    (0x015D, 0x015D, GeneralCategory::Ll),
    (0x015E, 0x015E, GeneralCategory::Lu),
    (0x015F, 0x015F, GeneralCategory::Ll),
    (0x0160, 0x0160, GeneralCategory::Lu),
    (0x0161, 0x0161, GeneralCategory::Ll),
    (0x0162, 0x0162, GeneralCategory::Lu),
    (0x0163, 0x0163, GeneralCategory::Ll),
    (0x0164, 0x0164, GeneralCategory::Lu),
    (0x0165, 0x0165, GeneralCategory::Ll),
    (0x0166, 0x0166, GeneralCategory::Lu),
    (0x0167, 0x0167, GeneralCategory::Ll),
    (0x0168, 0x0168, GeneralCategory::Lu),
    (0x0169, 0x0169, GeneralCategory::Ll),
    (0x016A, 0x016A, GeneralCategory::Lu),
    (0x016B, 0x016B, GeneralCategory::Ll),
    (0x016C, 0x016C, GeneralCategory::Lu),
    (0x016D, 0x016D, GeneralCategory::Ll),
    (0x016E, 0x016E, GeneralCategory::Lu),
    (0x016F, 0x016F, GeneralCategory::Ll),
    (0x0170, 0x0170, GeneralCategory::Lu),
    (0x0171, 0x0171, GeneralCategory::Ll),
    (0x0172, 0x0172, GeneralCategory::Lu),
    (0x0173, 0x0173, GeneralCategory::Ll),
    (0x0174, 0x0174, GeneralCategory::Lu),
    (0x0175, 0x0175, GeneralCategory::Ll),
    (0x0176, 0x0176, GeneralCategory::Lu),
    (0x0177, 0x0177, GeneralCategory::Ll),
    (0x0178, 0x0179, GeneralCategory::Lu), // 2 chars
    (0x017A, 0x017A, GeneralCategory::Ll),
    (0x017B, 0x017B, GeneralCategory::Lu),
    (0x017C, 0x017C, GeneralCategory::Ll),
    (0x017D, 0x017D, GeneralCategory::Lu),
    (0x017E, 0x0180, GeneralCategory::Ll), // 3 chars
    (0x0181, 0x0182, GeneralCategory::Lu), // 2 chars
    (0x0183, 0x0183, GeneralCategory::Ll),
    (0x0184, 0x0184, GeneralCategory::Lu),
    (0x0185, 0x0185, GeneralCategory::Ll),
    (0x0186, 0x0187, GeneralCategory::Lu), // 2 chars
    (0x0188, 0x0188, GeneralCategory::Ll),
    (0x0189, 0x018B, GeneralCategory::Lu), // 3 chars
    (0x018C, 0x018D, GeneralCategory::Ll), // 2 chars
    (0x018E, 0x0191, GeneralCategory::Lu), // 4 chars
    (0x0192, 0x0192, GeneralCategory::Ll),
    (0x0193, 0x0194, GeneralCategory::Lu), // 2 chars
    (0x0195, 0x0195, GeneralCategory::Ll),
    (0x0196, 0x0198, GeneralCategory::Lu), // 3 chars
    (0x0199, 0x019B, GeneralCategory::Ll), // 3 chars
    (0x019C, 0x019D, GeneralCategory::Lu), // 2 chars
    (0x019E, 0x019E, GeneralCategory::Ll),
    (0x019F, 0x01A0, GeneralCategory::Lu), // 2 chars
    (0x01A1, 0x01A1, GeneralCategory::Ll),
    (0x01A2, 0x01A2, GeneralCategory::Lu),
    (0x01A3, 0x01A3, GeneralCategory::Ll),
    (0x01A4, 0x01A4, GeneralCategory::Lu),
    (0x01A5, 0x01A5, GeneralCategory::Ll),
    (0x01A6, 0x01A7, GeneralCategory::Lu), // 2 chars
    (0x01A8, 0x01A8, GeneralCategory::Ll),
    (0x01A9, 0x01A9, GeneralCategory::Lu),
    (0x01AA, 0x01AB, GeneralCategory::Ll), // 2 chars
    (0x01AC, 0x01AC, GeneralCategory::Lu),
    (0x01AD, 0x01AD, GeneralCategory::Ll),
    (0x01AE, 0x01AF, GeneralCategory::Lu), // 2 chars
    (0x01B0, 0x01B0, GeneralCategory::Ll),
    (0x01B1, 0x01B3, GeneralCategory::Lu), // 3 chars
    (0x01B4, 0x01B4, GeneralCategory::Ll),
    (0x01B5, 0x01B5, GeneralCategory::Lu),
    (0x01B6, 0x01B6, GeneralCategory::Ll),
    (0x01B7, 0x01B8, GeneralCategory::Lu), // 2 chars
    (0x01B9, 0x01BA, GeneralCategory::Ll), // 2 chars
    (0x01BB, 0x01BB, GeneralCategory::Lo),
    (0x01BC, 0x01BC, GeneralCategory::Lu),
    (0x01BD, 0x01BF, GeneralCategory::Ll), // 3 chars
    (0x01C0, 0x01C3, GeneralCategory::Lo), // 4 chars
    (0x01C4, 0x01C4, GeneralCategory::Lu),
    (0x01C5, 0x01C5, GeneralCategory::Lt),
    (0x01C6, 0x01C6, GeneralCategory::Ll),
    (0x01C7, 0x01C7, GeneralCategory::Lu),
    (0x01C8, 0x01C8, GeneralCategory::Lt),
    (0x01C9, 0x01C9, GeneralCategory::Ll),
    (0x01CA, 0x01CA, GeneralCategory::Lu),
    (0x01CB, 0x01CB, GeneralCategory::Lt),
    (0x01CC, 0x01CC, GeneralCategory::Ll),
    (0x01CD, 0x01CD, GeneralCategory::Lu),
    (0x01CE, 0x01CE, GeneralCategory::Ll),
    (0x01CF, 0x01CF, GeneralCategory::Lu),
    (0x01D0, 0x01D0, GeneralCategory::Ll),
    (0x01D1, 0x01D1, GeneralCategory::Lu),
    (0x01D2, 0x01D2, GeneralCategory::Ll),
    (0x01D3, 0x01D3, GeneralCategory::Lu),
    (0x01D4, 0x01D4, GeneralCategory::Ll),
    (0x01D5, 0x01D5, GeneralCategory::Lu),
    (0x01D6, 0x01D6, GeneralCategory::Ll),
    (0x01D7, 0x01D7, GeneralCategory::Lu),
    (0x01D8, 0x01D8, GeneralCategory::Ll),
    (0x01D9, 0x01D9, GeneralCategory::Lu),
    (0x01DA, 0x01DA, GeneralCategory::Ll),
    (0x01DB, 0x01DB, GeneralCategory::Lu),
    (0x01DC, 0x01DD, GeneralCategory::Ll), // 2 chars
    (0x01DE, 0x01DE, GeneralCategory::Lu),
    (0x01DF, 0x01DF, GeneralCategory::Ll),
    (0x01E0, 0x01E0, GeneralCategory::Lu),
    (0x01E1, 0x01E1, GeneralCategory::Ll),
    (0x01E2, 0x01E2, GeneralCategory::Lu),
    (0x01E3, 0x01E3, GeneralCategory::Ll),
    (0x01E4, 0x01E4, GeneralCategory::Lu),
    (0x01E5, 0x01E5, GeneralCategory::Ll),
    (0x01E6, 0x01E6, GeneralCategory::Lu),
    (0x01E7, 0x01E7, GeneralCategory::Ll),
    (0x01E8, 0x01E8, GeneralCategory::Lu),
    (0x01E9, 0x01E9, GeneralCategory::Ll),
    (0x01EA, 0x01EA, GeneralCategory::Lu),
    (0x01EB, 0x01EB, GeneralCategory::Ll),
    (0x01EC, 0x01EC, GeneralCategory::Lu),
    (0x01ED, 0x01ED, GeneralCategory::Ll),
    (0x01EE, 0x01EE, GeneralCategory::Lu),
    (0x01EF, 0x01F0, GeneralCategory::Ll), // 2 chars
    (0x01F1, 0x01F1, GeneralCategory::Lu),
    (0x01F2, 0x01F2, GeneralCategory::Lt),
    (0x01F3, 0x01F3, GeneralCategory::Ll),
    (0x01F4, 0x01F4, GeneralCategory::Lu),
    (0x01F5, 0x01F5, GeneralCategory::Ll),
    (0x01F6, 0x01F8, GeneralCategory::Lu), // 3 chars
    (0x01F9, 0x01F9, GeneralCategory::Ll),
    (0x01FA, 0x01FA, GeneralCategory::Lu),
    (0x01FB, 0x01FB, GeneralCategory::Ll),
    (0x01FC, 0x01FC, GeneralCategory::Lu),
    (0x01FD, 0x01FD, GeneralCategory::Ll),
    (0x01FE, 0x01FE, GeneralCategory::Lu),
    (0x01FF, 0x01FF, GeneralCategory::Ll),
    (0x0200, 0x0200, GeneralCategory::Lu),
    (0x0201, 0x0201, GeneralCategory::Ll),
    (0x0202, 0x0202, GeneralCategory::Lu),
    (0x0203, 0x0203, GeneralCategory::Ll),
    (0x0204, 0x0204, GeneralCategory::Lu),
    (0x0205, 0x0205, GeneralCategory::Ll),
    (0x0206, 0x0206, GeneralCategory::Lu),
    (0x0207, 0x0207, GeneralCategory::Ll),
    (0x0208, 0x0208, GeneralCategory::Lu),
    (0x0209, 0x0209, GeneralCategory::Ll),
    (0x020A, 0x020A, GeneralCategory::Lu),
    (0x020B, 0x020B, GeneralCategory::Ll),
    (0x020C, 0x020C, GeneralCategory::Lu),
    (0x020D, 0x020D, GeneralCategory::Ll),
    (0x020E, 0x020E, GeneralCategory::Lu),
    (0x020F, 0x020F, GeneralCategory::Ll),
    (0x0210, 0x0210, GeneralCategory::Lu),
    (0x0211, 0x0211, GeneralCategory::Ll),
    (0x0212, 0x0212, GeneralCategory::Lu),
    (0x0213, 0x0213, GeneralCategory::Ll),
    (0x0214, 0x0214, GeneralCategory::Lu),
    (0x0215, 0x0215, GeneralCategory::Ll),
    (0x0216, 0x0216, GeneralCategory::Lu),
    (0x0217, 0x0217, GeneralCategory::Ll),
    (0x0218, 0x0218, GeneralCategory::Lu),
    (0x0219, 0x0219, GeneralCategory::Ll),
    (0x021A, 0x021A, GeneralCategory::Lu),
    (0x021B, 0x021B, GeneralCategory::Ll),
    (0x021C, 0x021C, GeneralCategory::Lu),
    (0x021D, 0x021D, GeneralCategory::Ll),
    (0x021E, 0x021E, GeneralCategory::Lu),
    (0x021F, 0x021F, GeneralCategory::Ll),
    (0x0220, 0x0220, GeneralCategory::Lu),
    (0x0221, 0x0221, GeneralCategory::Ll),
    (0x0222, 0x0222, GeneralCategory::Lu),
    (0x0223, 0x0223, GeneralCategory::Ll),
    (0x0224, 0x0224, GeneralCategory::Lu),
    (0x0225, 0x0225, GeneralCategory::Ll),
    (0x0226, 0x0226, GeneralCategory::Lu),
    (0x0227, 0x0227, GeneralCategory::Ll),
    (0x0228, 0x0228, GeneralCategory::Lu),
    (0x0229, 0x0229, GeneralCategory::Ll),
    (0x022A, 0x022A, GeneralCategory::Lu),
    (0x022B, 0x022B, GeneralCategory::Ll),
    (0x022C, 0x022C, GeneralCategory::Lu),
    (0x022D, 0x022D, GeneralCategory::Ll),
    (0x022E, 0x022E, GeneralCategory::Lu),
    (0x022F, 0x022F, GeneralCategory::Ll),
    (0x0230, 0x0230, GeneralCategory::Lu),
    (0x0231, 0x0231, GeneralCategory::Ll),
    (0x0232, 0x0232, GeneralCategory::Lu),
    (0x0233, 0x0239, GeneralCategory::Ll), // 7 chars
    (0x023A, 0x023B, GeneralCategory::Lu), // 2 chars
    (0x023C, 0x023C, GeneralCategory::Ll),
    (0x023D, 0x023E, GeneralCategory::Lu), // 2 chars
    (0x023F, 0x0240, GeneralCategory::Ll), // 2 chars
    (0x0241, 0x0241, GeneralCategory::Lu),
    (0x0242, 0x0242, GeneralCategory::Ll),
    (0x0243, 0x0246, GeneralCategory::Lu), // 4 chars
    (0x0247, 0x0247, GeneralCategory::Ll),
    (0x0248, 0x0248, GeneralCategory::Lu),
    (0x0249, 0x0249, GeneralCategory::Ll),
    (0x024A, 0x024A, GeneralCategory::Lu),
    (0x024B, 0x024B, GeneralCategory::Ll),
    (0x024C, 0x024C, GeneralCategory::Lu),
    (0x024D, 0x024D, GeneralCategory::Ll),
    (0x024E, 0x024E, GeneralCategory::Lu),
    (0x024F, 0x0293, GeneralCategory::Ll), // 69 chars
    (0x0294, 0x0295, GeneralCategory::Lo), // 2 chars
    (0x0296, 0x02AF, GeneralCategory::Ll), // 26 chars
    (0x02B0, 0x02C1, GeneralCategory::Lm), // 18 chars
    (0x02C2, 0x02C5, GeneralCategory::Sk), // 4 chars
    (0x02C6, 0x02D1, GeneralCategory::Lm), // 12 chars
    (0x02D2, 0x02DF, GeneralCategory::Sk), // 14 chars
    (0x02E0, 0x02E4, GeneralCategory::Lm), // 5 chars
    (0x02E5, 0x02EB, GeneralCategory::Sk), // 7 chars
    (0x02EC, 0x02EC, GeneralCategory::Lm),
    (0x02ED, 0x02ED, GeneralCategory::Sk),
    (0x02EE, 0x02EE, GeneralCategory::Lm),
    (0x02EF, 0x02FF, GeneralCategory::Sk), // 17 chars
    (0x0300, 0x036F, GeneralCategory::Mn), // 112 chars
    (0x0370, 0x0370, GeneralCategory::Lu),
    (0x0371, 0x0371, GeneralCategory::Ll),
    (0x0372, 0x0372, GeneralCategory::Lu),
    (0x0373, 0x0373, GeneralCategory::Ll),
    (0x0374, 0x0374, GeneralCategory::Lm),
    (0x0375, 0x0375, GeneralCategory::Sk),
    (0x0376, 0x0376, GeneralCategory::Lu),
    (0x0377, 0x0377, GeneralCategory::Ll),
    (0x037A, 0x037A, GeneralCategory::Lm),
    (0x037B, 0x037D, GeneralCategory::Ll), // 3 chars
    (0x037E, 0x037E, GeneralCategory::Po),
    (0x037F, 0x037F, GeneralCategory::Lu),
    (0x0384, 0x0385, GeneralCategory::Sk), // 2 chars
    (0x0386, 0x0386, GeneralCategory::Lu),
    (0x0387, 0x0387, GeneralCategory::Po),
    (0x0388, 0x038A, GeneralCategory::Lu), // 3 chars
    (0x038C, 0x038C, GeneralCategory::Lu),
    (0x038E, 0x038F, GeneralCategory::Lu), // 2 chars
    (0x0390, 0x0390, GeneralCategory::Ll),
    (0x0391, 0x03A1, GeneralCategory::Lu), // 17 chars
    (0x03A3, 0x03AB, GeneralCategory::Lu), // 9 chars
    (0x03AC, 0x03CE, GeneralCategory::Ll), // 35 chars
    (0x03CF, 0x03CF, GeneralCategory::Lu),
    (0x03D0, 0x03D1, GeneralCategory::Ll), // 2 chars
    (0x03D2, 0x03D4, GeneralCategory::Lu), // 3 chars
    (0x03D5, 0x03D7, GeneralCategory::Ll), // 3 chars
    (0x03D8, 0x03D8, GeneralCategory::Lu),
    (0x03D9, 0x03D9, GeneralCategory::Ll),
    (0x03DA, 0x03DA, GeneralCategory::Lu),
    (0x03DB, 0x03DB, GeneralCategory::Ll),
    (0x03DC, 0x03DC, GeneralCategory::Lu),
    (0x03DD, 0x03DD, GeneralCategory::Ll),
    (0x03DE, 0x03DE, GeneralCategory::Lu),
    (0x03DF, 0x03DF, GeneralCategory::Ll),
    (0x03E0, 0x03E0, GeneralCategory::Lu),
    (0x03E1, 0x03E1, GeneralCategory::Ll),
    (0x03E2, 0x03E2, GeneralCategory::Lu),
    (0x03E3, 0x03E3, GeneralCategory::Ll),
    (0x03E4, 0x03E4, GeneralCategory::Lu),
    (0x03E5, 0x03E5, GeneralCategory::Ll),
    (0x03E6, 0x03E6, GeneralCategory::Lu),
    (0x03E7, 0x03E7, GeneralCategory::Ll),
    (0x03E8, 0x03E8, GeneralCategory::Lu),
    (0x03E9, 0x03E9, GeneralCategory::Ll),
    (0x03EA, 0x03EA, GeneralCategory::Lu),
    (0x03EB, 0x03EB, GeneralCategory::Ll),
    (0x03EC, 0x03EC, GeneralCategory::Lu),
    (0x03ED, 0x03ED, GeneralCategory::Ll),
    (0x03EE, 0x03EE, GeneralCategory::Lu),
    (0x03EF, 0x03F3, GeneralCategory::Ll), // 5 chars
    (0x03F4, 0x03F4, GeneralCategory::Lu),
    (0x03F5, 0x03F5, GeneralCategory::Ll),
    (0x03F6, 0x03F6, GeneralCategory::Sm),
    (0x03F7, 0x03F7, GeneralCategory::Lu),
    (0x03F8, 0x03F8, GeneralCategory::Ll),
    (0x03F9, 0x03FA, GeneralCategory::Lu), // 2 chars
    (0x03FB, 0x03FC, GeneralCategory::Ll), // 2 chars
    (0x03FD, 0x042F, GeneralCategory::Lu), // 51 chars
    (0x0430, 0x045F, GeneralCategory::Ll), // 48 chars
    (0x0460, 0x0460, GeneralCategory::Lu),
    (0x0461, 0x0461, GeneralCategory::Ll),
    (0x0462, 0x0462, GeneralCategory::Lu),
    (0x0463, 0x0463, GeneralCategory::Ll),
    (0x0464, 0x0464, GeneralCategory::Lu),
    (0x0465, 0x0465, GeneralCategory::Ll),
    (0x0466, 0x0466, GeneralCategory::Lu),
    (0x0467, 0x0467, GeneralCategory::Ll),
    (0x0468, 0x0468, GeneralCategory::Lu),
    (0x0469, 0x0469, GeneralCategory::Ll),
    (0x046A, 0x046A, GeneralCategory::Lu),
    (0x046B, 0x046B, GeneralCategory::Ll),
    (0x046C, 0x046C, GeneralCategory::Lu),
    (0x046D, 0x046D, GeneralCategory::Ll),
    (0x046E, 0x046E, GeneralCategory::Lu),
    (0x046F, 0x046F, GeneralCategory::Ll),
    (0x0470, 0x0470, GeneralCategory::Lu),
    (0x0471, 0x0471, GeneralCategory::Ll),
    (0x0472, 0x0472, GeneralCategory::Lu),
    (0x0473, 0x0473, GeneralCategory::Ll),
    (0x0474, 0x0474, GeneralCategory::Lu),
    (0x0475, 0x0475, GeneralCategory::Ll),
    (0x0476, 0x0476, GeneralCategory::Lu),
    (0x0477, 0x0477, GeneralCategory::Ll),
    (0x0478, 0x0478, GeneralCategory::Lu),
    (0x0479, 0x0479, GeneralCategory::Ll),
    (0x047A, 0x047A, GeneralCategory::Lu),
    (0x047B, 0x047B, GeneralCategory::Ll),
    (0x047C, 0x047C, GeneralCategory::Lu),
    (0x047D, 0x047D, GeneralCategory::Ll),
    (0x047E, 0x047E, GeneralCategory::Lu),
    (0x047F, 0x047F, GeneralCategory::Ll),
    (0x0480, 0x0480, GeneralCategory::Lu),
    (0x0481, 0x0481, GeneralCategory::Ll),
    (0x0482, 0x0482, GeneralCategory::So),
    (0x0483, 0x0487, GeneralCategory::Mn), // 5 chars
    (0x0488, 0x0489, GeneralCategory::Me), // 2 chars
    (0x048A, 0x048A, GeneralCategory::Lu),
    (0x048B, 0x048B, GeneralCategory::Ll),
    (0x048C, 0x048C, GeneralCategory::Lu),
    (0x048D, 0x048D, GeneralCategory::Ll),
    (0x048E, 0x048E, GeneralCategory::Lu),
    (0x048F, 0x048F, GeneralCategory::Ll),
    (0x0490, 0x0490, GeneralCategory::Lu),
    (0x0491, 0x0491, GeneralCategory::Ll),
    (0x0492, 0x0492, GeneralCategory::Lu),
    (0x0493, 0x0493, GeneralCategory::Ll),
    (0x0494, 0x0494, GeneralCategory::Lu),
    (0x0495, 0x0495, GeneralCategory::Ll),
    (0x0496, 0x0496, GeneralCategory::Lu),
    (0x0497, 0x0497, GeneralCategory::Ll),
    (0x0498, 0x0498, GeneralCategory::Lu),
    (0x0499, 0x0499, GeneralCategory::Ll),
    (0x049A, 0x049A, GeneralCategory::Lu),
    (0x049B, 0x049B, GeneralCategory::Ll),
    (0x049C, 0x049C, GeneralCategory::Lu),
    (0x049D, 0x049D, GeneralCategory::Ll),
    (0x049E, 0x049E, GeneralCategory::Lu),
    (0x049F, 0x049F, GeneralCategory::Ll),
    (0x04A0, 0x04A0, GeneralCategory::Lu),
    (0x04A1, 0x04A1, GeneralCategory::Ll),
    (0x04A2, 0x04A2, GeneralCategory::Lu),
    (0x04A3, 0x04A3, GeneralCategory::Ll),
    (0x04A4, 0x04A4, GeneralCategory::Lu),
    (0x04A5, 0x04A5, GeneralCategory::Ll),
    (0x04A6, 0x04A6, GeneralCategory::Lu),
    (0x04A7, 0x04A7, GeneralCategory::Ll),
    (0x04A8, 0x04A8, GeneralCategory::Lu),
    (0x04A9, 0x04A9, GeneralCategory::Ll),
    (0x04AA, 0x04AA, GeneralCategory::Lu),
    (0x04AB, 0x04AB, GeneralCategory::Ll),
    (0x04AC, 0x04AC, GeneralCategory::Lu),
    (0x04AD, 0x04AD, GeneralCategory::Ll),
    (0x04AE, 0x04AE, GeneralCategory::Lu),
    (0x04AF, 0x04AF, GeneralCategory::Ll),
    (0x04B0, 0x04B0, GeneralCategory::Lu),
    (0x04B1, 0x04B1, GeneralCategory::Ll),
    (0x04B2, 0x04B2, GeneralCategory::Lu),
    (0x04B3, 0x04B3, GeneralCategory::Ll),
    (0x04B4, 0x04B4, GeneralCategory::Lu),
    (0x04B5, 0x04B5, GeneralCategory::Ll),
    (0x04B6, 0x04B6, GeneralCategory::Lu),
    (0x04B7, 0x04B7, GeneralCategory::Ll),
    (0x04B8, 0x04B8, GeneralCategory::Lu),
    (0x04B9, 0x04B9, GeneralCategory::Ll),
    (0x04BA, 0x04BA, GeneralCategory::Lu),
    (0x04BB, 0x04BB, GeneralCategory::Ll),
    (0x04BC, 0x04BC, GeneralCategory::Lu),
    (0x04BD, 0x04BD, GeneralCategory::Ll),
    (0x04BE, 0x04BE, GeneralCategory::Lu),
    (0x04BF, 0x04BF, GeneralCategory::Ll),
    (0x04C0, 0x04C1, GeneralCategory::Lu), // 2 chars
    (0x04C2, 0x04C2, GeneralCategory::Ll),
    (0x04C3, 0x04C3, GeneralCategory::Lu),
    (0x04C4, 0x04C4, GeneralCategory::Ll),
    (0x04C5, 0x04C5, GeneralCategory::Lu),
    (0x04C6, 0x04C6, GeneralCategory::Ll),
    (0x04C7, 0x04C7, GeneralCategory::Lu),
    (0x04C8, 0x04C8, GeneralCategory::Ll),
    (0x04C9, 0x04C9, GeneralCategory::Lu),
    (0x04CA, 0x04CA, GeneralCategory::Ll),
    (0x04CB, 0x04CB, GeneralCategory::Lu),
    (0x04CC, 0x04CC, GeneralCategory::Ll),
    (0x04CD, 0x04CD, GeneralCategory::Lu),
    (0x04CE, 0x04CF, GeneralCategory::Ll), // 2 chars
    (0x04D0, 0x04D0, GeneralCategory::Lu),
    (0x04D1, 0x04D1, GeneralCategory::Ll),
    (0x04D2, 0x04D2, GeneralCategory::Lu),
    (0x04D3, 0x04D3, GeneralCategory::Ll),
    (0x04D4, 0x04D4, GeneralCategory::Lu),
    (0x04D5, 0x04D5, GeneralCategory::Ll),
    (0x04D6, 0x04D6, GeneralCategory::Lu),
    (0x04D7, 0x04D7, GeneralCategory::Ll),
    (0x04D8, 0x04D8, GeneralCategory::Lu),
    (0x04D9, 0x04D9, GeneralCategory::Ll),
    (0x04DA, 0x04DA, GeneralCategory::Lu),
    (0x04DB, 0x04DB, GeneralCategory::Ll),
    (0x04DC, 0x04DC, GeneralCategory::Lu),
    (0x04DD, 0x04DD, GeneralCategory::Ll),
    (0x04DE, 0x04DE, GeneralCategory::Lu),
    (0x04DF, 0x04DF, GeneralCategory::Ll),
    (0x04E0, 0x04E0, GeneralCategory::Lu),
    (0x04E1, 0x04E1, GeneralCategory::Ll),
    (0x04E2, 0x04E2, GeneralCategory::Lu),
    (0x04E3, 0x04E3, GeneralCategory::Ll),
    (0x04E4, 0x04E4, GeneralCategory::Lu),
    (0x04E5, 0x04E5, GeneralCategory::Ll),
    (0x04E6, 0x04E6, GeneralCategory::Lu),
    (0x04E7, 0x04E7, GeneralCategory::Ll),
    (0x04E8, 0x04E8, GeneralCategory::Lu),
    (0x04E9, 0x04E9, GeneralCategory::Ll),
    (0x04EA, 0x04EA, GeneralCategory::Lu),
    (0x04EB, 0x04EB, GeneralCategory::Ll),
    (0x04EC, 0x04EC, GeneralCategory::Lu),
    (0x04ED, 0x04ED, GeneralCategory::Ll),
    (0x04EE, 0x04EE, GeneralCategory::Lu),
    (0x04EF, 0x04EF, GeneralCategory::Ll),
    (0x04F0, 0x04F0, GeneralCategory::Lu),
    (0x04F1, 0x04F1, GeneralCategory::Ll),
    (0x04F2, 0x04F2, GeneralCategory::Lu),
    (0x04F3, 0x04F3, GeneralCategory::Ll),
    (0x04F4, 0x04F4, GeneralCategory::Lu),
    (0x04F5, 0x04F5, GeneralCategory::Ll),
    (0x04F6, 0x04F6, GeneralCategory::Lu),
    (0x04F7, 0x04F7, GeneralCategory::Ll),
    (0x04F8, 0x04F8, GeneralCategory::Lu),
    (0x04F9, 0x04F9, GeneralCategory::Ll),
    (0x04FA, 0x04FA, GeneralCategory::Lu),
    (0x04FB, 0x04FB, GeneralCategory::Ll),
    (0x04FC, 0x04FC, GeneralCategory::Lu),
    (0x04FD, 0x04FD, GeneralCategory::Ll),
    (0x04FE, 0x04FE, GeneralCategory::Lu),
    (0x04FF, 0x04FF, GeneralCategory::Ll),
    (0x0500, 0x0500, GeneralCategory::Lu),
    (0x0501, 0x0501, GeneralCategory::Ll),
    (0x0502, 0x0502, GeneralCategory::Lu),
    (0x0503, 0x0503, GeneralCategory::Ll),
    (0x0504, 0x0504, GeneralCategory::Lu),
    (0x0505, 0x0505, GeneralCategory::Ll),
    (0x0506, 0x0506, GeneralCategory::Lu),
    (0x0507, 0x0507, GeneralCategory::Ll),
    (0x0508, 0x0508, GeneralCategory::Lu),
    (0x0509, 0x0509, GeneralCategory::Ll),
    (0x050A, 0x050A, GeneralCategory::Lu),
    (0x050B, 0x050B, GeneralCategory::Ll),
    (0x050C, 0x050C, GeneralCategory::Lu),
    (0x050D, 0x050D, GeneralCategory::Ll),
    (0x050E, 0x050E, GeneralCategory::Lu),
    (0x050F, 0x050F, GeneralCategory::Ll),
    (0x0510, 0x0510, GeneralCategory::Lu),
    (0x0511, 0x0511, GeneralCategory::Ll),
    (0x0512, 0x0512, GeneralCategory::Lu),
    (0x0513, 0x0513, GeneralCategory::Ll),
    (0x0514, 0x0514, GeneralCategory::Lu),
    (0x0515, 0x0515, GeneralCategory::Ll),
    (0x0516, 0x0516, GeneralCategory::Lu),
    (0x0517, 0x0517, GeneralCategory::Ll),
    (0x0518, 0x0518, GeneralCategory::Lu),
    (0x0519, 0x0519, GeneralCategory::Ll),
    (0x051A, 0x051A, GeneralCategory::Lu),
    (0x051B, 0x051B, GeneralCategory::Ll),
    (0x051C, 0x051C, GeneralCategory::Lu),
    (0x051D, 0x051D, GeneralCategory::Ll),
    (0x051E, 0x051E, GeneralCategory::Lu),
    (0x051F, 0x051F, GeneralCategory::Ll),
    (0x0520, 0x0520, GeneralCategory::Lu),
    (0x0521, 0x0521, GeneralCategory::Ll),
    (0x0522, 0x0522, GeneralCategory::Lu),
    (0x0523, 0x0523, GeneralCategory::Ll),
    (0x0524, 0x0524, GeneralCategory::Lu),
    (0x0525, 0x0525, GeneralCategory::Ll),
    (0x0526, 0x0526, GeneralCategory::Lu),
    (0x0527, 0x0527, GeneralCategory::Ll),
    (0x0528, 0x0528, GeneralCategory::Lu),
    (0x0529, 0x0529, GeneralCategory::Ll),
    (0x052A, 0x052A, GeneralCategory::Lu),
    (0x052B, 0x052B, GeneralCategory::Ll),
    (0x052C, 0x052C, GeneralCategory::Lu),
    (0x052D, 0x052D, GeneralCategory::Ll),
    (0x052E, 0x052E, GeneralCategory::Lu),
    (0x052F, 0x052F, GeneralCategory::Ll),
    (0x0531, 0x0556, GeneralCategory::Lu), // 38 chars
    (0x0559, 0x0559, GeneralCategory::Lm),
    (0x055A, 0x055F, GeneralCategory::Po), // 6 chars
    (0x0560, 0x0588, GeneralCategory::Ll), // 41 chars
    (0x0589, 0x0589, GeneralCategory::Po),
    (0x058A, 0x058A, GeneralCategory::Pd),
    (0x058D, 0x058E, GeneralCategory::So), // 2 chars
    (0x058F, 0x058F, GeneralCategory::Sc),
    (0x0591, 0x05BD, GeneralCategory::Mn), // 45 chars
    (0x05BE, 0x05BE, GeneralCategory::Pd),
    (0x05BF, 0x05BF, GeneralCategory::Mn),
    (0x05C0, 0x05C0, GeneralCategory::Po),
    (0x05C1, 0x05C2, GeneralCategory::Mn), // 2 chars
    (0x05C3, 0x05C3, GeneralCategory::Po),
    (0x05C4, 0x05C5, GeneralCategory::Mn), // 2 chars
    (0x05C6, 0x05C6, GeneralCategory::Po),
    (0x05C7, 0x05C7, GeneralCategory::Mn),
    (0x05D0, 0x05EA, GeneralCategory::Lo), // 27 chars
    (0x05EF, 0x05F2, GeneralCategory::Lo), // 4 chars
    (0x05F3, 0x05F4, GeneralCategory::Po), // 2 chars
    (0x0600, 0x0605, GeneralCategory::Cf), // 6 chars
    (0x0606, 0x0608, GeneralCategory::Sm), // 3 chars
    (0x0609, 0x060A, GeneralCategory::Po), // 2 chars
    (0x060B, 0x060B, GeneralCategory::Sc),
    (0x060C, 0x060D, GeneralCategory::Po), // 2 chars
    (0x060E, 0x060F, GeneralCategory::So), // 2 chars
    (0x0610, 0x061A, GeneralCategory::Mn), // 11 chars
    (0x061B, 0x061B, GeneralCategory::Po),
    (0x061C, 0x061C, GeneralCategory::Cf),
    (0x061D, 0x061F, GeneralCategory::Po), // 3 chars
    (0x0620, 0x063F, GeneralCategory::Lo), // 32 chars
    (0x0640, 0x0640, GeneralCategory::Lm),
    (0x0641, 0x064A, GeneralCategory::Lo), // 10 chars
    (0x064B, 0x065F, GeneralCategory::Mn), // 21 chars
    (0x0660, 0x0669, GeneralCategory::Nd), // 10 chars
    (0x066A, 0x066D, GeneralCategory::Po), // 4 chars
    (0x066E, 0x066F, GeneralCategory::Lo), // 2 chars
    (0x0670, 0x0670, GeneralCategory::Mn),
    (0x0671, 0x06D3, GeneralCategory::Lo), // 99 chars
    (0x06D4, 0x06D4, GeneralCategory::Po),
    (0x06D5, 0x06D5, GeneralCategory::Lo),
    (0x06D6, 0x06DC, GeneralCategory::Mn), // 7 chars
    (0x06DD, 0x06DD, GeneralCategory::Cf),
    (0x06DE, 0x06DE, GeneralCategory::So),
    (0x06DF, 0x06E4, GeneralCategory::Mn), // 6 chars
    (0x06E5, 0x06E6, GeneralCategory::Lm), // 2 chars
    (0x06E7, 0x06E8, GeneralCategory::Mn), // 2 chars
    (0x06E9, 0x06E9, GeneralCategory::So),
    (0x06EA, 0x06ED, GeneralCategory::Mn), // 4 chars
    (0x06EE, 0x06EF, GeneralCategory::Lo), // 2 chars
    (0x06F0, 0x06F9, GeneralCategory::Nd), // 10 chars
    (0x06FA, 0x06FC, GeneralCategory::Lo), // 3 chars
    (0x06FD, 0x06FE, GeneralCategory::So), // 2 chars
    (0x06FF, 0x06FF, GeneralCategory::Lo),
    (0x0700, 0x070D, GeneralCategory::Po), // 14 chars
    (0x070F, 0x070F, GeneralCategory::Cf),
    (0x0710, 0x0710, GeneralCategory::Lo),
    (0x0711, 0x0711, GeneralCategory::Mn),
    (0x0712, 0x072F, GeneralCategory::Lo), // 30 chars
    (0x0730, 0x074A, GeneralCategory::Mn), // 27 chars
    (0x074D, 0x07A5, GeneralCategory::Lo), // 89 chars
    (0x07A6, 0x07B0, GeneralCategory::Mn), // 11 chars
    (0x07B1, 0x07B1, GeneralCategory::Lo),
    (0x07C0, 0x07C9, GeneralCategory::Nd), // 10 chars
    (0x07CA, 0x07EA, GeneralCategory::Lo), // 33 chars
    (0x07EB, 0x07F3, GeneralCategory::Mn), // 9 chars
    (0x07F4, 0x07F5, GeneralCategory::Lm), // 2 chars
    (0x07F6, 0x07F6, GeneralCategory::So),
    (0x07F7, 0x07F9, GeneralCategory::Po), // 3 chars
    (0x07FA, 0x07FA, GeneralCategory::Lm),
    (0x07FD, 0x07FD, GeneralCategory::Mn),
    (0x07FE, 0x07FF, GeneralCategory::Sc), // 2 chars
    (0x0800, 0x0815, GeneralCategory::Lo), // 22 chars
    (0x0816, 0x0819, GeneralCategory::Mn), // 4 chars
    (0x081A, 0x081A, GeneralCategory::Lm),
    (0x081B, 0x0823, GeneralCategory::Mn), // 9 chars
    (0x0824, 0x0824, GeneralCategory::Lm),
    (0x0825, 0x0827, GeneralCategory::Mn), // 3 chars
    (0x0828, 0x0828, GeneralCategory::Lm),
    (0x0829, 0x082D, GeneralCategory::Mn), // 5 chars
    (0x0830, 0x083E, GeneralCategory::Po), // 15 chars
    (0x0840, 0x0858, GeneralCategory::Lo), // 25 chars
    (0x0859, 0x085B, GeneralCategory::Mn), // 3 chars
    (0x085E, 0x085E, GeneralCategory::Po),
    (0x0860, 0x086A, GeneralCategory::Lo), // 11 chars
    (0x0870, 0x0887, GeneralCategory::Lo), // 24 chars
    (0x0888, 0x0888, GeneralCategory::Sk),
    (0x0889, 0x088F, GeneralCategory::Lo), // 7 chars
    (0x0890, 0x0891, GeneralCategory::Cf), // 2 chars
    (0x0897, 0x089F, GeneralCategory::Mn), // 9 chars
    (0x08A0, 0x08C8, GeneralCategory::Lo), // 41 chars
    (0x08C9, 0x08C9, GeneralCategory::Lm),
    (0x08CA, 0x08E1, GeneralCategory::Mn), // 24 chars
    (0x08E2, 0x08E2, GeneralCategory::Cf),
    (0x08E3, 0x0902, GeneralCategory::Mn), // 32 chars
    (0x0903, 0x0903, GeneralCategory::Mc),
    (0x0904, 0x0939, GeneralCategory::Lo), // 54 chars
    (0x093A, 0x093A, GeneralCategory::Mn),
    (0x093B, 0x093B, GeneralCategory::Mc),
    (0x093C, 0x093C, GeneralCategory::Mn),
    (0x093D, 0x093D, GeneralCategory::Lo),
    (0x093E, 0x0940, GeneralCategory::Mc), // 3 chars
    (0x0941, 0x0948, GeneralCategory::Mn), // 8 chars
    (0x0949, 0x094C, GeneralCategory::Mc), // 4 chars
    (0x094D, 0x094D, GeneralCategory::Mn),
    (0x094E, 0x094F, GeneralCategory::Mc), // 2 chars
    (0x0950, 0x0950, GeneralCategory::Lo),
    (0x0951, 0x0957, GeneralCategory::Mn), // 7 chars
    (0x0958, 0x0961, GeneralCategory::Lo), // 10 chars
    (0x0962, 0x0963, GeneralCategory::Mn), // 2 chars
    (0x0964, 0x0965, GeneralCategory::Po), // 2 chars
    (0x0966, 0x096F, GeneralCategory::Nd), // 10 chars
    (0x0970, 0x0970, GeneralCategory::Po),
    (0x0971, 0x0971, GeneralCategory::Lm),
    (0x0972, 0x0980, GeneralCategory::Lo), // 15 chars
    (0x0981, 0x0981, GeneralCategory::Mn),
    (0x0982, 0x0983, GeneralCategory::Mc), // 2 chars
    (0x0985, 0x098C, GeneralCategory::Lo), // 8 chars
    (0x098F, 0x0990, GeneralCategory::Lo), // 2 chars
    (0x0993, 0x09A8, GeneralCategory::Lo), // 22 chars
    (0x09AA, 0x09B0, GeneralCategory::Lo), // 7 chars
    (0x09B2, 0x09B2, GeneralCategory::Lo),
    (0x09B6, 0x09B9, GeneralCategory::Lo), // 4 chars
    (0x09BC, 0x09BC, GeneralCategory::Mn),
    (0x09BD, 0x09BD, GeneralCategory::Lo),
    (0x09BE, 0x09C0, GeneralCategory::Mc), // 3 chars
    (0x09C1, 0x09C4, GeneralCategory::Mn), // 4 chars
    (0x09C7, 0x09C8, GeneralCategory::Mc), // 2 chars
    (0x09CB, 0x09CC, GeneralCategory::Mc), // 2 chars
    (0x09CD, 0x09CD, GeneralCategory::Mn),
    (0x09CE, 0x09CE, GeneralCategory::Lo),
    (0x09D7, 0x09D7, GeneralCategory::Mc),
    (0x09DC, 0x09DD, GeneralCategory::Lo), // 2 chars
    (0x09DF, 0x09E1, GeneralCategory::Lo), // 3 chars
    (0x09E2, 0x09E3, GeneralCategory::Mn), // 2 chars
    (0x09E6, 0x09EF, GeneralCategory::Nd), // 10 chars
    (0x09F0, 0x09F1, GeneralCategory::Lo), // 2 chars
    (0x09F2, 0x09F3, GeneralCategory::Sc), // 2 chars
    (0x09F4, 0x09F9, GeneralCategory::No), // 6 chars
    (0x09FA, 0x09FA, GeneralCategory::So),
    (0x09FB, 0x09FB, GeneralCategory::Sc),
    (0x09FC, 0x09FC, GeneralCategory::Lo),
    (0x09FD, 0x09FD, GeneralCategory::Po),
    (0x09FE, 0x09FE, GeneralCategory::Mn),
    (0x0A01, 0x0A02, GeneralCategory::Mn), // 2 chars
    (0x0A03, 0x0A03, GeneralCategory::Mc),
    (0x0A05, 0x0A0A, GeneralCategory::Lo), // 6 chars
    (0x0A0F, 0x0A10, GeneralCategory::Lo), // 2 chars
    (0x0A13, 0x0A28, GeneralCategory::Lo), // 22 chars
    (0x0A2A, 0x0A30, GeneralCategory::Lo), // 7 chars
    (0x0A32, 0x0A33, GeneralCategory::Lo), // 2 chars
    (0x0A35, 0x0A36, GeneralCategory::Lo), // 2 chars
    (0x0A38, 0x0A39, GeneralCategory::Lo), // 2 chars
    (0x0A3C, 0x0A3C, GeneralCategory::Mn),
    (0x0A3E, 0x0A40, GeneralCategory::Mc), // 3 chars
    (0x0A41, 0x0A42, GeneralCategory::Mn), // 2 chars
    (0x0A47, 0x0A48, GeneralCategory::Mn), // 2 chars
    (0x0A4B, 0x0A4D, GeneralCategory::Mn), // 3 chars
    (0x0A51, 0x0A51, GeneralCategory::Mn),
    (0x0A59, 0x0A5C, GeneralCategory::Lo), // 4 chars
    (0x0A5E, 0x0A5E, GeneralCategory::Lo),
    (0x0A66, 0x0A6F, GeneralCategory::Nd), // 10 chars
    (0x0A70, 0x0A71, GeneralCategory::Mn), // 2 chars
    (0x0A72, 0x0A74, GeneralCategory::Lo), // 3 chars
    (0x0A75, 0x0A75, GeneralCategory::Mn),
    (0x0A76, 0x0A76, GeneralCategory::Po),
    (0x0A81, 0x0A82, GeneralCategory::Mn), // 2 chars
    (0x0A83, 0x0A83, GeneralCategory::Mc),
    (0x0A85, 0x0A8D, GeneralCategory::Lo), // 9 chars
    (0x0A8F, 0x0A91, GeneralCategory::Lo), // 3 chars
    (0x0A93, 0x0AA8, GeneralCategory::Lo), // 22 chars
    (0x0AAA, 0x0AB0, GeneralCategory::Lo), // 7 chars
    (0x0AB2, 0x0AB3, GeneralCategory::Lo), // 2 chars
    (0x0AB5, 0x0AB9, GeneralCategory::Lo), // 5 chars
    (0x0ABC, 0x0ABC, GeneralCategory::Mn),
    (0x0ABD, 0x0ABD, GeneralCategory::Lo),
    (0x0ABE, 0x0AC0, GeneralCategory::Mc), // 3 chars
    (0x0AC1, 0x0AC5, GeneralCategory::Mn), // 5 chars
    (0x0AC7, 0x0AC8, GeneralCategory::Mn), // 2 chars
    (0x0AC9, 0x0AC9, GeneralCategory::Mc),
    (0x0ACB, 0x0ACC, GeneralCategory::Mc), // 2 chars
    (0x0ACD, 0x0ACD, GeneralCategory::Mn),
    (0x0AD0, 0x0AD0, GeneralCategory::Lo),
    (0x0AE0, 0x0AE1, GeneralCategory::Lo), // 2 chars
    (0x0AE2, 0x0AE3, GeneralCategory::Mn), // 2 chars
    (0x0AE6, 0x0AEF, GeneralCategory::Nd), // 10 chars
    (0x0AF0, 0x0AF0, GeneralCategory::Po),
    (0x0AF1, 0x0AF1, GeneralCategory::Sc),
    (0x0AF9, 0x0AF9, GeneralCategory::Lo),
    (0x0AFA, 0x0AFF, GeneralCategory::Mn), // 6 chars
    (0x0B01, 0x0B01, GeneralCategory::Mn),
    (0x0B02, 0x0B03, GeneralCategory::Mc), // 2 chars
    (0x0B05, 0x0B0C, GeneralCategory::Lo), // 8 chars
    (0x0B0F, 0x0B10, GeneralCategory::Lo), // 2 chars
    (0x0B13, 0x0B28, GeneralCategory::Lo), // 22 chars
    (0x0B2A, 0x0B30, GeneralCategory::Lo), // 7 chars
    (0x0B32, 0x0B33, GeneralCategory::Lo), // 2 chars
    (0x0B35, 0x0B39, GeneralCategory::Lo), // 5 chars
    (0x0B3C, 0x0B3C, GeneralCategory::Mn),
    (0x0B3D, 0x0B3D, GeneralCategory::Lo),
    (0x0B3E, 0x0B3E, GeneralCategory::Mc),
    (0x0B3F, 0x0B3F, GeneralCategory::Mn),
    (0x0B40, 0x0B40, GeneralCategory::Mc),
    (0x0B41, 0x0B44, GeneralCategory::Mn), // 4 chars
    (0x0B47, 0x0B48, GeneralCategory::Mc), // 2 chars
    (0x0B4B, 0x0B4C, GeneralCategory::Mc), // 2 chars
    (0x0B4D, 0x0B4D, GeneralCategory::Mn),
    (0x0B55, 0x0B56, GeneralCategory::Mn), // 2 chars
    (0x0B57, 0x0B57, GeneralCategory::Mc),
    (0x0B5C, 0x0B5D, GeneralCategory::Lo), // 2 chars
    (0x0B5F, 0x0B61, GeneralCategory::Lo), // 3 chars
    (0x0B62, 0x0B63, GeneralCategory::Mn), // 2 chars
    (0x0B66, 0x0B6F, GeneralCategory::Nd), // 10 chars
    (0x0B70, 0x0B70, GeneralCategory::So),
    (0x0B71, 0x0B71, GeneralCategory::Lo),
    (0x0B72, 0x0B77, GeneralCategory::No), // 6 chars
    (0x0B82, 0x0B82, GeneralCategory::Mn),
    (0x0B83, 0x0B83, GeneralCategory::Lo),
    (0x0B85, 0x0B8A, GeneralCategory::Lo), // 6 chars
    (0x0B8E, 0x0B90, GeneralCategory::Lo), // 3 chars
    (0x0B92, 0x0B95, GeneralCategory::Lo), // 4 chars
    (0x0B99, 0x0B9A, GeneralCategory::Lo), // 2 chars
    (0x0B9C, 0x0B9C, GeneralCategory::Lo),
    (0x0B9E, 0x0B9F, GeneralCategory::Lo), // 2 chars
    (0x0BA3, 0x0BA4, GeneralCategory::Lo), // 2 chars
    (0x0BA8, 0x0BAA, GeneralCategory::Lo), // 3 chars
    (0x0BAE, 0x0BB9, GeneralCategory::Lo), // 12 chars
    (0x0BBE, 0x0BBF, GeneralCategory::Mc), // 2 chars
    (0x0BC0, 0x0BC0, GeneralCategory::Mn),
    (0x0BC1, 0x0BC2, GeneralCategory::Mc), // 2 chars
    (0x0BC6, 0x0BC8, GeneralCategory::Mc), // 3 chars
    (0x0BCA, 0x0BCC, GeneralCategory::Mc), // 3 chars
    (0x0BCD, 0x0BCD, GeneralCategory::Mn),
    (0x0BD0, 0x0BD0, GeneralCategory::Lo),
    (0x0BD7, 0x0BD7, GeneralCategory::Mc),
    (0x0BE6, 0x0BEF, GeneralCategory::Nd), // 10 chars
    (0x0BF0, 0x0BF2, GeneralCategory::No), // 3 chars
    (0x0BF3, 0x0BF8, GeneralCategory::So), // 6 chars
    (0x0BF9, 0x0BF9, GeneralCategory::Sc),
    (0x0BFA, 0x0BFA, GeneralCategory::So),
    (0x0C00, 0x0C00, GeneralCategory::Mn),
    (0x0C01, 0x0C03, GeneralCategory::Mc), // 3 chars
    (0x0C04, 0x0C04, GeneralCategory::Mn),
    (0x0C05, 0x0C0C, GeneralCategory::Lo), // 8 chars
    (0x0C0E, 0x0C10, GeneralCategory::Lo), // 3 chars
    (0x0C12, 0x0C28, GeneralCategory::Lo), // 23 chars
    (0x0C2A, 0x0C39, GeneralCategory::Lo), // 16 chars
    (0x0C3C, 0x0C3C, GeneralCategory::Mn),
    (0x0C3D, 0x0C3D, GeneralCategory::Lo),
    (0x0C3E, 0x0C40, GeneralCategory::Mn), // 3 chars
    (0x0C41, 0x0C44, GeneralCategory::Mc), // 4 chars
    (0x0C46, 0x0C48, GeneralCategory::Mn), // 3 chars
    (0x0C4A, 0x0C4D, GeneralCategory::Mn), // 4 chars
    (0x0C55, 0x0C56, GeneralCategory::Mn), // 2 chars
    (0x0C58, 0x0C5A, GeneralCategory::Lo), // 3 chars
    (0x0C5C, 0x0C5D, GeneralCategory::Lo), // 2 chars
    (0x0C60, 0x0C61, GeneralCategory::Lo), // 2 chars
    (0x0C62, 0x0C63, GeneralCategory::Mn), // 2 chars
    (0x0C66, 0x0C6F, GeneralCategory::Nd), // 10 chars
    (0x0C77, 0x0C77, GeneralCategory::Po),
    (0x0C78, 0x0C7E, GeneralCategory::No), // 7 chars
    (0x0C7F, 0x0C7F, GeneralCategory::So),
    (0x0C80, 0x0C80, GeneralCategory::Lo),
    (0x0C81, 0x0C81, GeneralCategory::Mn),
    (0x0C82, 0x0C83, GeneralCategory::Mc), // 2 chars
    (0x0C84, 0x0C84, GeneralCategory::Po),
    (0x0C85, 0x0C8C, GeneralCategory::Lo), // 8 chars
    (0x0C8E, 0x0C90, GeneralCategory::Lo), // 3 chars
    (0x0C92, 0x0CA8, GeneralCategory::Lo), // 23 chars
    (0x0CAA, 0x0CB3, GeneralCategory::Lo), // 10 chars
    (0x0CB5, 0x0CB9, GeneralCategory::Lo), // 5 chars
    (0x0CBC, 0x0CBC, GeneralCategory::Mn),
    (0x0CBD, 0x0CBD, GeneralCategory::Lo),
    (0x0CBE, 0x0CBE, GeneralCategory::Mc),
    (0x0CBF, 0x0CBF, GeneralCategory::Mn),
    (0x0CC0, 0x0CC4, GeneralCategory::Mc), // 5 chars
    (0x0CC6, 0x0CC6, GeneralCategory::Mn),
    (0x0CC7, 0x0CC8, GeneralCategory::Mc), // 2 chars
    (0x0CCA, 0x0CCB, GeneralCategory::Mc), // 2 chars
    (0x0CCC, 0x0CCD, GeneralCategory::Mn), // 2 chars
    (0x0CD5, 0x0CD6, GeneralCategory::Mc), // 2 chars
    (0x0CDC, 0x0CDE, GeneralCategory::Lo), // 3 chars
    (0x0CE0, 0x0CE1, GeneralCategory::Lo), // 2 chars
    (0x0CE2, 0x0CE3, GeneralCategory::Mn), // 2 chars
    (0x0CE6, 0x0CEF, GeneralCategory::Nd), // 10 chars
    (0x0CF1, 0x0CF2, GeneralCategory::Lo), // 2 chars
    (0x0CF3, 0x0CF3, GeneralCategory::Mc),
    (0x0D00, 0x0D01, GeneralCategory::Mn), // 2 chars
    (0x0D02, 0x0D03, GeneralCategory::Mc), // 2 chars
    (0x0D04, 0x0D0C, GeneralCategory::Lo), // 9 chars
    (0x0D0E, 0x0D10, GeneralCategory::Lo), // 3 chars
    (0x0D12, 0x0D3A, GeneralCategory::Lo), // 41 chars
    (0x0D3B, 0x0D3C, GeneralCategory::Mn), // 2 chars
    (0x0D3D, 0x0D3D, GeneralCategory::Lo),
    (0x0D3E, 0x0D40, GeneralCategory::Mc), // 3 chars
    (0x0D41, 0x0D44, GeneralCategory::Mn), // 4 chars
    (0x0D46, 0x0D48, GeneralCategory::Mc), // 3 chars
    (0x0D4A, 0x0D4C, GeneralCategory::Mc), // 3 chars
    (0x0D4D, 0x0D4D, GeneralCategory::Mn),
    (0x0D4E, 0x0D4E, GeneralCategory::Lo),
    (0x0D4F, 0x0D4F, GeneralCategory::So),
    (0x0D54, 0x0D56, GeneralCategory::Lo), // 3 chars
    (0x0D57, 0x0D57, GeneralCategory::Mc),
    (0x0D58, 0x0D5E, GeneralCategory::No), // 7 chars
    (0x0D5F, 0x0D61, GeneralCategory::Lo), // 3 chars
    (0x0D62, 0x0D63, GeneralCategory::Mn), // 2 chars
    (0x0D66, 0x0D6F, GeneralCategory::Nd), // 10 chars
    (0x0D70, 0x0D78, GeneralCategory::No), // 9 chars
    (0x0D79, 0x0D79, GeneralCategory::So),
    (0x0D7A, 0x0D7F, GeneralCategory::Lo), // 6 chars
    (0x0D81, 0x0D81, GeneralCategory::Mn),
    (0x0D82, 0x0D83, GeneralCategory::Mc), // 2 chars
    (0x0D85, 0x0D96, GeneralCategory::Lo), // 18 chars
    (0x0D9A, 0x0DB1, GeneralCategory::Lo), // 24 chars
    (0x0DB3, 0x0DBB, GeneralCategory::Lo), // 9 chars
    (0x0DBD, 0x0DBD, GeneralCategory::Lo),
    (0x0DC0, 0x0DC6, GeneralCategory::Lo), // 7 chars
    (0x0DCA, 0x0DCA, GeneralCategory::Mn),
    (0x0DCF, 0x0DD1, GeneralCategory::Mc), // 3 chars
    (0x0DD2, 0x0DD4, GeneralCategory::Mn), // 3 chars
    (0x0DD6, 0x0DD6, GeneralCategory::Mn),
    (0x0DD8, 0x0DDF, GeneralCategory::Mc), // 8 chars
    (0x0DE6, 0x0DEF, GeneralCategory::Nd), // 10 chars
    (0x0DF2, 0x0DF3, GeneralCategory::Mc), // 2 chars
    (0x0DF4, 0x0DF4, GeneralCategory::Po),
    (0x0E01, 0x0E30, GeneralCategory::Lo), // 48 chars
    (0x0E31, 0x0E31, GeneralCategory::Mn),
    (0x0E32, 0x0E33, GeneralCategory::Lo), // 2 chars
    (0x0E34, 0x0E3A, GeneralCategory::Mn), // 7 chars
    (0x0E3F, 0x0E3F, GeneralCategory::Sc),
    (0x0E40, 0x0E45, GeneralCategory::Lo), // 6 chars
    (0x0E46, 0x0E46, GeneralCategory::Lm),
    (0x0E47, 0x0E4E, GeneralCategory::Mn), // 8 chars
    (0x0E4F, 0x0E4F, GeneralCategory::Po),
    (0x0E50, 0x0E59, GeneralCategory::Nd), // 10 chars
    (0x0E5A, 0x0E5B, GeneralCategory::Po), // 2 chars
    (0x0E81, 0x0E82, GeneralCategory::Lo), // 2 chars
    (0x0E84, 0x0E84, GeneralCategory::Lo),
    (0x0E86, 0x0E8A, GeneralCategory::Lo), // 5 chars
    (0x0E8C, 0x0EA3, GeneralCategory::Lo), // 24 chars
    (0x0EA5, 0x0EA5, GeneralCategory::Lo),
    (0x0EA7, 0x0EB0, GeneralCategory::Lo), // 10 chars
    (0x0EB1, 0x0EB1, GeneralCategory::Mn),
    (0x0EB2, 0x0EB3, GeneralCategory::Lo), // 2 chars
    (0x0EB4, 0x0EBC, GeneralCategory::Mn), // 9 chars
    (0x0EBD, 0x0EBD, GeneralCategory::Lo),
    (0x0EC0, 0x0EC4, GeneralCategory::Lo), // 5 chars
    (0x0EC6, 0x0EC6, GeneralCategory::Lm),
    (0x0EC8, 0x0ECE, GeneralCategory::Mn), // 7 chars
    (0x0ED0, 0x0ED9, GeneralCategory::Nd), // 10 chars
    (0x0EDC, 0x0EDF, GeneralCategory::Lo), // 4 chars
    (0x0F00, 0x0F00, GeneralCategory::Lo),
    (0x0F01, 0x0F03, GeneralCategory::So), // 3 chars
    (0x0F04, 0x0F12, GeneralCategory::Po), // 15 chars
    (0x0F13, 0x0F13, GeneralCategory::So),
    (0x0F14, 0x0F14, GeneralCategory::Po),
    (0x0F15, 0x0F17, GeneralCategory::So), // 3 chars
    (0x0F18, 0x0F19, GeneralCategory::Mn), // 2 chars
    (0x0F1A, 0x0F1F, GeneralCategory::So), // 6 chars
    (0x0F20, 0x0F29, GeneralCategory::Nd), // 10 chars
    (0x0F2A, 0x0F33, GeneralCategory::No), // 10 chars
    (0x0F34, 0x0F34, GeneralCategory::So),
    (0x0F35, 0x0F35, GeneralCategory::Mn),
    (0x0F36, 0x0F36, GeneralCategory::So),
    (0x0F37, 0x0F37, GeneralCategory::Mn),
    (0x0F38, 0x0F38, GeneralCategory::So),
    (0x0F39, 0x0F39, GeneralCategory::Mn),
    (0x0F3A, 0x0F3A, GeneralCategory::Ps),
    (0x0F3B, 0x0F3B, GeneralCategory::Pe),
    (0x0F3C, 0x0F3C, GeneralCategory::Ps),
    (0x0F3D, 0x0F3D, GeneralCategory::Pe),
    (0x0F3E, 0x0F3F, GeneralCategory::Mc), // 2 chars
    (0x0F40, 0x0F47, GeneralCategory::Lo), // 8 chars
    (0x0F49, 0x0F6C, GeneralCategory::Lo), // 36 chars
    (0x0F71, 0x0F7E, GeneralCategory::Mn), // 14 chars
    (0x0F7F, 0x0F7F, GeneralCategory::Mc),
    (0x0F80, 0x0F84, GeneralCategory::Mn), // 5 chars
    (0x0F85, 0x0F85, GeneralCategory::Po),
    (0x0F86, 0x0F87, GeneralCategory::Mn), // 2 chars
    (0x0F88, 0x0F8C, GeneralCategory::Lo), // 5 chars
    (0x0F8D, 0x0F97, GeneralCategory::Mn), // 11 chars
    (0x0F99, 0x0FBC, GeneralCategory::Mn), // 36 chars
    (0x0FBE, 0x0FC5, GeneralCategory::So), // 8 chars
    (0x0FC6, 0x0FC6, GeneralCategory::Mn),
    (0x0FC7, 0x0FCC, GeneralCategory::So), // 6 chars
    (0x0FCE, 0x0FCF, GeneralCategory::So), // 2 chars
    (0x0FD0, 0x0FD4, GeneralCategory::Po), // 5 chars
    (0x0FD5, 0x0FD8, GeneralCategory::So), // 4 chars
    (0x0FD9, 0x0FDA, GeneralCategory::Po), // 2 chars
    (0x1000, 0x102A, GeneralCategory::Lo), // 43 chars
    (0x102B, 0x102C, GeneralCategory::Mc), // 2 chars
    (0x102D, 0x1030, GeneralCategory::Mn), // 4 chars
    (0x1031, 0x1031, GeneralCategory::Mc),
    (0x1032, 0x1037, GeneralCategory::Mn), // 6 chars
    (0x1038, 0x1038, GeneralCategory::Mc),
    (0x1039, 0x103A, GeneralCategory::Mn), // 2 chars
    (0x103B, 0x103C, GeneralCategory::Mc), // 2 chars
    (0x103D, 0x103E, GeneralCategory::Mn), // 2 chars
    (0x103F, 0x103F, GeneralCategory::Lo),
    (0x1040, 0x1049, GeneralCategory::Nd), // 10 chars
    (0x104A, 0x104F, GeneralCategory::Po), // 6 chars
    (0x1050, 0x1055, GeneralCategory::Lo), // 6 chars
    (0x1056, 0x1057, GeneralCategory::Mc), // 2 chars
    (0x1058, 0x1059, GeneralCategory::Mn), // 2 chars
    (0x105A, 0x105D, GeneralCategory::Lo), // 4 chars
    (0x105E, 0x1060, GeneralCategory::Mn), // 3 chars
    (0x1061, 0x1061, GeneralCategory::Lo),
    (0x1062, 0x1064, GeneralCategory::Mc), // 3 chars
    (0x1065, 0x1066, GeneralCategory::Lo), // 2 chars
    (0x1067, 0x106D, GeneralCategory::Mc), // 7 chars
    (0x106E, 0x1070, GeneralCategory::Lo), // 3 chars
    (0x1071, 0x1074, GeneralCategory::Mn), // 4 chars
    (0x1075, 0x1081, GeneralCategory::Lo), // 13 chars
    (0x1082, 0x1082, GeneralCategory::Mn),
    (0x1083, 0x1084, GeneralCategory::Mc), // 2 chars
    (0x1085, 0x1086, GeneralCategory::Mn), // 2 chars
    (0x1087, 0x108C, GeneralCategory::Mc), // 6 chars
    (0x108D, 0x108D, GeneralCategory::Mn),
    (0x108E, 0x108E, GeneralCategory::Lo),
    (0x108F, 0x108F, GeneralCategory::Mc),
    (0x1090, 0x1099, GeneralCategory::Nd), // 10 chars
    (0x109A, 0x109C, GeneralCategory::Mc), // 3 chars
    (0x109D, 0x109D, GeneralCategory::Mn),
    (0x109E, 0x109F, GeneralCategory::So), // 2 chars
    (0x10A0, 0x10C5, GeneralCategory::Lu), // 38 chars
    (0x10C7, 0x10C7, GeneralCategory::Lu),
    (0x10CD, 0x10CD, GeneralCategory::Lu),
    (0x10D0, 0x10FA, GeneralCategory::Ll), // 43 chars
    (0x10FB, 0x10FB, GeneralCategory::Po),
    (0x10FC, 0x10FC, GeneralCategory::Lm),
    (0x10FD, 0x10FF, GeneralCategory::Ll), // 3 chars
    (0x1100, 0x1248, GeneralCategory::Lo), // 329 chars
    (0x124A, 0x124D, GeneralCategory::Lo), // 4 chars
    (0x1250, 0x1256, GeneralCategory::Lo), // 7 chars
    (0x1258, 0x1258, GeneralCategory::Lo),
    (0x125A, 0x125D, GeneralCategory::Lo), // 4 chars
    (0x1260, 0x1288, GeneralCategory::Lo), // 41 chars
    (0x128A, 0x128D, GeneralCategory::Lo), // 4 chars
    (0x1290, 0x12B0, GeneralCategory::Lo), // 33 chars
    (0x12B2, 0x12B5, GeneralCategory::Lo), // 4 chars
    (0x12B8, 0x12BE, GeneralCategory::Lo), // 7 chars
    (0x12C0, 0x12C0, GeneralCategory::Lo),
    (0x12C2, 0x12C5, GeneralCategory::Lo), // 4 chars
    (0x12C8, 0x12D6, GeneralCategory::Lo), // 15 chars
    (0x12D8, 0x1310, GeneralCategory::Lo), // 57 chars
    (0x1312, 0x1315, GeneralCategory::Lo), // 4 chars
    (0x1318, 0x135A, GeneralCategory::Lo), // 67 chars
    (0x135D, 0x135F, GeneralCategory::Mn), // 3 chars
    (0x1360, 0x1368, GeneralCategory::Po), // 9 chars
    (0x1369, 0x137C, GeneralCategory::No), // 20 chars
    (0x1380, 0x138F, GeneralCategory::Lo), // 16 chars
    (0x1390, 0x1399, GeneralCategory::So), // 10 chars
    (0x13A0, 0x13F5, GeneralCategory::Lu), // 86 chars
    (0x13F8, 0x13FD, GeneralCategory::Ll), // 6 chars
    (0x1400, 0x1400, GeneralCategory::Pd),
    (0x1401, 0x166C, GeneralCategory::Lo), // 620 chars
    (0x166D, 0x166D, GeneralCategory::So),
    (0x166E, 0x166E, GeneralCategory::Po),
    (0x166F, 0x167F, GeneralCategory::Lo), // 17 chars
    (0x1680, 0x1680, GeneralCategory::Zs),
    (0x1681, 0x169A, GeneralCategory::Lo), // 26 chars
    (0x169B, 0x169B, GeneralCategory::Ps),
    (0x169C, 0x169C, GeneralCategory::Pe),
    (0x16A0, 0x16EA, GeneralCategory::Lo), // 75 chars
    (0x16EB, 0x16ED, GeneralCategory::Po), // 3 chars
    (0x16EE, 0x16F0, GeneralCategory::Nl), // 3 chars
    (0x16F1, 0x16F8, GeneralCategory::Lo), // 8 chars
    (0x1700, 0x1711, GeneralCategory::Lo), // 18 chars
    (0x1712, 0x1714, GeneralCategory::Mn), // 3 chars
    (0x1715, 0x1715, GeneralCategory::Mc),
    (0x171F, 0x1731, GeneralCategory::Lo), // 19 chars
    (0x1732, 0x1733, GeneralCategory::Mn), // 2 chars
    (0x1734, 0x1734, GeneralCategory::Mc),
    (0x1735, 0x1736, GeneralCategory::Po), // 2 chars
    (0x1740, 0x1751, GeneralCategory::Lo), // 18 chars
    (0x1752, 0x1753, GeneralCategory::Mn), // 2 chars
    (0x1760, 0x176C, GeneralCategory::Lo), // 13 chars
    (0x176E, 0x1770, GeneralCategory::Lo), // 3 chars
    (0x1772, 0x1773, GeneralCategory::Mn), // 2 chars
    (0x1780, 0x17B3, GeneralCategory::Lo), // 52 chars
    (0x17B4, 0x17B5, GeneralCategory::Mn), // 2 chars
    (0x17B6, 0x17B6, GeneralCategory::Mc),
    (0x17B7, 0x17BD, GeneralCategory::Mn), // 7 chars
    (0x17BE, 0x17C5, GeneralCategory::Mc), // 8 chars
    (0x17C6, 0x17C6, GeneralCategory::Mn),
    (0x17C7, 0x17C8, GeneralCategory::Mc), // 2 chars
    (0x17C9, 0x17D3, GeneralCategory::Mn), // 11 chars
    (0x17D4, 0x17D6, GeneralCategory::Po), // 3 chars
    (0x17D7, 0x17D7, GeneralCategory::Lm),
    (0x17D8, 0x17DA, GeneralCategory::Po), // 3 chars
    (0x17DB, 0x17DB, GeneralCategory::Sc),
    (0x17DC, 0x17DC, GeneralCategory::Lo),
    (0x17DD, 0x17DD, GeneralCategory::Mn),
    (0x17E0, 0x17E9, GeneralCategory::Nd), // 10 chars
    (0x17F0, 0x17F9, GeneralCategory::No), // 10 chars
    (0x1800, 0x1805, GeneralCategory::Po), // 6 chars
    (0x1806, 0x1806, GeneralCategory::Pd),
    (0x1807, 0x180A, GeneralCategory::Po), // 4 chars
    (0x180B, 0x180D, GeneralCategory::Mn), // 3 chars
    (0x180E, 0x180E, GeneralCategory::Cf),
    (0x180F, 0x180F, GeneralCategory::Mn),
    (0x1810, 0x1819, GeneralCategory::Nd), // 10 chars
    (0x1820, 0x1842, GeneralCategory::Lo), // 35 chars
    (0x1843, 0x1843, GeneralCategory::Lm),
    (0x1844, 0x1878, GeneralCategory::Lo), // 53 chars
    (0x1880, 0x1884, GeneralCategory::Lo), // 5 chars
    (0x1885, 0x1886, GeneralCategory::Mn), // 2 chars
    (0x1887, 0x18A8, GeneralCategory::Lo), // 34 chars
    (0x18A9, 0x18A9, GeneralCategory::Mn),
    (0x18AA, 0x18AA, GeneralCategory::Lo),
    (0x18B0, 0x18F5, GeneralCategory::Lo), // 70 chars
    (0x1900, 0x191E, GeneralCategory::Lo), // 31 chars
    (0x1920, 0x1922, GeneralCategory::Mn), // 3 chars
    (0x1923, 0x1926, GeneralCategory::Mc), // 4 chars
    (0x1927, 0x1928, GeneralCategory::Mn), // 2 chars
    (0x1929, 0x192B, GeneralCategory::Mc), // 3 chars
    (0x1930, 0x1931, GeneralCategory::Mc), // 2 chars
    (0x1932, 0x1932, GeneralCategory::Mn),
    (0x1933, 0x1938, GeneralCategory::Mc), // 6 chars
    (0x1939, 0x193B, GeneralCategory::Mn), // 3 chars
    (0x1940, 0x1940, GeneralCategory::So),
    (0x1944, 0x1945, GeneralCategory::Po), // 2 chars
    (0x1946, 0x194F, GeneralCategory::Nd), // 10 chars
    (0x1950, 0x196D, GeneralCategory::Lo), // 30 chars
    (0x1970, 0x1974, GeneralCategory::Lo), // 5 chars
    (0x1980, 0x19AB, GeneralCategory::Lo), // 44 chars
    (0x19B0, 0x19C9, GeneralCategory::Lo), // 26 chars
    (0x19D0, 0x19D9, GeneralCategory::Nd), // 10 chars
    (0x19DA, 0x19DA, GeneralCategory::No),
    (0x19DE, 0x19FF, GeneralCategory::So), // 34 chars
    (0x1A00, 0x1A16, GeneralCategory::Lo), // 23 chars
    (0x1A17, 0x1A18, GeneralCategory::Mn), // 2 chars
    (0x1A19, 0x1A1A, GeneralCategory::Mc), // 2 chars
    (0x1A1B, 0x1A1B, GeneralCategory::Mn),
    (0x1A1E, 0x1A1F, GeneralCategory::Po), // 2 chars
    (0x1A20, 0x1A54, GeneralCategory::Lo), // 53 chars
    (0x1A55, 0x1A55, GeneralCategory::Mc),
    (0x1A56, 0x1A56, GeneralCategory::Mn),
    (0x1A57, 0x1A57, GeneralCategory::Mc),
    (0x1A58, 0x1A5E, GeneralCategory::Mn), // 7 chars
    (0x1A60, 0x1A60, GeneralCategory::Mn),
    (0x1A61, 0x1A61, GeneralCategory::Mc),
    (0x1A62, 0x1A62, GeneralCategory::Mn),
    (0x1A63, 0x1A64, GeneralCategory::Mc), // 2 chars
    (0x1A65, 0x1A6C, GeneralCategory::Mn), // 8 chars
    (0x1A6D, 0x1A72, GeneralCategory::Mc), // 6 chars
    (0x1A73, 0x1A7C, GeneralCategory::Mn), // 10 chars
    (0x1A7F, 0x1A7F, GeneralCategory::Mn),
    (0x1A80, 0x1A89, GeneralCategory::Nd), // 10 chars
    (0x1A90, 0x1A99, GeneralCategory::Nd), // 10 chars
    (0x1AA0, 0x1AA6, GeneralCategory::Po), // 7 chars
    (0x1AA7, 0x1AA7, GeneralCategory::Lm),
    (0x1AA8, 0x1AAD, GeneralCategory::Po), // 6 chars
    (0x1AB0, 0x1ABD, GeneralCategory::Mn), // 14 chars
    (0x1ABE, 0x1ABE, GeneralCategory::Me),
    (0x1ABF, 0x1ADD, GeneralCategory::Mn), // 31 chars
    (0x1AE0, 0x1AEB, GeneralCategory::Mn), // 12 chars
    (0x1B00, 0x1B03, GeneralCategory::Mn), // 4 chars
    (0x1B04, 0x1B04, GeneralCategory::Mc),
    (0x1B05, 0x1B33, GeneralCategory::Lo), // 47 chars
    (0x1B34, 0x1B34, GeneralCategory::Mn),
    (0x1B35, 0x1B35, GeneralCategory::Mc),
    (0x1B36, 0x1B3A, GeneralCategory::Mn), // 5 chars
    (0x1B3B, 0x1B3B, GeneralCategory::Mc),
    (0x1B3C, 0x1B3C, GeneralCategory::Mn),
    (0x1B3D, 0x1B41, GeneralCategory::Mc), // 5 chars
    (0x1B42, 0x1B42, GeneralCategory::Mn),
    (0x1B43, 0x1B44, GeneralCategory::Mc), // 2 chars
    (0x1B45, 0x1B4C, GeneralCategory::Lo), // 8 chars
    (0x1B4E, 0x1B4F, GeneralCategory::Po), // 2 chars
    (0x1B50, 0x1B59, GeneralCategory::Nd), // 10 chars
    (0x1B5A, 0x1B60, GeneralCategory::Po), // 7 chars
    (0x1B61, 0x1B6A, GeneralCategory::So), // 10 chars
    (0x1B6B, 0x1B73, GeneralCategory::Mn), // 9 chars
    (0x1B74, 0x1B7C, GeneralCategory::So), // 9 chars
    (0x1B7D, 0x1B7F, GeneralCategory::Po), // 3 chars
    (0x1B80, 0x1B81, GeneralCategory::Mn), // 2 chars
    (0x1B82, 0x1B82, GeneralCategory::Mc),
    (0x1B83, 0x1BA0, GeneralCategory::Lo), // 30 chars
    (0x1BA1, 0x1BA1, GeneralCategory::Mc),
    (0x1BA2, 0x1BA5, GeneralCategory::Mn), // 4 chars
    (0x1BA6, 0x1BA7, GeneralCategory::Mc), // 2 chars
    (0x1BA8, 0x1BA9, GeneralCategory::Mn), // 2 chars
    (0x1BAA, 0x1BAA, GeneralCategory::Mc),
    (0x1BAB, 0x1BAD, GeneralCategory::Mn), // 3 chars
    (0x1BAE, 0x1BAF, GeneralCategory::Lo), // 2 chars
    (0x1BB0, 0x1BB9, GeneralCategory::Nd), // 10 chars
    (0x1BBA, 0x1BE5, GeneralCategory::Lo), // 44 chars
    (0x1BE6, 0x1BE6, GeneralCategory::Mn),
    (0x1BE7, 0x1BE7, GeneralCategory::Mc),
    (0x1BE8, 0x1BE9, GeneralCategory::Mn), // 2 chars
    (0x1BEA, 0x1BEC, GeneralCategory::Mc), // 3 chars
    (0x1BED, 0x1BED, GeneralCategory::Mn),
    (0x1BEE, 0x1BEE, GeneralCategory::Mc),
    (0x1BEF, 0x1BF1, GeneralCategory::Mn), // 3 chars
    (0x1BF2, 0x1BF3, GeneralCategory::Mc), // 2 chars
    (0x1BFC, 0x1BFF, GeneralCategory::Po), // 4 chars
    (0x1C00, 0x1C23, GeneralCategory::Lo), // 36 chars
    (0x1C24, 0x1C2B, GeneralCategory::Mc), // 8 chars
    (0x1C2C, 0x1C33, GeneralCategory::Mn), // 8 chars
    (0x1C34, 0x1C35, GeneralCategory::Mc), // 2 chars
    (0x1C36, 0x1C37, GeneralCategory::Mn), // 2 chars
    (0x1C3B, 0x1C3F, GeneralCategory::Po), // 5 chars
    (0x1C40, 0x1C49, GeneralCategory::Nd), // 10 chars
    (0x1C4D, 0x1C4F, GeneralCategory::Lo), // 3 chars
    (0x1C50, 0x1C59, GeneralCategory::Nd), // 10 chars
    (0x1C5A, 0x1C77, GeneralCategory::Lo), // 30 chars
    (0x1C78, 0x1C7D, GeneralCategory::Lm), // 6 chars
    (0x1C7E, 0x1C7F, GeneralCategory::Po), // 2 chars
    (0x1C80, 0x1C88, GeneralCategory::Ll), // 9 chars
    (0x1C89, 0x1C89, GeneralCategory::Lu),
    (0x1C8A, 0x1C8A, GeneralCategory::Ll),
    (0x1C90, 0x1CBA, GeneralCategory::Lu), // 43 chars
    (0x1CBD, 0x1CBF, GeneralCategory::Lu), // 3 chars
    (0x1CC0, 0x1CC7, GeneralCategory::Po), // 8 chars
    (0x1CD0, 0x1CD2, GeneralCategory::Mn), // 3 chars
    (0x1CD3, 0x1CD3, GeneralCategory::Po),
    (0x1CD4, 0x1CE0, GeneralCategory::Mn), // 13 chars
    (0x1CE1, 0x1CE1, GeneralCategory::Mc),
    (0x1CE2, 0x1CE8, GeneralCategory::Mn), // 7 chars
    (0x1CE9, 0x1CEC, GeneralCategory::Lo), // 4 chars
    (0x1CED, 0x1CED, GeneralCategory::Mn),
    (0x1CEE, 0x1CF3, GeneralCategory::Lo), // 6 chars
    (0x1CF4, 0x1CF4, GeneralCategory::Mn),
    (0x1CF5, 0x1CF6, GeneralCategory::Lo), // 2 chars
    (0x1CF7, 0x1CF7, GeneralCategory::Mc),
    (0x1CF8, 0x1CF9, GeneralCategory::Mn), // 2 chars
    (0x1CFA, 0x1CFA, GeneralCategory::Lo),
    (0x1D00, 0x1D2B, GeneralCategory::Ll), // 44 chars
    (0x1D2C, 0x1D6A, GeneralCategory::Lm), // 63 chars
    (0x1D6B, 0x1D77, GeneralCategory::Ll), // 13 chars
    (0x1D78, 0x1D78, GeneralCategory::Lm),
    (0x1D79, 0x1D9A, GeneralCategory::Ll), // 34 chars
    (0x1D9B, 0x1DBF, GeneralCategory::Lm), // 37 chars
    (0x1DC0, 0x1DFF, GeneralCategory::Mn), // 64 chars
    (0x1E00, 0x1E00, GeneralCategory::Lu),
    (0x1E01, 0x1E01, GeneralCategory::Ll),
    (0x1E02, 0x1E02, GeneralCategory::Lu),
    (0x1E03, 0x1E03, GeneralCategory::Ll),
    (0x1E04, 0x1E04, GeneralCategory::Lu),
    (0x1E05, 0x1E05, GeneralCategory::Ll),
    (0x1E06, 0x1E06, GeneralCategory::Lu),
    (0x1E07, 0x1E07, GeneralCategory::Ll),
    (0x1E08, 0x1E08, GeneralCategory::Lu),
    (0x1E09, 0x1E09, GeneralCategory::Ll),
    (0x1E0A, 0x1E0A, GeneralCategory::Lu),
    (0x1E0B, 0x1E0B, GeneralCategory::Ll),
    (0x1E0C, 0x1E0C, GeneralCategory::Lu),
    (0x1E0D, 0x1E0D, GeneralCategory::Ll),
    (0x1E0E, 0x1E0E, GeneralCategory::Lu),
    (0x1E0F, 0x1E0F, GeneralCategory::Ll),
    (0x1E10, 0x1E10, GeneralCategory::Lu),
    (0x1E11, 0x1E11, GeneralCategory::Ll),
    (0x1E12, 0x1E12, GeneralCategory::Lu),
    (0x1E13, 0x1E13, GeneralCategory::Ll),
    (0x1E14, 0x1E14, GeneralCategory::Lu),
    (0x1E15, 0x1E15, GeneralCategory::Ll),
    (0x1E16, 0x1E16, GeneralCategory::Lu),
    (0x1E17, 0x1E17, GeneralCategory::Ll),
    (0x1E18, 0x1E18, GeneralCategory::Lu),
    (0x1E19, 0x1E19, GeneralCategory::Ll),
    (0x1E1A, 0x1E1A, GeneralCategory::Lu),
    (0x1E1B, 0x1E1B, GeneralCategory::Ll),
    (0x1E1C, 0x1E1C, GeneralCategory::Lu),
    (0x1E1D, 0x1E1D, GeneralCategory::Ll),
    (0x1E1E, 0x1E1E, GeneralCategory::Lu),
    (0x1E1F, 0x1E1F, GeneralCategory::Ll),
    (0x1E20, 0x1E20, GeneralCategory::Lu),
    (0x1E21, 0x1E21, GeneralCategory::Ll),
    (0x1E22, 0x1E22, GeneralCategory::Lu),
    (0x1E23, 0x1E23, GeneralCategory::Ll),
    (0x1E24, 0x1E24, GeneralCategory::Lu),
    (0x1E25, 0x1E25, GeneralCategory::Ll),
    (0x1E26, 0x1E26, GeneralCategory::Lu),
    (0x1E27, 0x1E27, GeneralCategory::Ll),
    (0x1E28, 0x1E28, GeneralCategory::Lu),
    (0x1E29, 0x1E29, GeneralCategory::Ll),
    (0x1E2A, 0x1E2A, GeneralCategory::Lu),
    (0x1E2B, 0x1E2B, GeneralCategory::Ll),
    (0x1E2C, 0x1E2C, GeneralCategory::Lu),
    (0x1E2D, 0x1E2D, GeneralCategory::Ll),
    (0x1E2E, 0x1E2E, GeneralCategory::Lu),
    (0x1E2F, 0x1E2F, GeneralCategory::Ll),
    (0x1E30, 0x1E30, GeneralCategory::Lu),
    (0x1E31, 0x1E31, GeneralCategory::Ll),
    (0x1E32, 0x1E32, GeneralCategory::Lu),
    (0x1E33, 0x1E33, GeneralCategory::Ll),
    (0x1E34, 0x1E34, GeneralCategory::Lu),
    (0x1E35, 0x1E35, GeneralCategory::Ll),
    (0x1E36, 0x1E36, GeneralCategory::Lu),
    (0x1E37, 0x1E37, GeneralCategory::Ll),
    (0x1E38, 0x1E38, GeneralCategory::Lu),
    (0x1E39, 0x1E39, GeneralCategory::Ll),
    (0x1E3A, 0x1E3A, GeneralCategory::Lu),
    (0x1E3B, 0x1E3B, GeneralCategory::Ll),
    (0x1E3C, 0x1E3C, GeneralCategory::Lu),
    (0x1E3D, 0x1E3D, GeneralCategory::Ll),
    (0x1E3E, 0x1E3E, GeneralCategory::Lu),
    (0x1E3F, 0x1E3F, GeneralCategory::Ll),
    (0x1E40, 0x1E40, GeneralCategory::Lu),
    (0x1E41, 0x1E41, GeneralCategory::Ll),
    (0x1E42, 0x1E42, GeneralCategory::Lu),
    (0x1E43, 0x1E43, GeneralCategory::Ll),
    (0x1E44, 0x1E44, GeneralCategory::Lu),
    (0x1E45, 0x1E45, GeneralCategory::Ll),
    (0x1E46, 0x1E46, GeneralCategory::Lu),
    (0x1E47, 0x1E47, GeneralCategory::Ll),
    (0x1E48, 0x1E48, GeneralCategory::Lu),
    (0x1E49, 0x1E49, GeneralCategory::Ll),
    (0x1E4A, 0x1E4A, GeneralCategory::Lu),
    (0x1E4B, 0x1E4B, GeneralCategory::Ll),
    (0x1E4C, 0x1E4C, GeneralCategory::Lu),
    (0x1E4D, 0x1E4D, GeneralCategory::Ll),
    (0x1E4E, 0x1E4E, GeneralCategory::Lu),
    (0x1E4F, 0x1E4F, GeneralCategory::Ll),
    (0x1E50, 0x1E50, GeneralCategory::Lu),
    (0x1E51, 0x1E51, GeneralCategory::Ll),
    (0x1E52, 0x1E52, GeneralCategory::Lu),
    (0x1E53, 0x1E53, GeneralCategory::Ll),
    (0x1E54, 0x1E54, GeneralCategory::Lu),
    (0x1E55, 0x1E55, GeneralCategory::Ll),
    (0x1E56, 0x1E56, GeneralCategory::Lu),
    (0x1E57, 0x1E57, GeneralCategory::Ll),
    (0x1E58, 0x1E58, GeneralCategory::Lu),
    (0x1E59, 0x1E59, GeneralCategory::Ll),
    (0x1E5A, 0x1E5A, GeneralCategory::Lu),
    (0x1E5B, 0x1E5B, GeneralCategory::Ll),
    (0x1E5C, 0x1E5C, GeneralCategory::Lu),
    (0x1E5D, 0x1E5D, GeneralCategory::Ll),
    (0x1E5E, 0x1E5E, GeneralCategory::Lu),
    (0x1E5F, 0x1E5F, GeneralCategory::Ll),
    (0x1E60, 0x1E60, GeneralCategory::Lu),
    (0x1E61, 0x1E61, GeneralCategory::Ll),
    (0x1E62, 0x1E62, GeneralCategory::Lu),
    (0x1E63, 0x1E63, GeneralCategory::Ll),
    (0x1E64, 0x1E64, GeneralCategory::Lu),
    (0x1E65, 0x1E65, GeneralCategory::Ll),
    (0x1E66, 0x1E66, GeneralCategory::Lu),
    (0x1E67, 0x1E67, GeneralCategory::Ll),
    (0x1E68, 0x1E68, GeneralCategory::Lu),
    (0x1E69, 0x1E69, GeneralCategory::Ll),
    (0x1E6A, 0x1E6A, GeneralCategory::Lu),
    (0x1E6B, 0x1E6B, GeneralCategory::Ll),
    (0x1E6C, 0x1E6C, GeneralCategory::Lu),
    (0x1E6D, 0x1E6D, GeneralCategory::Ll),
    (0x1E6E, 0x1E6E, GeneralCategory::Lu),
    (0x1E6F, 0x1E6F, GeneralCategory::Ll),
    (0x1E70, 0x1E70, GeneralCategory::Lu),
    (0x1E71, 0x1E71, GeneralCategory::Ll),
    (0x1E72, 0x1E72, GeneralCategory::Lu),
    (0x1E73, 0x1E73, GeneralCategory::Ll),
    (0x1E74, 0x1E74, GeneralCategory::Lu),
    (0x1E75, 0x1E75, GeneralCategory::Ll),
    (0x1E76, 0x1E76, GeneralCategory::Lu),
    (0x1E77, 0x1E77, GeneralCategory::Ll),
    (0x1E78, 0x1E78, GeneralCategory::Lu),
    (0x1E79, 0x1E79, GeneralCategory::Ll),
    (0x1E7A, 0x1E7A, GeneralCategory::Lu),
    (0x1E7B, 0x1E7B, GeneralCategory::Ll),
    (0x1E7C, 0x1E7C, GeneralCategory::Lu),
    (0x1E7D, 0x1E7D, GeneralCategory::Ll),
    (0x1E7E, 0x1E7E, GeneralCategory::Lu),
    (0x1E7F, 0x1E7F, GeneralCategory::Ll),
    (0x1E80, 0x1E80, GeneralCategory::Lu),
    (0x1E81, 0x1E81, GeneralCategory::Ll),
    (0x1E82, 0x1E82, GeneralCategory::Lu),
    (0x1E83, 0x1E83, GeneralCategory::Ll),
    (0x1E84, 0x1E84, GeneralCategory::Lu),
    (0x1E85, 0x1E85, GeneralCategory::Ll),
    (0x1E86, 0x1E86, GeneralCategory::Lu),
    (0x1E87, 0x1E87, GeneralCategory::Ll),
    (0x1E88, 0x1E88, GeneralCategory::Lu),
    (0x1E89, 0x1E89, GeneralCategory::Ll),
    (0x1E8A, 0x1E8A, GeneralCategory::Lu),
    (0x1E8B, 0x1E8B, GeneralCategory::Ll),
    (0x1E8C, 0x1E8C, GeneralCategory::Lu),
    (0x1E8D, 0x1E8D, GeneralCategory::Ll),
    (0x1E8E, 0x1E8E, GeneralCategory::Lu),
    (0x1E8F, 0x1E8F, GeneralCategory::Ll),
    (0x1E90, 0x1E90, GeneralCategory::Lu),
    (0x1E91, 0x1E91, GeneralCategory::Ll),
    (0x1E92, 0x1E92, GeneralCategory::Lu),
    (0x1E93, 0x1E93, GeneralCategory::Ll),
    (0x1E94, 0x1E94, GeneralCategory::Lu),
    (0x1E95, 0x1E9D, GeneralCategory::Ll), // 9 chars
    (0x1E9E, 0x1E9E, GeneralCategory::Lu),
    (0x1E9F, 0x1E9F, GeneralCategory::Ll),
    (0x1EA0, 0x1EA0, GeneralCategory::Lu),
    (0x1EA1, 0x1EA1, GeneralCategory::Ll),
    (0x1EA2, 0x1EA2, GeneralCategory::Lu),
    (0x1EA3, 0x1EA3, GeneralCategory::Ll),
    (0x1EA4, 0x1EA4, GeneralCategory::Lu),
    (0x1EA5, 0x1EA5, GeneralCategory::Ll),
    (0x1EA6, 0x1EA6, GeneralCategory::Lu),
    (0x1EA7, 0x1EA7, GeneralCategory::Ll),
    (0x1EA8, 0x1EA8, GeneralCategory::Lu),
    (0x1EA9, 0x1EA9, GeneralCategory::Ll),
    (0x1EAA, 0x1EAA, GeneralCategory::Lu),
    (0x1EAB, 0x1EAB, GeneralCategory::Ll),
    (0x1EAC, 0x1EAC, GeneralCategory::Lu),
    (0x1EAD, 0x1EAD, GeneralCategory::Ll),
    (0x1EAE, 0x1EAE, GeneralCategory::Lu),
    (0x1EAF, 0x1EAF, GeneralCategory::Ll),
    (0x1EB0, 0x1EB0, GeneralCategory::Lu),
    (0x1EB1, 0x1EB1, GeneralCategory::Ll),
    (0x1EB2, 0x1EB2, GeneralCategory::Lu),
    (0x1EB3, 0x1EB3, GeneralCategory::Ll),
    (0x1EB4, 0x1EB4, GeneralCategory::Lu),
    (0x1EB5, 0x1EB5, GeneralCategory::Ll),
    (0x1EB6, 0x1EB6, GeneralCategory::Lu),
    (0x1EB7, 0x1EB7, GeneralCategory::Ll),
    (0x1EB8, 0x1EB8, GeneralCategory::Lu),
    (0x1EB9, 0x1EB9, GeneralCategory::Ll),
    (0x1EBA, 0x1EBA, GeneralCategory::Lu),
    (0x1EBB, 0x1EBB, GeneralCategory::Ll),
    (0x1EBC, 0x1EBC, GeneralCategory::Lu),
    (0x1EBD, 0x1EBD, GeneralCategory::Ll),
    (0x1EBE, 0x1EBE, GeneralCategory::Lu),
    (0x1EBF, 0x1EBF, GeneralCategory::Ll),
    (0x1EC0, 0x1EC0, GeneralCategory::Lu),
    (0x1EC1, 0x1EC1, GeneralCategory::Ll),
    (0x1EC2, 0x1EC2, GeneralCategory::Lu),
    (0x1EC3, 0x1EC3, GeneralCategory::Ll),
    (0x1EC4, 0x1EC4, GeneralCategory::Lu),
    (0x1EC5, 0x1EC5, GeneralCategory::Ll),
    (0x1EC6, 0x1EC6, GeneralCategory::Lu),
    (0x1EC7, 0x1EC7, GeneralCategory::Ll),
    (0x1EC8, 0x1EC8, GeneralCategory::Lu),
    (0x1EC9, 0x1EC9, GeneralCategory::Ll),
    (0x1ECA, 0x1ECA, GeneralCategory::Lu),
    (0x1ECB, 0x1ECB, GeneralCategory::Ll),
    (0x1ECC, 0x1ECC, GeneralCategory::Lu),
    (0x1ECD, 0x1ECD, GeneralCategory::Ll),
    (0x1ECE, 0x1ECE, GeneralCategory::Lu),
    (0x1ECF, 0x1ECF, GeneralCategory::Ll),
    (0x1ED0, 0x1ED0, GeneralCategory::Lu),
    (0x1ED1, 0x1ED1, GeneralCategory::Ll),
    (0x1ED2, 0x1ED2, GeneralCategory::Lu),
    (0x1ED3, 0x1ED3, GeneralCategory::Ll),
    (0x1ED4, 0x1ED4, GeneralCategory::Lu),
    (0x1ED5, 0x1ED5, GeneralCategory::Ll),
    (0x1ED6, 0x1ED6, GeneralCategory::Lu),
    (0x1ED7, 0x1ED7, GeneralCategory::Ll),
    (0x1ED8, 0x1ED8, GeneralCategory::Lu),
    (0x1ED9, 0x1ED9, GeneralCategory::Ll),
    (0x1EDA, 0x1EDA, GeneralCategory::Lu),
    (0x1EDB, 0x1EDB, GeneralCategory::Ll),
    (0x1EDC, 0x1EDC, GeneralCategory::Lu),
    (0x1EDD, 0x1EDD, GeneralCategory::Ll),
    (0x1EDE, 0x1EDE, GeneralCategory::Lu),
    (0x1EDF, 0x1EDF, GeneralCategory::Ll),
    (0x1EE0, 0x1EE0, GeneralCategory::Lu),
    (0x1EE1, 0x1EE1, GeneralCategory::Ll),
    (0x1EE2, 0x1EE2, GeneralCategory::Lu),
    (0x1EE3, 0x1EE3, GeneralCategory::Ll),
    (0x1EE4, 0x1EE4, GeneralCategory::Lu),
    (0x1EE5, 0x1EE5, GeneralCategory::Ll),
    (0x1EE6, 0x1EE6, GeneralCategory::Lu),
    (0x1EE7, 0x1EE7, GeneralCategory::Ll),
    (0x1EE8, 0x1EE8, GeneralCategory::Lu),
    (0x1EE9, 0x1EE9, GeneralCategory::Ll),
    (0x1EEA, 0x1EEA, GeneralCategory::Lu),
    (0x1EEB, 0x1EEB, GeneralCategory::Ll),
    (0x1EEC, 0x1EEC, GeneralCategory::Lu),
    (0x1EED, 0x1EED, GeneralCategory::Ll),
    (0x1EEE, 0x1EEE, GeneralCategory::Lu),
    (0x1EEF, 0x1EEF, GeneralCategory::Ll),
    (0x1EF0, 0x1EF0, GeneralCategory::Lu),
    (0x1EF1, 0x1EF1, GeneralCategory::Ll),
    (0x1EF2, 0x1EF2, GeneralCategory::Lu),
    (0x1EF3, 0x1EF3, GeneralCategory::Ll),
    (0x1EF4, 0x1EF4, GeneralCategory::Lu),
    (0x1EF5, 0x1EF5, GeneralCategory::Ll),
    (0x1EF6, 0x1EF6, GeneralCategory::Lu),
    (0x1EF7, 0x1EF7, GeneralCategory::Ll),
    (0x1EF8, 0x1EF8, GeneralCategory::Lu),
    (0x1EF9, 0x1EF9, GeneralCategory::Ll),
    (0x1EFA, 0x1EFA, GeneralCategory::Lu),
    (0x1EFB, 0x1EFB, GeneralCategory::Ll),
    (0x1EFC, 0x1EFC, GeneralCategory::Lu),
    (0x1EFD, 0x1EFD, GeneralCategory::Ll),
    (0x1EFE, 0x1EFE, GeneralCategory::Lu),
    (0x1EFF, 0x1F07, GeneralCategory::Ll), // 9 chars
    (0x1F08, 0x1F0F, GeneralCategory::Lu), // 8 chars
    (0x1F10, 0x1F15, GeneralCategory::Ll), // 6 chars
    (0x1F18, 0x1F1D, GeneralCategory::Lu), // 6 chars
    (0x1F20, 0x1F27, GeneralCategory::Ll), // 8 chars
    (0x1F28, 0x1F2F, GeneralCategory::Lu), // 8 chars
    (0x1F30, 0x1F37, GeneralCategory::Ll), // 8 chars
    (0x1F38, 0x1F3F, GeneralCategory::Lu), // 8 chars
    (0x1F40, 0x1F45, GeneralCategory::Ll), // 6 chars
    (0x1F48, 0x1F4D, GeneralCategory::Lu), // 6 chars
    (0x1F50, 0x1F57, GeneralCategory::Ll), // 8 chars
    (0x1F59, 0x1F59, GeneralCategory::Lu),
    (0x1F5B, 0x1F5B, GeneralCategory::Lu),
    (0x1F5D, 0x1F5D, GeneralCategory::Lu),
    (0x1F5F, 0x1F5F, GeneralCategory::Lu),
    (0x1F60, 0x1F67, GeneralCategory::Ll), // 8 chars
    (0x1F68, 0x1F6F, GeneralCategory::Lu), // 8 chars
    (0x1F70, 0x1F7D, GeneralCategory::Ll), // 14 chars
    (0x1F80, 0x1F87, GeneralCategory::Ll), // 8 chars
    (0x1F88, 0x1F8F, GeneralCategory::Lt), // 8 chars
    (0x1F90, 0x1F97, GeneralCategory::Ll), // 8 chars
    (0x1F98, 0x1F9F, GeneralCategory::Lt), // 8 chars
    (0x1FA0, 0x1FA7, GeneralCategory::Ll), // 8 chars
    (0x1FA8, 0x1FAF, GeneralCategory::Lt), // 8 chars
    (0x1FB0, 0x1FB4, GeneralCategory::Ll), // 5 chars
    (0x1FB6, 0x1FB7, GeneralCategory::Ll), // 2 chars
    (0x1FB8, 0x1FBB, GeneralCategory::Lu), // 4 chars
    (0x1FBC, 0x1FBC, GeneralCategory::Lt),
    (0x1FBD, 0x1FBD, GeneralCategory::Sk),
    (0x1FBE, 0x1FBE, GeneralCategory::Ll),
    (0x1FBF, 0x1FC1, GeneralCategory::Sk), // 3 chars
    (0x1FC2, 0x1FC4, GeneralCategory::Ll), // 3 chars
    (0x1FC6, 0x1FC7, GeneralCategory::Ll), // 2 chars
    (0x1FC8, 0x1FCB, GeneralCategory::Lu), // 4 chars
    (0x1FCC, 0x1FCC, GeneralCategory::Lt),
    (0x1FCD, 0x1FCF, GeneralCategory::Sk), // 3 chars
    (0x1FD0, 0x1FD3, GeneralCategory::Ll), // 4 chars
    (0x1FD6, 0x1FD7, GeneralCategory::Ll), // 2 chars
    (0x1FD8, 0x1FDB, GeneralCategory::Lu), // 4 chars
    (0x1FDD, 0x1FDF, GeneralCategory::Sk), // 3 chars
    (0x1FE0, 0x1FE7, GeneralCategory::Ll), // 8 chars
    (0x1FE8, 0x1FEC, GeneralCategory::Lu), // 5 chars
    (0x1FED, 0x1FEF, GeneralCategory::Sk), // 3 chars
    (0x1FF2, 0x1FF4, GeneralCategory::Ll), // 3 chars
    (0x1FF6, 0x1FF7, GeneralCategory::Ll), // 2 chars
    (0x1FF8, 0x1FFB, GeneralCategory::Lu), // 4 chars
    (0x1FFC, 0x1FFC, GeneralCategory::Lt),
    (0x1FFD, 0x1FFE, GeneralCategory::Sk), // 2 chars
    (0x2000, 0x200A, GeneralCategory::Zs), // 11 chars
    (0x200B, 0x200F, GeneralCategory::Cf), // 5 chars
    (0x2010, 0x2015, GeneralCategory::Pd), // 6 chars
    (0x2016, 0x2017, GeneralCategory::Po), // 2 chars
    (0x2018, 0x2018, GeneralCategory::Pi),
    (0x2019, 0x2019, GeneralCategory::Pf),
    (0x201A, 0x201A, GeneralCategory::Ps),
    (0x201B, 0x201C, GeneralCategory::Pi), // 2 chars
    (0x201D, 0x201D, GeneralCategory::Pf),
    (0x201E, 0x201E, GeneralCategory::Ps),
    (0x201F, 0x201F, GeneralCategory::Pi),
    (0x2020, 0x2027, GeneralCategory::Po), // 8 chars
    (0x2028, 0x2028, GeneralCategory::Zl),
    (0x2029, 0x2029, GeneralCategory::Zp),
    (0x202A, 0x202E, GeneralCategory::Cf), // 5 chars
    (0x202F, 0x202F, GeneralCategory::Zs),
    (0x2030, 0x2038, GeneralCategory::Po), // 9 chars
    (0x2039, 0x2039, GeneralCategory::Pi),
    (0x203A, 0x203A, GeneralCategory::Pf),
    (0x203B, 0x203E, GeneralCategory::Po), // 4 chars
    (0x203F, 0x2040, GeneralCategory::Pc), // 2 chars
    (0x2041, 0x2043, GeneralCategory::Po), // 3 chars
    (0x2044, 0x2044, GeneralCategory::Sm),
    (0x2045, 0x2045, GeneralCategory::Ps),
    (0x2046, 0x2046, GeneralCategory::Pe),
    (0x2047, 0x2051, GeneralCategory::Po), // 11 chars
    (0x2052, 0x2052, GeneralCategory::Sm),
    (0x2053, 0x2053, GeneralCategory::Po),
    (0x2054, 0x2054, GeneralCategory::Pc),
    (0x2055, 0x205E, GeneralCategory::Po), // 10 chars
    (0x205F, 0x205F, GeneralCategory::Zs),
    (0x2060, 0x2064, GeneralCategory::Cf), // 5 chars
    (0x2066, 0x206F, GeneralCategory::Cf), // 10 chars
    (0x2070, 0x2070, GeneralCategory::No),
    (0x2071, 0x2071, GeneralCategory::Lm),
    (0x2074, 0x2079, GeneralCategory::No), // 6 chars
    (0x207A, 0x207C, GeneralCategory::Sm), // 3 chars
    (0x207D, 0x207D, GeneralCategory::Ps),
    (0x207E, 0x207E, GeneralCategory::Pe),
    (0x207F, 0x207F, GeneralCategory::Lm),
    (0x2080, 0x2089, GeneralCategory::No), // 10 chars
    (0x208A, 0x208C, GeneralCategory::Sm), // 3 chars
    (0x208D, 0x208D, GeneralCategory::Ps),
    (0x208E, 0x208E, GeneralCategory::Pe),
    (0x2090, 0x209C, GeneralCategory::Lm), // 13 chars
    (0x20A0, 0x20C1, GeneralCategory::Sc), // 34 chars
    (0x20D0, 0x20DC, GeneralCategory::Mn), // 13 chars
    (0x20DD, 0x20E0, GeneralCategory::Me), // 4 chars
    (0x20E1, 0x20E1, GeneralCategory::Mn),
    (0x20E2, 0x20E4, GeneralCategory::Me), // 3 chars
    (0x20E5, 0x20F0, GeneralCategory::Mn), // 12 chars
    (0x2100, 0x2101, GeneralCategory::So), // 2 chars
    (0x2102, 0x2102, GeneralCategory::Lu),
    (0x2103, 0x2106, GeneralCategory::So), // 4 chars
    (0x2107, 0x2107, GeneralCategory::Lu),
    (0x2108, 0x2109, GeneralCategory::So), // 2 chars
    (0x210A, 0x210A, GeneralCategory::Ll),
    (0x210B, 0x210D, GeneralCategory::Lu), // 3 chars
    (0x210E, 0x210F, GeneralCategory::Ll), // 2 chars
    (0x2110, 0x2112, GeneralCategory::Lu), // 3 chars
    (0x2113, 0x2113, GeneralCategory::Ll),
    (0x2114, 0x2114, GeneralCategory::So),
    (0x2115, 0x2115, GeneralCategory::Lu),
    (0x2116, 0x2117, GeneralCategory::So), // 2 chars
    (0x2118, 0x2118, GeneralCategory::Sm),
    (0x2119, 0x211D, GeneralCategory::Lu), // 5 chars
    (0x211E, 0x2123, GeneralCategory::So), // 6 chars
    (0x2124, 0x2124, GeneralCategory::Lu),
    (0x2125, 0x2125, GeneralCategory::So),
    (0x2126, 0x2126, GeneralCategory::Lu),
    (0x2127, 0x2127, GeneralCategory::So),
    (0x2128, 0x2128, GeneralCategory::Lu),
    (0x2129, 0x2129, GeneralCategory::So),
    (0x212A, 0x212D, GeneralCategory::Lu), // 4 chars
    (0x212E, 0x212E, GeneralCategory::So),
    (0x212F, 0x212F, GeneralCategory::Ll),
    (0x2130, 0x2133, GeneralCategory::Lu), // 4 chars
    (0x2134, 0x2134, GeneralCategory::Ll),
    (0x2135, 0x2138, GeneralCategory::Lo), // 4 chars
    (0x2139, 0x2139, GeneralCategory::Ll),
    (0x213A, 0x213B, GeneralCategory::So), // 2 chars
    (0x213C, 0x213D, GeneralCategory::Ll), // 2 chars
    (0x213E, 0x213F, GeneralCategory::Lu), // 2 chars
    (0x2140, 0x2144, GeneralCategory::Sm), // 5 chars
    (0x2145, 0x2145, GeneralCategory::Lu),
    (0x2146, 0x2149, GeneralCategory::Ll), // 4 chars
    (0x214A, 0x214A, GeneralCategory::So),
    (0x214B, 0x214B, GeneralCategory::Sm),
    (0x214C, 0x214D, GeneralCategory::So), // 2 chars
    (0x214E, 0x214E, GeneralCategory::Ll),
    (0x214F, 0x214F, GeneralCategory::So),
    (0x2150, 0x215F, GeneralCategory::No), // 16 chars
    (0x2160, 0x2182, GeneralCategory::Nl), // 35 chars
    (0x2183, 0x2183, GeneralCategory::Lu),
    (0x2184, 0x2184, GeneralCategory::Ll),
    (0x2185, 0x2188, GeneralCategory::Nl), // 4 chars
    (0x2189, 0x2189, GeneralCategory::No),
    (0x218A, 0x218B, GeneralCategory::So), // 2 chars
    (0x2190, 0x2194, GeneralCategory::Sm), // 5 chars
    (0x2195, 0x2199, GeneralCategory::So), // 5 chars
    (0x219A, 0x219B, GeneralCategory::Sm), // 2 chars
    (0x219C, 0x219F, GeneralCategory::So), // 4 chars
    (0x21A0, 0x21A0, GeneralCategory::Sm),
    (0x21A1, 0x21A2, GeneralCategory::So), // 2 chars
    (0x21A3, 0x21A3, GeneralCategory::Sm),
    (0x21A4, 0x21A5, GeneralCategory::So), // 2 chars
    (0x21A6, 0x21A6, GeneralCategory::Sm),
    (0x21A7, 0x21AD, GeneralCategory::So), // 7 chars
    (0x21AE, 0x21AE, GeneralCategory::Sm),
    (0x21AF, 0x21CD, GeneralCategory::So), // 31 chars
    (0x21CE, 0x21CF, GeneralCategory::Sm), // 2 chars
    (0x21D0, 0x21D1, GeneralCategory::So), // 2 chars
    (0x21D2, 0x21D2, GeneralCategory::Sm),
    (0x21D3, 0x21D3, GeneralCategory::So),
    (0x21D4, 0x21D4, GeneralCategory::Sm),
    (0x21D5, 0x21F3, GeneralCategory::So), // 31 chars
    (0x21F4, 0x22FF, GeneralCategory::Sm), // 268 chars
    (0x2300, 0x2307, GeneralCategory::So), // 8 chars
    (0x2308, 0x2308, GeneralCategory::Ps),
    (0x2309, 0x2309, GeneralCategory::Pe),
    (0x230A, 0x230A, GeneralCategory::Ps),
    (0x230B, 0x230B, GeneralCategory::Pe),
    (0x230C, 0x231F, GeneralCategory::So), // 20 chars
    (0x2320, 0x2321, GeneralCategory::Sm), // 2 chars
    (0x2322, 0x2328, GeneralCategory::So), // 7 chars
    (0x2329, 0x2329, GeneralCategory::Ps),
    (0x232A, 0x232A, GeneralCategory::Pe),
    (0x232B, 0x237B, GeneralCategory::So), // 81 chars
    (0x237C, 0x237C, GeneralCategory::Sm),
    (0x237D, 0x239A, GeneralCategory::So), // 30 chars
    (0x239B, 0x23B3, GeneralCategory::Sm), // 25 chars
    (0x23B4, 0x23DB, GeneralCategory::So), // 40 chars
    (0x23DC, 0x23E1, GeneralCategory::Sm), // 6 chars
    (0x23E2, 0x2429, GeneralCategory::So), // 72 chars
    (0x2440, 0x244A, GeneralCategory::So), // 11 chars
    (0x2460, 0x249B, GeneralCategory::No), // 60 chars
    (0x249C, 0x24E9, GeneralCategory::So), // 78 chars
    (0x24EA, 0x24FF, GeneralCategory::No), // 22 chars
    (0x2500, 0x25B6, GeneralCategory::So), // 183 chars
    (0x25B7, 0x25B7, GeneralCategory::Sm),
    (0x25B8, 0x25C0, GeneralCategory::So), // 9 chars
    (0x25C1, 0x25C1, GeneralCategory::Sm),
    (0x25C2, 0x25F7, GeneralCategory::So), // 54 chars
    (0x25F8, 0x25FF, GeneralCategory::Sm), // 8 chars
    (0x2600, 0x266E, GeneralCategory::So), // 111 chars
    (0x266F, 0x266F, GeneralCategory::Sm),
    (0x2670, 0x2767, GeneralCategory::So), // 248 chars
    (0x2768, 0x2768, GeneralCategory::Ps),
    (0x2769, 0x2769, GeneralCategory::Pe),
    (0x276A, 0x276A, GeneralCategory::Ps),
    (0x276B, 0x276B, GeneralCategory::Pe),
    (0x276C, 0x276C, GeneralCategory::Ps),
    (0x276D, 0x276D, GeneralCategory::Pe),
    (0x276E, 0x276E, GeneralCategory::Ps),
    (0x276F, 0x276F, GeneralCategory::Pe),
    (0x2770, 0x2770, GeneralCategory::Ps),
    (0x2771, 0x2771, GeneralCategory::Pe),
    (0x2772, 0x2772, GeneralCategory::Ps),
    (0x2773, 0x2773, GeneralCategory::Pe),
    (0x2774, 0x2774, GeneralCategory::Ps),
    (0x2775, 0x2775, GeneralCategory::Pe),
    (0x2776, 0x2793, GeneralCategory::No), // 30 chars
    (0x2794, 0x27BF, GeneralCategory::So), // 44 chars
    (0x27C0, 0x27C4, GeneralCategory::Sm), // 5 chars
    (0x27C5, 0x27C5, GeneralCategory::Ps),
    (0x27C6, 0x27C6, GeneralCategory::Pe),
    (0x27C7, 0x27E5, GeneralCategory::Sm), // 31 chars
    (0x27E6, 0x27E6, GeneralCategory::Ps),
    (0x27E7, 0x27E7, GeneralCategory::Pe),
    (0x27E8, 0x27E8, GeneralCategory::Ps),
    (0x27E9, 0x27E9, GeneralCategory::Pe),
    (0x27EA, 0x27EA, GeneralCategory::Ps),
    (0x27EB, 0x27EB, GeneralCategory::Pe),
    (0x27EC, 0x27EC, GeneralCategory::Ps),
    (0x27ED, 0x27ED, GeneralCategory::Pe),
    (0x27EE, 0x27EE, GeneralCategory::Ps),
    (0x27EF, 0x27EF, GeneralCategory::Pe),
    (0x27F0, 0x27FF, GeneralCategory::Sm), // 16 chars
    (0x2800, 0x28FF, GeneralCategory::So), // 256 chars
    (0x2900, 0x2982, GeneralCategory::Sm), // 131 chars
    (0x2983, 0x2983, GeneralCategory::Ps),
    (0x2984, 0x2984, GeneralCategory::Pe),
    (0x2985, 0x2985, GeneralCategory::Ps),
    (0x2986, 0x2986, GeneralCategory::Pe),
    (0x2987, 0x2987, GeneralCategory::Ps),
    (0x2988, 0x2988, GeneralCategory::Pe),
    (0x2989, 0x2989, GeneralCategory::Ps),
    (0x298A, 0x298A, GeneralCategory::Pe),
    (0x298B, 0x298B, GeneralCategory::Ps),
    (0x298C, 0x298C, GeneralCategory::Pe),
    (0x298D, 0x298D, GeneralCategory::Ps),
    (0x298E, 0x298E, GeneralCategory::Pe),
    (0x298F, 0x298F, GeneralCategory::Ps),
    (0x2990, 0x2990, GeneralCategory::Pe),
    (0x2991, 0x2991, GeneralCategory::Ps),
    (0x2992, 0x2992, GeneralCategory::Pe),
    (0x2993, 0x2993, GeneralCategory::Ps),
    (0x2994, 0x2994, GeneralCategory::Pe),
    (0x2995, 0x2995, GeneralCategory::Ps),
    (0x2996, 0x2996, GeneralCategory::Pe),
    (0x2997, 0x2997, GeneralCategory::Ps),
    (0x2998, 0x2998, GeneralCategory::Pe),
    (0x2999, 0x29D7, GeneralCategory::Sm), // 63 chars
    (0x29D8, 0x29D8, GeneralCategory::Ps),
    (0x29D9, 0x29D9, GeneralCategory::Pe),
    (0x29DA, 0x29DA, GeneralCategory::Ps),
    (0x29DB, 0x29DB, GeneralCategory::Pe),
    (0x29DC, 0x29FB, GeneralCategory::Sm), // 32 chars
    (0x29FC, 0x29FC, GeneralCategory::Ps),
    (0x29FD, 0x29FD, GeneralCategory::Pe),
    (0x29FE, 0x2AFF, GeneralCategory::Sm), // 258 chars
    (0x2B00, 0x2B2F, GeneralCategory::So), // 48 chars
    (0x2B30, 0x2B44, GeneralCategory::Sm), // 21 chars
    (0x2B45, 0x2B46, GeneralCategory::So), // 2 chars
    (0x2B47, 0x2B4C, GeneralCategory::Sm), // 6 chars
    (0x2B4D, 0x2B73, GeneralCategory::So), // 39 chars
    (0x2B76, 0x2BFF, GeneralCategory::So), // 138 chars
    (0x2C00, 0x2C2F, GeneralCategory::Lu), // 48 chars
    (0x2C30, 0x2C5F, GeneralCategory::Ll), // 48 chars
    (0x2C60, 0x2C60, GeneralCategory::Lu),
    (0x2C61, 0x2C61, GeneralCategory::Ll),
    (0x2C62, 0x2C64, GeneralCategory::Lu), // 3 chars
    (0x2C65, 0x2C66, GeneralCategory::Ll), // 2 chars
    (0x2C67, 0x2C67, GeneralCategory::Lu),
    (0x2C68, 0x2C68, GeneralCategory::Ll),
    (0x2C69, 0x2C69, GeneralCategory::Lu),
    (0x2C6A, 0x2C6A, GeneralCategory::Ll),
    (0x2C6B, 0x2C6B, GeneralCategory::Lu),
    (0x2C6C, 0x2C6C, GeneralCategory::Ll),
    (0x2C6D, 0x2C70, GeneralCategory::Lu), // 4 chars
    (0x2C71, 0x2C71, GeneralCategory::Ll),
    (0x2C72, 0x2C72, GeneralCategory::Lu),
    (0x2C73, 0x2C74, GeneralCategory::Ll), // 2 chars
    (0x2C75, 0x2C75, GeneralCategory::Lu),
    (0x2C76, 0x2C7B, GeneralCategory::Ll), // 6 chars
    (0x2C7C, 0x2C7D, GeneralCategory::Lm), // 2 chars
    (0x2C7E, 0x2C80, GeneralCategory::Lu), // 3 chars
    (0x2C81, 0x2C81, GeneralCategory::Ll),
    (0x2C82, 0x2C82, GeneralCategory::Lu),
    (0x2C83, 0x2C83, GeneralCategory::Ll),
    (0x2C84, 0x2C84, GeneralCategory::Lu),
    (0x2C85, 0x2C85, GeneralCategory::Ll),
    (0x2C86, 0x2C86, GeneralCategory::Lu),
    (0x2C87, 0x2C87, GeneralCategory::Ll),
    (0x2C88, 0x2C88, GeneralCategory::Lu),
    (0x2C89, 0x2C89, GeneralCategory::Ll),
    (0x2C8A, 0x2C8A, GeneralCategory::Lu),
    (0x2C8B, 0x2C8B, GeneralCategory::Ll),
    (0x2C8C, 0x2C8C, GeneralCategory::Lu),
    (0x2C8D, 0x2C8D, GeneralCategory::Ll),
    (0x2C8E, 0x2C8E, GeneralCategory::Lu),
    (0x2C8F, 0x2C8F, GeneralCategory::Ll),
    (0x2C90, 0x2C90, GeneralCategory::Lu),
    (0x2C91, 0x2C91, GeneralCategory::Ll),
    (0x2C92, 0x2C92, GeneralCategory::Lu),
    (0x2C93, 0x2C93, GeneralCategory::Ll),
    (0x2C94, 0x2C94, GeneralCategory::Lu),
    (0x2C95, 0x2C95, GeneralCategory::Ll),
    (0x2C96, 0x2C96, GeneralCategory::Lu),
    (0x2C97, 0x2C97, GeneralCategory::Ll),
    (0x2C98, 0x2C98, GeneralCategory::Lu),
    (0x2C99, 0x2C99, GeneralCategory::Ll),
    (0x2C9A, 0x2C9A, GeneralCategory::Lu),
    (0x2C9B, 0x2C9B, GeneralCategory::Ll),
    (0x2C9C, 0x2C9C, GeneralCategory::Lu),
    (0x2C9D, 0x2C9D, GeneralCategory::Ll),
    (0x2C9E, 0x2C9E, GeneralCategory::Lu),
    (0x2C9F, 0x2C9F, GeneralCategory::Ll),
    (0x2CA0, 0x2CA0, GeneralCategory::Lu),
    (0x2CA1, 0x2CA1, GeneralCategory::Ll),
    (0x2CA2, 0x2CA2, GeneralCategory::Lu),
    (0x2CA3, 0x2CA3, GeneralCategory::Ll),
    (0x2CA4, 0x2CA4, GeneralCategory::Lu),
    (0x2CA5, 0x2CA5, GeneralCategory::Ll),
    (0x2CA6, 0x2CA6, GeneralCategory::Lu),
    (0x2CA7, 0x2CA7, GeneralCategory::Ll),
    (0x2CA8, 0x2CA8, GeneralCategory::Lu),
    (0x2CA9, 0x2CA9, GeneralCategory::Ll),
    (0x2CAA, 0x2CAA, GeneralCategory::Lu),
    (0x2CAB, 0x2CAB, GeneralCategory::Ll),
    (0x2CAC, 0x2CAC, GeneralCategory::Lu),
    (0x2CAD, 0x2CAD, GeneralCategory::Ll),
    (0x2CAE, 0x2CAE, GeneralCategory::Lu),
    (0x2CAF, 0x2CAF, GeneralCategory::Ll),
    (0x2CB0, 0x2CB0, GeneralCategory::Lu),
    (0x2CB1, 0x2CB1, GeneralCategory::Ll),
    (0x2CB2, 0x2CB2, GeneralCategory::Lu),
    (0x2CB3, 0x2CB3, GeneralCategory::Ll),
    (0x2CB4, 0x2CB4, GeneralCategory::Lu),
    (0x2CB5, 0x2CB5, GeneralCategory::Ll),
    (0x2CB6, 0x2CB6, GeneralCategory::Lu),
    (0x2CB7, 0x2CB7, GeneralCategory::Ll),
    (0x2CB8, 0x2CB8, GeneralCategory::Lu),
    (0x2CB9, 0x2CB9, GeneralCategory::Ll),
    (0x2CBA, 0x2CBA, GeneralCategory::Lu),
    (0x2CBB, 0x2CBB, GeneralCategory::Ll),
    (0x2CBC, 0x2CBC, GeneralCategory::Lu),
    (0x2CBD, 0x2CBD, GeneralCategory::Ll),
    (0x2CBE, 0x2CBE, GeneralCategory::Lu),
    (0x2CBF, 0x2CBF, GeneralCategory::Ll),
    (0x2CC0, 0x2CC0, GeneralCategory::Lu),
    (0x2CC1, 0x2CC1, GeneralCategory::Ll),
    (0x2CC2, 0x2CC2, GeneralCategory::Lu),
    (0x2CC3, 0x2CC3, GeneralCategory::Ll),
    (0x2CC4, 0x2CC4, GeneralCategory::Lu),
    (0x2CC5, 0x2CC5, GeneralCategory::Ll),
    (0x2CC6, 0x2CC6, GeneralCategory::Lu),
    (0x2CC7, 0x2CC7, GeneralCategory::Ll),
    (0x2CC8, 0x2CC8, GeneralCategory::Lu),
    (0x2CC9, 0x2CC9, GeneralCategory::Ll),
    (0x2CCA, 0x2CCA, GeneralCategory::Lu),
    (0x2CCB, 0x2CCB, GeneralCategory::Ll),
    (0x2CCC, 0x2CCC, GeneralCategory::Lu),
    (0x2CCD, 0x2CCD, GeneralCategory::Ll),
    (0x2CCE, 0x2CCE, GeneralCategory::Lu),
    (0x2CCF, 0x2CCF, GeneralCategory::Ll),
    (0x2CD0, 0x2CD0, GeneralCategory::Lu),
    (0x2CD1, 0x2CD1, GeneralCategory::Ll),
    (0x2CD2, 0x2CD2, GeneralCategory::Lu),
    (0x2CD3, 0x2CD3, GeneralCategory::Ll),
    (0x2CD4, 0x2CD4, GeneralCategory::Lu),
    (0x2CD5, 0x2CD5, GeneralCategory::Ll),
    (0x2CD6, 0x2CD6, GeneralCategory::Lu),
    (0x2CD7, 0x2CD7, GeneralCategory::Ll),
    (0x2CD8, 0x2CD8, GeneralCategory::Lu),
    (0x2CD9, 0x2CD9, GeneralCategory::Ll),
    (0x2CDA, 0x2CDA, GeneralCategory::Lu),
    (0x2CDB, 0x2CDB, GeneralCategory::Ll),
    (0x2CDC, 0x2CDC, GeneralCategory::Lu),
    (0x2CDD, 0x2CDD, GeneralCategory::Ll),
    (0x2CDE, 0x2CDE, GeneralCategory::Lu),
    (0x2CDF, 0x2CDF, GeneralCategory::Ll),
    (0x2CE0, 0x2CE0, GeneralCategory::Lu),
    (0x2CE1, 0x2CE1, GeneralCategory::Ll),
    (0x2CE2, 0x2CE2, GeneralCategory::Lu),
    (0x2CE3, 0x2CE4, GeneralCategory::Ll), // 2 chars
    (0x2CE5, 0x2CEA, GeneralCategory::So), // 6 chars
    (0x2CEB, 0x2CEB, GeneralCategory::Lu),
    (0x2CEC, 0x2CEC, GeneralCategory::Ll),
    (0x2CED, 0x2CED, GeneralCategory::Lu),
    (0x2CEE, 0x2CEE, GeneralCategory::Ll),
    (0x2CEF, 0x2CF1, GeneralCategory::Mn), // 3 chars
    (0x2CF2, 0x2CF2, GeneralCategory::Lu),
    (0x2CF3, 0x2CF3, GeneralCategory::Ll),
    (0x2CF9, 0x2CFC, GeneralCategory::Po), // 4 chars
    (0x2CFD, 0x2CFD, GeneralCategory::No),
    (0x2CFE, 0x2CFF, GeneralCategory::Po), // 2 chars
    (0x2D00, 0x2D25, GeneralCategory::Ll), // 38 chars
    (0x2D27, 0x2D27, GeneralCategory::Ll),
    (0x2D2D, 0x2D2D, GeneralCategory::Ll),
    (0x2D30, 0x2D67, GeneralCategory::Lo), // 56 chars
    (0x2D6F, 0x2D6F, GeneralCategory::Lm),
    (0x2D70, 0x2D70, GeneralCategory::Po),
    (0x2D7F, 0x2D7F, GeneralCategory::Mn),
    (0x2D80, 0x2D96, GeneralCategory::Lo), // 23 chars
    (0x2DA0, 0x2DA6, GeneralCategory::Lo), // 7 chars
    (0x2DA8, 0x2DAE, GeneralCategory::Lo), // 7 chars
    (0x2DB0, 0x2DB6, GeneralCategory::Lo), // 7 chars
    (0x2DB8, 0x2DBE, GeneralCategory::Lo), // 7 chars
    (0x2DC0, 0x2DC6, GeneralCategory::Lo), // 7 chars
    (0x2DC8, 0x2DCE, GeneralCategory::Lo), // 7 chars
    (0x2DD0, 0x2DD6, GeneralCategory::Lo), // 7 chars
    (0x2DD8, 0x2DDE, GeneralCategory::Lo), // 7 chars
    (0x2DE0, 0x2DFF, GeneralCategory::Mn), // 32 chars
    (0x2E00, 0x2E01, GeneralCategory::Po), // 2 chars
    (0x2E02, 0x2E02, GeneralCategory::Pi),
    (0x2E03, 0x2E03, GeneralCategory::Pf),
    (0x2E04, 0x2E04, GeneralCategory::Pi),
    (0x2E05, 0x2E05, GeneralCategory::Pf),
    (0x2E06, 0x2E08, GeneralCategory::Po), // 3 chars
    (0x2E09, 0x2E09, GeneralCategory::Pi),
    (0x2E0A, 0x2E0A, GeneralCategory::Pf),
    (0x2E0B, 0x2E0B, GeneralCategory::Po),
    (0x2E0C, 0x2E0C, GeneralCategory::Pi),
    (0x2E0D, 0x2E0D, GeneralCategory::Pf),
    (0x2E0E, 0x2E16, GeneralCategory::Po), // 9 chars
    (0x2E17, 0x2E17, GeneralCategory::Pd),
    (0x2E18, 0x2E19, GeneralCategory::Po), // 2 chars
    (0x2E1A, 0x2E1A, GeneralCategory::Pd),
    (0x2E1B, 0x2E1B, GeneralCategory::Po),
    (0x2E1C, 0x2E1C, GeneralCategory::Pi),
    (0x2E1D, 0x2E1D, GeneralCategory::Pf),
    (0x2E1E, 0x2E1F, GeneralCategory::Po), // 2 chars
    (0x2E20, 0x2E20, GeneralCategory::Pi),
    (0x2E21, 0x2E21, GeneralCategory::Pf),
    (0x2E22, 0x2E22, GeneralCategory::Ps),
    (0x2E23, 0x2E23, GeneralCategory::Pe),
    (0x2E24, 0x2E24, GeneralCategory::Ps),
    (0x2E25, 0x2E25, GeneralCategory::Pe),
    (0x2E26, 0x2E26, GeneralCategory::Ps),
    (0x2E27, 0x2E27, GeneralCategory::Pe),
    (0x2E28, 0x2E28, GeneralCategory::Ps),
    (0x2E29, 0x2E29, GeneralCategory::Pe),
    (0x2E2A, 0x2E2E, GeneralCategory::Po), // 5 chars
    (0x2E2F, 0x2E2F, GeneralCategory::Lm),
    (0x2E30, 0x2E39, GeneralCategory::Po), // 10 chars
    (0x2E3A, 0x2E3B, GeneralCategory::Pd), // 2 chars
    (0x2E3C, 0x2E3F, GeneralCategory::Po), // 4 chars
    (0x2E40, 0x2E40, GeneralCategory::Pd),
    (0x2E41, 0x2E41, GeneralCategory::Po),
    (0x2E42, 0x2E42, GeneralCategory::Ps),
    (0x2E43, 0x2E4F, GeneralCategory::Po), // 13 chars
    (0x2E50, 0x2E51, GeneralCategory::So), // 2 chars
    (0x2E52, 0x2E54, GeneralCategory::Po), // 3 chars
    (0x2E55, 0x2E55, GeneralCategory::Ps),
    (0x2E56, 0x2E56, GeneralCategory::Pe),
    (0x2E57, 0x2E57, GeneralCategory::Ps),
    (0x2E58, 0x2E58, GeneralCategory::Pe),
    (0x2E59, 0x2E59, GeneralCategory::Ps),
    (0x2E5A, 0x2E5A, GeneralCategory::Pe),
    (0x2E5B, 0x2E5B, GeneralCategory::Ps),
    (0x2E5C, 0x2E5C, GeneralCategory::Pe),
    (0x2E5D, 0x2E5D, GeneralCategory::Pd),
    (0x2E80, 0x2E99, GeneralCategory::So), // 26 chars
    (0x2E9B, 0x2EF3, GeneralCategory::So), // 89 chars
    (0x2F00, 0x2FD5, GeneralCategory::So), // 214 chars
    (0x2FF0, 0x2FFF, GeneralCategory::So), // 16 chars
    (0x3000, 0x3000, GeneralCategory::Zs),
    (0x3001, 0x3003, GeneralCategory::Po), // 3 chars
    (0x3004, 0x3004, GeneralCategory::So),
    (0x3005, 0x3005, GeneralCategory::Lm),
    (0x3006, 0x3006, GeneralCategory::Lo),
    (0x3007, 0x3007, GeneralCategory::Nl),
    (0x3008, 0x3008, GeneralCategory::Ps),
    (0x3009, 0x3009, GeneralCategory::Pe),
    (0x300A, 0x300A, GeneralCategory::Ps),
    (0x300B, 0x300B, GeneralCategory::Pe),
    (0x300C, 0x300C, GeneralCategory::Ps),
    (0x300D, 0x300D, GeneralCategory::Pe),
    (0x300E, 0x300E, GeneralCategory::Ps),
    (0x300F, 0x300F, GeneralCategory::Pe),
    (0x3010, 0x3010, GeneralCategory::Ps),
    (0x3011, 0x3011, GeneralCategory::Pe),
    (0x3012, 0x3013, GeneralCategory::So), // 2 chars
    (0x3014, 0x3014, GeneralCategory::Ps),
    (0x3015, 0x3015, GeneralCategory::Pe),
    (0x3016, 0x3016, GeneralCategory::Ps),
    (0x3017, 0x3017, GeneralCategory::Pe),
    (0x3018, 0x3018, GeneralCategory::Ps),
    (0x3019, 0x3019, GeneralCategory::Pe),
    (0x301A, 0x301A, GeneralCategory::Ps),
    (0x301B, 0x301B, GeneralCategory::Pe),
    (0x301C, 0x301C, GeneralCategory::Pd),
    (0x301D, 0x301D, GeneralCategory::Ps),
    (0x301E, 0x301F, GeneralCategory::Pe), // 2 chars
    (0x3020, 0x3020, GeneralCategory::So),
    (0x3021, 0x3029, GeneralCategory::Nl), // 9 chars
    (0x302A, 0x302D, GeneralCategory::Mn), // 4 chars
    (0x302E, 0x302F, GeneralCategory::Mc), // 2 chars
    (0x3030, 0x3030, GeneralCategory::Pd),
    (0x3031, 0x3035, GeneralCategory::Lm), // 5 chars
    (0x3036, 0x3037, GeneralCategory::So), // 2 chars
    (0x3038, 0x303A, GeneralCategory::Nl), // 3 chars
    (0x303B, 0x303B, GeneralCategory::Lm),
    (0x303C, 0x303C, GeneralCategory::Lo),
    (0x303D, 0x303D, GeneralCategory::Po),
    (0x303E, 0x303F, GeneralCategory::So), // 2 chars
    (0x3041, 0x3096, GeneralCategory::Lo), // 86 chars
    (0x3099, 0x309A, GeneralCategory::Mn), // 2 chars
    (0x309B, 0x309C, GeneralCategory::Sk), // 2 chars
    (0x309D, 0x309E, GeneralCategory::Lm), // 2 chars
    (0x309F, 0x309F, GeneralCategory::Lo),
    (0x30A0, 0x30A0, GeneralCategory::Pd),
    (0x30A1, 0x30FA, GeneralCategory::Lo), // 90 chars
    (0x30FB, 0x30FB, GeneralCategory::Po),
    (0x30FC, 0x30FE, GeneralCategory::Lm), // 3 chars
    (0x30FF, 0x30FF, GeneralCategory::Lo),
    (0x3105, 0x312F, GeneralCategory::Lo), // 43 chars
    (0x3131, 0x318E, GeneralCategory::Lo), // 94 chars
    (0x3190, 0x3191, GeneralCategory::So), // 2 chars
    (0x3192, 0x3195, GeneralCategory::No), // 4 chars
    (0x3196, 0x319F, GeneralCategory::So), // 10 chars
    (0x31A0, 0x31BF, GeneralCategory::Lo), // 32 chars
    (0x31C0, 0x31E5, GeneralCategory::So), // 38 chars
    (0x31EF, 0x31EF, GeneralCategory::So),
    (0x31F0, 0x31FF, GeneralCategory::Lo), // 16 chars
    (0x3200, 0x321E, GeneralCategory::So), // 31 chars
    (0x3220, 0x3229, GeneralCategory::No), // 10 chars
    (0x322A, 0x3247, GeneralCategory::So), // 30 chars
    (0x3248, 0x324F, GeneralCategory::No), // 8 chars
    (0x3250, 0x3250, GeneralCategory::So),
    (0x3251, 0x325F, GeneralCategory::No), // 15 chars
    (0x3260, 0x327F, GeneralCategory::So), // 32 chars
    (0x3280, 0x3289, GeneralCategory::No), // 10 chars
    (0x328A, 0x32B0, GeneralCategory::So), // 39 chars
    (0x32B1, 0x32BF, GeneralCategory::No), // 15 chars
    (0x32C0, 0x33FF, GeneralCategory::So), // 320 chars
    (0x3400, 0x4DBF, GeneralCategory::Lo), // 6592 chars
    (0x4DC0, 0x4DFF, GeneralCategory::So), // 64 chars
    (0x4E00, 0xA014, GeneralCategory::Lo), // 21013 chars
    (0xA015, 0xA015, GeneralCategory::Lm),
    (0xA016, 0xA48C, GeneralCategory::Lo), // 1143 chars
    (0xA490, 0xA4C6, GeneralCategory::So), // 55 chars
    (0xA4D0, 0xA4F7, GeneralCategory::Lo), // 40 chars
    (0xA4F8, 0xA4FD, GeneralCategory::Lm), // 6 chars
    (0xA4FE, 0xA4FF, GeneralCategory::Po), // 2 chars
    (0xA500, 0xA60B, GeneralCategory::Lo), // 268 chars
    (0xA60C, 0xA60C, GeneralCategory::Lm),
    (0xA60D, 0xA60F, GeneralCategory::Po), // 3 chars
    (0xA610, 0xA61F, GeneralCategory::Lo), // 16 chars
    (0xA620, 0xA629, GeneralCategory::Nd), // 10 chars
    (0xA62A, 0xA62B, GeneralCategory::Lo), // 2 chars
    (0xA640, 0xA640, GeneralCategory::Lu),
    (0xA641, 0xA641, GeneralCategory::Ll),
    (0xA642, 0xA642, GeneralCategory::Lu),
    (0xA643, 0xA643, GeneralCategory::Ll),
    (0xA644, 0xA644, GeneralCategory::Lu),
    (0xA645, 0xA645, GeneralCategory::Ll),
    (0xA646, 0xA646, GeneralCategory::Lu),
    (0xA647, 0xA647, GeneralCategory::Ll),
    (0xA648, 0xA648, GeneralCategory::Lu),
    (0xA649, 0xA649, GeneralCategory::Ll),
    (0xA64A, 0xA64A, GeneralCategory::Lu),
    (0xA64B, 0xA64B, GeneralCategory::Ll),
    (0xA64C, 0xA64C, GeneralCategory::Lu),
    (0xA64D, 0xA64D, GeneralCategory::Ll),
    (0xA64E, 0xA64E, GeneralCategory::Lu),
    (0xA64F, 0xA64F, GeneralCategory::Ll),
    (0xA650, 0xA650, GeneralCategory::Lu),
    (0xA651, 0xA651, GeneralCategory::Ll),
    (0xA652, 0xA652, GeneralCategory::Lu),
    (0xA653, 0xA653, GeneralCategory::Ll),
    (0xA654, 0xA654, GeneralCategory::Lu),
    (0xA655, 0xA655, GeneralCategory::Ll),
    (0xA656, 0xA656, GeneralCategory::Lu),
    (0xA657, 0xA657, GeneralCategory::Ll),
    (0xA658, 0xA658, GeneralCategory::Lu),
    (0xA659, 0xA659, GeneralCategory::Ll),
    (0xA65A, 0xA65A, GeneralCategory::Lu),
    (0xA65B, 0xA65B, GeneralCategory::Ll),
    (0xA65C, 0xA65C, GeneralCategory::Lu),
    (0xA65D, 0xA65D, GeneralCategory::Ll),
    (0xA65E, 0xA65E, GeneralCategory::Lu),
    (0xA65F, 0xA65F, GeneralCategory::Ll),
    (0xA660, 0xA660, GeneralCategory::Lu),
    (0xA661, 0xA661, GeneralCategory::Ll),
    (0xA662, 0xA662, GeneralCategory::Lu),
    (0xA663, 0xA663, GeneralCategory::Ll),
    (0xA664, 0xA664, GeneralCategory::Lu),
    (0xA665, 0xA665, GeneralCategory::Ll),
    (0xA666, 0xA666, GeneralCategory::Lu),
    (0xA667, 0xA667, GeneralCategory::Ll),
    (0xA668, 0xA668, GeneralCategory::Lu),
    (0xA669, 0xA669, GeneralCategory::Ll),
    (0xA66A, 0xA66A, GeneralCategory::Lu),
    (0xA66B, 0xA66B, GeneralCategory::Ll),
    (0xA66C, 0xA66C, GeneralCategory::Lu),
    (0xA66D, 0xA66D, GeneralCategory::Ll),
    (0xA66E, 0xA66E, GeneralCategory::Lo),
    (0xA66F, 0xA66F, GeneralCategory::Mn),
    (0xA670, 0xA672, GeneralCategory::Me), // 3 chars
    (0xA673, 0xA673, GeneralCategory::Po),
    (0xA674, 0xA67D, GeneralCategory::Mn), // 10 chars
    (0xA67E, 0xA67E, GeneralCategory::Po),
    (0xA67F, 0xA67F, GeneralCategory::Lm),
    (0xA680, 0xA680, GeneralCategory::Lu),
    (0xA681, 0xA681, GeneralCategory::Ll),
    (0xA682, 0xA682, GeneralCategory::Lu),
    (0xA683, 0xA683, GeneralCategory::Ll),
    (0xA684, 0xA684, GeneralCategory::Lu),
    (0xA685, 0xA685, GeneralCategory::Ll),
    (0xA686, 0xA686, GeneralCategory::Lu),
    (0xA687, 0xA687, GeneralCategory::Ll),
    (0xA688, 0xA688, GeneralCategory::Lu),
    (0xA689, 0xA689, GeneralCategory::Ll),
    (0xA68A, 0xA68A, GeneralCategory::Lu),
    (0xA68B, 0xA68B, GeneralCategory::Ll),
    (0xA68C, 0xA68C, GeneralCategory::Lu),
    (0xA68D, 0xA68D, GeneralCategory::Ll),
    (0xA68E, 0xA68E, GeneralCategory::Lu),
    (0xA68F, 0xA68F, GeneralCategory::Ll),
    (0xA690, 0xA690, GeneralCategory::Lu),
    (0xA691, 0xA691, GeneralCategory::Ll),
    (0xA692, 0xA692, GeneralCategory::Lu),
    (0xA693, 0xA693, GeneralCategory::Ll),
    (0xA694, 0xA694, GeneralCategory::Lu),
    (0xA695, 0xA695, GeneralCategory::Ll),
    (0xA696, 0xA696, GeneralCategory::Lu),
    (0xA697, 0xA697, GeneralCategory::Ll),
    (0xA698, 0xA698, GeneralCategory::Lu),
    (0xA699, 0xA699, GeneralCategory::Ll),
    (0xA69A, 0xA69A, GeneralCategory::Lu),
    (0xA69B, 0xA69B, GeneralCategory::Ll),
    (0xA69C, 0xA69D, GeneralCategory::Lm), // 2 chars
    (0xA69E, 0xA69F, GeneralCategory::Mn), // 2 chars
    (0xA6A0, 0xA6E5, GeneralCategory::Lo), // 70 chars
    (0xA6E6, 0xA6EF, GeneralCategory::Nl), // 10 chars
    (0xA6F0, 0xA6F1, GeneralCategory::Mn), // 2 chars
    (0xA6F2, 0xA6F7, GeneralCategory::Po), // 6 chars
    (0xA700, 0xA716, GeneralCategory::Sk), // 23 chars
    (0xA717, 0xA71F, GeneralCategory::Lm), // 9 chars
    (0xA720, 0xA721, GeneralCategory::Sk), // 2 chars
    (0xA722, 0xA722, GeneralCategory::Lu),
    (0xA723, 0xA723, GeneralCategory::Ll),
    (0xA724, 0xA724, GeneralCategory::Lu),
    (0xA725, 0xA725, GeneralCategory::Ll),
    (0xA726, 0xA726, GeneralCategory::Lu),
    (0xA727, 0xA727, GeneralCategory::Ll),
    (0xA728, 0xA728, GeneralCategory::Lu),
    (0xA729, 0xA729, GeneralCategory::Ll),
    (0xA72A, 0xA72A, GeneralCategory::Lu),
    (0xA72B, 0xA72B, GeneralCategory::Ll),
    (0xA72C, 0xA72C, GeneralCategory::Lu),
    (0xA72D, 0xA72D, GeneralCategory::Ll),
    (0xA72E, 0xA72E, GeneralCategory::Lu),
    (0xA72F, 0xA731, GeneralCategory::Ll), // 3 chars
    (0xA732, 0xA732, GeneralCategory::Lu),
    (0xA733, 0xA733, GeneralCategory::Ll),
    (0xA734, 0xA734, GeneralCategory::Lu),
    (0xA735, 0xA735, GeneralCategory::Ll),
    (0xA736, 0xA736, GeneralCategory::Lu),
    (0xA737, 0xA737, GeneralCategory::Ll),
    (0xA738, 0xA738, GeneralCategory::Lu),
    (0xA739, 0xA739, GeneralCategory::Ll),
    (0xA73A, 0xA73A, GeneralCategory::Lu),
    (0xA73B, 0xA73B, GeneralCategory::Ll),
    (0xA73C, 0xA73C, GeneralCategory::Lu),
    (0xA73D, 0xA73D, GeneralCategory::Ll),
    (0xA73E, 0xA73E, GeneralCategory::Lu),
    (0xA73F, 0xA73F, GeneralCategory::Ll),
    (0xA740, 0xA740, GeneralCategory::Lu),
    (0xA741, 0xA741, GeneralCategory::Ll),
    (0xA742, 0xA742, GeneralCategory::Lu),
    (0xA743, 0xA743, GeneralCategory::Ll),
    (0xA744, 0xA744, GeneralCategory::Lu),
    (0xA745, 0xA745, GeneralCategory::Ll),
    (0xA746, 0xA746, GeneralCategory::Lu),
    (0xA747, 0xA747, GeneralCategory::Ll),
    (0xA748, 0xA748, GeneralCategory::Lu),
    (0xA749, 0xA749, GeneralCategory::Ll),
    (0xA74A, 0xA74A, GeneralCategory::Lu),
    (0xA74B, 0xA74B, GeneralCategory::Ll),
    (0xA74C, 0xA74C, GeneralCategory::Lu),
    (0xA74D, 0xA74D, GeneralCategory::Ll),
    (0xA74E, 0xA74E, GeneralCategory::Lu),
    (0xA74F, 0xA74F, GeneralCategory::Ll),
    (0xA750, 0xA750, GeneralCategory::Lu),
    (0xA751, 0xA751, GeneralCategory::Ll),
    (0xA752, 0xA752, GeneralCategory::Lu),
    (0xA753, 0xA753, GeneralCategory::Ll),
    (0xA754, 0xA754, GeneralCategory::Lu),
    (0xA755, 0xA755, GeneralCategory::Ll),
    (0xA756, 0xA756, GeneralCategory::Lu),
    (0xA757, 0xA757, GeneralCategory::Ll),
    (0xA758, 0xA758, GeneralCategory::Lu),
    (0xA759, 0xA759, GeneralCategory::Ll),
    (0xA75A, 0xA75A, GeneralCategory::Lu),
    (0xA75B, 0xA75B, GeneralCategory::Ll),
    (0xA75C, 0xA75C, GeneralCategory::Lu),
    (0xA75D, 0xA75D, GeneralCategory::Ll),
    (0xA75E, 0xA75E, GeneralCategory::Lu),
    (0xA75F, 0xA75F, GeneralCategory::Ll),
    (0xA760, 0xA760, GeneralCategory::Lu),
    (0xA761, 0xA761, GeneralCategory::Ll),
    (0xA762, 0xA762, GeneralCategory::Lu),
    (0xA763, 0xA763, GeneralCategory::Ll),
    (0xA764, 0xA764, GeneralCategory::Lu),
    (0xA765, 0xA765, GeneralCategory::Ll),
    (0xA766, 0xA766, GeneralCategory::Lu),
    (0xA767, 0xA767, GeneralCategory::Ll),
    (0xA768, 0xA768, GeneralCategory::Lu),
    (0xA769, 0xA769, GeneralCategory::Ll),
    (0xA76A, 0xA76A, GeneralCategory::Lu),
    (0xA76B, 0xA76B, GeneralCategory::Ll),
    (0xA76C, 0xA76C, GeneralCategory::Lu),
    (0xA76D, 0xA76D, GeneralCategory::Ll),
    (0xA76E, 0xA76E, GeneralCategory::Lu),
    (0xA76F, 0xA76F, GeneralCategory::Ll),
    (0xA770, 0xA770, GeneralCategory::Lm),
    (0xA771, 0xA778, GeneralCategory::Ll), // 8 chars
    (0xA779, 0xA779, GeneralCategory::Lu),
    (0xA77A, 0xA77A, GeneralCategory::Ll),
    (0xA77B, 0xA77B, GeneralCategory::Lu),
    (0xA77C, 0xA77C, GeneralCategory::Ll),
    (0xA77D, 0xA77E, GeneralCategory::Lu), // 2 chars
    (0xA77F, 0xA77F, GeneralCategory::Ll),
    (0xA780, 0xA780, GeneralCategory::Lu),
    (0xA781, 0xA781, GeneralCategory::Ll),
    (0xA782, 0xA782, GeneralCategory::Lu),
    (0xA783, 0xA783, GeneralCategory::Ll),
    (0xA784, 0xA784, GeneralCategory::Lu),
    (0xA785, 0xA785, GeneralCategory::Ll),
    (0xA786, 0xA786, GeneralCategory::Lu),
    (0xA787, 0xA787, GeneralCategory::Ll),
    (0xA788, 0xA788, GeneralCategory::Lm),
    (0xA789, 0xA78A, GeneralCategory::Sk), // 2 chars
    (0xA78B, 0xA78B, GeneralCategory::Lu),
    (0xA78C, 0xA78C, GeneralCategory::Ll),
    (0xA78D, 0xA78D, GeneralCategory::Lu),
    (0xA78E, 0xA78E, GeneralCategory::Ll),
    (0xA78F, 0xA78F, GeneralCategory::Lo),
    (0xA790, 0xA790, GeneralCategory::Lu),
    (0xA791, 0xA791, GeneralCategory::Ll),
    (0xA792, 0xA792, GeneralCategory::Lu),
    (0xA793, 0xA795, GeneralCategory::Ll), // 3 chars
    (0xA796, 0xA796, GeneralCategory::Lu),
    (0xA797, 0xA797, GeneralCategory::Ll),
    (0xA798, 0xA798, GeneralCategory::Lu),
    (0xA799, 0xA799, GeneralCategory::Ll),
    (0xA79A, 0xA79A, GeneralCategory::Lu),
    (0xA79B, 0xA79B, GeneralCategory::Ll),
    (0xA79C, 0xA79C, GeneralCategory::Lu),
    (0xA79D, 0xA79D, GeneralCategory::Ll),
    (0xA79E, 0xA79E, GeneralCategory::Lu),
    (0xA79F, 0xA79F, GeneralCategory::Ll),
    (0xA7A0, 0xA7A0, GeneralCategory::Lu),
    (0xA7A1, 0xA7A1, GeneralCategory::Ll),
    (0xA7A2, 0xA7A2, GeneralCategory::Lu),
    (0xA7A3, 0xA7A3, GeneralCategory::Ll),
    (0xA7A4, 0xA7A4, GeneralCategory::Lu),
    (0xA7A5, 0xA7A5, GeneralCategory::Ll),
    (0xA7A6, 0xA7A6, GeneralCategory::Lu),
    (0xA7A7, 0xA7A7, GeneralCategory::Ll),
    (0xA7A8, 0xA7A8, GeneralCategory::Lu),
    (0xA7A9, 0xA7A9, GeneralCategory::Ll),
    (0xA7AA, 0xA7AE, GeneralCategory::Lu), // 5 chars
    (0xA7AF, 0xA7AF, GeneralCategory::Ll),
    (0xA7B0, 0xA7B4, GeneralCategory::Lu), // 5 chars
    (0xA7B5, 0xA7B5, GeneralCategory::Ll),
    (0xA7B6, 0xA7B6, GeneralCategory::Lu),
    (0xA7B7, 0xA7B7, GeneralCategory::Ll),
    (0xA7B8, 0xA7B8, GeneralCategory::Lu),
    (0xA7B9, 0xA7B9, GeneralCategory::Ll),
    (0xA7BA, 0xA7BA, GeneralCategory::Lu),
    (0xA7BB, 0xA7BB, GeneralCategory::Ll),
    (0xA7BC, 0xA7BC, GeneralCategory::Lu),
    (0xA7BD, 0xA7BD, GeneralCategory::Ll),
    (0xA7BE, 0xA7BE, GeneralCategory::Lu),
    (0xA7BF, 0xA7BF, GeneralCategory::Ll),
    (0xA7C0, 0xA7C0, GeneralCategory::Lu),
    (0xA7C1, 0xA7C1, GeneralCategory::Ll),
    (0xA7C2, 0xA7C2, GeneralCategory::Lu),
    (0xA7C3, 0xA7C3, GeneralCategory::Ll),
    (0xA7C4, 0xA7C7, GeneralCategory::Lu), // 4 chars
    (0xA7C8, 0xA7C8, GeneralCategory::Ll),
    (0xA7C9, 0xA7C9, GeneralCategory::Lu),
    (0xA7CA, 0xA7CA, GeneralCategory::Ll),
    (0xA7CB, 0xA7CC, GeneralCategory::Lu), // 2 chars
    (0xA7CD, 0xA7CD, GeneralCategory::Ll),
    (0xA7CE, 0xA7CE, GeneralCategory::Lu),
    (0xA7CF, 0xA7CF, GeneralCategory::Ll),
    (0xA7D0, 0xA7D0, GeneralCategory::Lu),
    (0xA7D1, 0xA7D1, GeneralCategory::Ll),
    (0xA7D2, 0xA7D2, GeneralCategory::Lu),
    (0xA7D3, 0xA7D3, GeneralCategory::Ll),
    (0xA7D4, 0xA7D4, GeneralCategory::Lu),
    (0xA7D5, 0xA7D5, GeneralCategory::Ll),
    (0xA7D6, 0xA7D6, GeneralCategory::Lu),
    (0xA7D7, 0xA7D7, GeneralCategory::Ll),
    (0xA7D8, 0xA7D8, GeneralCategory::Lu),
    (0xA7D9, 0xA7D9, GeneralCategory::Ll),
    (0xA7DA, 0xA7DA, GeneralCategory::Lu),
    (0xA7DB, 0xA7DB, GeneralCategory::Ll),
    (0xA7DC, 0xA7DC, GeneralCategory::Lu),
    (0xA7F1, 0xA7F4, GeneralCategory::Lm), // 4 chars
    (0xA7F5, 0xA7F5, GeneralCategory::Lu),
    (0xA7F6, 0xA7F6, GeneralCategory::Ll),
    (0xA7F7, 0xA7F7, GeneralCategory::Lo),
    (0xA7F8, 0xA7F9, GeneralCategory::Lm), // 2 chars
    (0xA7FA, 0xA7FA, GeneralCategory::Ll),
    (0xA7FB, 0xA801, GeneralCategory::Lo), // 7 chars
    (0xA802, 0xA802, GeneralCategory::Mn),
    (0xA803, 0xA805, GeneralCategory::Lo), // 3 chars
    (0xA806, 0xA806, GeneralCategory::Mn),
    (0xA807, 0xA80A, GeneralCategory::Lo), // 4 chars
    (0xA80B, 0xA80B, GeneralCategory::Mn),
    (0xA80C, 0xA822, GeneralCategory::Lo), // 23 chars
    (0xA823, 0xA824, GeneralCategory::Mc), // 2 chars
    (0xA825, 0xA826, GeneralCategory::Mn), // 2 chars
    (0xA827, 0xA827, GeneralCategory::Mc),
    (0xA828, 0xA82B, GeneralCategory::So), // 4 chars
    (0xA82C, 0xA82C, GeneralCategory::Mn),
    (0xA830, 0xA835, GeneralCategory::No), // 6 chars
    (0xA836, 0xA837, GeneralCategory::So), // 2 chars
    (0xA838, 0xA838, GeneralCategory::Sc),
    (0xA839, 0xA839, GeneralCategory::So),
    (0xA840, 0xA873, GeneralCategory::Lo), // 52 chars
    (0xA874, 0xA877, GeneralCategory::Po), // 4 chars
    (0xA880, 0xA881, GeneralCategory::Mc), // 2 chars
    (0xA882, 0xA8B3, GeneralCategory::Lo), // 50 chars
    (0xA8B4, 0xA8C3, GeneralCategory::Mc), // 16 chars
    (0xA8C4, 0xA8C5, GeneralCategory::Mn), // 2 chars
    (0xA8CE, 0xA8CF, GeneralCategory::Po), // 2 chars
    (0xA8D0, 0xA8D9, GeneralCategory::Nd), // 10 chars
    (0xA8E0, 0xA8F1, GeneralCategory::Mn), // 18 chars
    (0xA8F2, 0xA8F7, GeneralCategory::Lo), // 6 chars
    (0xA8F8, 0xA8FA, GeneralCategory::Po), // 3 chars
    (0xA8FB, 0xA8FB, GeneralCategory::Lo),
    (0xA8FC, 0xA8FC, GeneralCategory::Po),
    (0xA8FD, 0xA8FE, GeneralCategory::Lo), // 2 chars
    (0xA8FF, 0xA8FF, GeneralCategory::Mn),
    (0xA900, 0xA909, GeneralCategory::Nd), // 10 chars
    (0xA90A, 0xA925, GeneralCategory::Lo), // 28 chars
    (0xA926, 0xA92D, GeneralCategory::Mn), // 8 chars
    (0xA92E, 0xA92F, GeneralCategory::Po), // 2 chars
    (0xA930, 0xA946, GeneralCategory::Lo), // 23 chars
    (0xA947, 0xA951, GeneralCategory::Mn), // 11 chars
    (0xA952, 0xA953, GeneralCategory::Mc), // 2 chars
    (0xA95F, 0xA95F, GeneralCategory::Po),
    (0xA960, 0xA97C, GeneralCategory::Lo), // 29 chars
    (0xA980, 0xA982, GeneralCategory::Mn), // 3 chars
    (0xA983, 0xA983, GeneralCategory::Mc),
    (0xA984, 0xA9B2, GeneralCategory::Lo), // 47 chars
    (0xA9B3, 0xA9B3, GeneralCategory::Mn),
    (0xA9B4, 0xA9B5, GeneralCategory::Mc), // 2 chars
    (0xA9B6, 0xA9B9, GeneralCategory::Mn), // 4 chars
    (0xA9BA, 0xA9BB, GeneralCategory::Mc), // 2 chars
    (0xA9BC, 0xA9BD, GeneralCategory::Mn), // 2 chars
    (0xA9BE, 0xA9C0, GeneralCategory::Mc), // 3 chars
    (0xA9C1, 0xA9CD, GeneralCategory::Po), // 13 chars
    (0xA9CF, 0xA9CF, GeneralCategory::Lm),
    (0xA9D0, 0xA9D9, GeneralCategory::Nd), // 10 chars
    (0xA9DE, 0xA9DF, GeneralCategory::Po), // 2 chars
    (0xA9E0, 0xA9E4, GeneralCategory::Lo), // 5 chars
    (0xA9E5, 0xA9E5, GeneralCategory::Mn),
    (0xA9E6, 0xA9E6, GeneralCategory::Lm),
    (0xA9E7, 0xA9EF, GeneralCategory::Lo), // 9 chars
    (0xA9F0, 0xA9F9, GeneralCategory::Nd), // 10 chars
    (0xA9FA, 0xA9FE, GeneralCategory::Lo), // 5 chars
    (0xAA00, 0xAA28, GeneralCategory::Lo), // 41 chars
    (0xAA29, 0xAA2E, GeneralCategory::Mn), // 6 chars
    (0xAA2F, 0xAA30, GeneralCategory::Mc), // 2 chars
    (0xAA31, 0xAA32, GeneralCategory::Mn), // 2 chars
    (0xAA33, 0xAA34, GeneralCategory::Mc), // 2 chars
    (0xAA35, 0xAA36, GeneralCategory::Mn), // 2 chars
    (0xAA40, 0xAA42, GeneralCategory::Lo), // 3 chars
    (0xAA43, 0xAA43, GeneralCategory::Mn),
    (0xAA44, 0xAA4B, GeneralCategory::Lo), // 8 chars
    (0xAA4C, 0xAA4C, GeneralCategory::Mn),
    (0xAA4D, 0xAA4D, GeneralCategory::Mc),
    (0xAA50, 0xAA59, GeneralCategory::Nd), // 10 chars
    (0xAA5C, 0xAA5F, GeneralCategory::Po), // 4 chars
    (0xAA60, 0xAA6F, GeneralCategory::Lo), // 16 chars
    (0xAA70, 0xAA70, GeneralCategory::Lm),
    (0xAA71, 0xAA76, GeneralCategory::Lo), // 6 chars
    (0xAA77, 0xAA79, GeneralCategory::So), // 3 chars
    (0xAA7A, 0xAA7A, GeneralCategory::Lo),
    (0xAA7B, 0xAA7B, GeneralCategory::Mc),
    (0xAA7C, 0xAA7C, GeneralCategory::Mn),
    (0xAA7D, 0xAA7D, GeneralCategory::Mc),
    (0xAA7E, 0xAAAF, GeneralCategory::Lo), // 50 chars
    (0xAAB0, 0xAAB0, GeneralCategory::Mn),
    (0xAAB1, 0xAAB1, GeneralCategory::Lo),
    (0xAAB2, 0xAAB4, GeneralCategory::Mn), // 3 chars
    (0xAAB5, 0xAAB6, GeneralCategory::Lo), // 2 chars
    (0xAAB7, 0xAAB8, GeneralCategory::Mn), // 2 chars
    (0xAAB9, 0xAABD, GeneralCategory::Lo), // 5 chars
    (0xAABE, 0xAABF, GeneralCategory::Mn), // 2 chars
    (0xAAC0, 0xAAC0, GeneralCategory::Lo),
    (0xAAC1, 0xAAC1, GeneralCategory::Mn),
    (0xAAC2, 0xAAC2, GeneralCategory::Lo),
    (0xAADB, 0xAADC, GeneralCategory::Lo), // 2 chars
    (0xAADD, 0xAADD, GeneralCategory::Lm),
    (0xAADE, 0xAADF, GeneralCategory::Po), // 2 chars
    (0xAAE0, 0xAAEA, GeneralCategory::Lo), // 11 chars
    (0xAAEB, 0xAAEB, GeneralCategory::Mc),
    (0xAAEC, 0xAAED, GeneralCategory::Mn), // 2 chars
    (0xAAEE, 0xAAEF, GeneralCategory::Mc), // 2 chars
    (0xAAF0, 0xAAF1, GeneralCategory::Po), // 2 chars
    (0xAAF2, 0xAAF2, GeneralCategory::Lo),
    (0xAAF3, 0xAAF4, GeneralCategory::Lm), // 2 chars
    (0xAAF5, 0xAAF5, GeneralCategory::Mc),
    (0xAAF6, 0xAAF6, GeneralCategory::Mn),
    (0xAB01, 0xAB06, GeneralCategory::Lo), // 6 chars
    (0xAB09, 0xAB0E, GeneralCategory::Lo), // 6 chars
    (0xAB11, 0xAB16, GeneralCategory::Lo), // 6 chars
    (0xAB20, 0xAB26, GeneralCategory::Lo), // 7 chars
    (0xAB28, 0xAB2E, GeneralCategory::Lo), // 7 chars
    (0xAB30, 0xAB5A, GeneralCategory::Ll), // 43 chars
    (0xAB5B, 0xAB5B, GeneralCategory::Sk),
    (0xAB5C, 0xAB5F, GeneralCategory::Lm), // 4 chars
    (0xAB60, 0xAB68, GeneralCategory::Ll), // 9 chars
    (0xAB69, 0xAB69, GeneralCategory::Lm),
    (0xAB6A, 0xAB6B, GeneralCategory::Sk), // 2 chars
    (0xAB70, 0xABBF, GeneralCategory::Ll), // 80 chars
    (0xABC0, 0xABE2, GeneralCategory::Lo), // 35 chars
    (0xABE3, 0xABE4, GeneralCategory::Mc), // 2 chars
    (0xABE5, 0xABE5, GeneralCategory::Mn),
    (0xABE6, 0xABE7, GeneralCategory::Mc), // 2 chars
    (0xABE8, 0xABE8, GeneralCategory::Mn),
    (0xABE9, 0xABEA, GeneralCategory::Mc), // 2 chars
    (0xABEB, 0xABEB, GeneralCategory::Po),
    (0xABEC, 0xABEC, GeneralCategory::Mc),
    (0xABED, 0xABED, GeneralCategory::Mn),
    (0xABF0, 0xABF9, GeneralCategory::Nd), // 10 chars
    (0xAC00, 0xD7A3, GeneralCategory::Lo), // 11172 chars
    (0xD7B0, 0xD7C6, GeneralCategory::Lo), // 23 chars
    (0xD7CB, 0xD7FB, GeneralCategory::Lo), // 49 chars
    (0xD800, 0xDFFF, GeneralCategory::Cs), // 2048 chars
    (0xE000, 0xF8FF, GeneralCategory::Co), // 6400 chars
    (0xF900, 0xFA6D, GeneralCategory::Lo), // 366 chars
    (0xFA70, 0xFAD9, GeneralCategory::Lo), // 106 chars
    (0xFB00, 0xFB06, GeneralCategory::Ll), // 7 chars
    (0xFB13, 0xFB17, GeneralCategory::Ll), // 5 chars
    (0xFB1D, 0xFB1D, GeneralCategory::Lo),
    (0xFB1E, 0xFB1E, GeneralCategory::Mn),
    (0xFB1F, 0xFB28, GeneralCategory::Lo), // 10 chars
    (0xFB29, 0xFB29, GeneralCategory::Sm),
    (0xFB2A, 0xFB36, GeneralCategory::Lo), // 13 chars
    (0xFB38, 0xFB3C, GeneralCategory::Lo), // 5 chars
    (0xFB3E, 0xFB3E, GeneralCategory::Lo),
    (0xFB40, 0xFB41, GeneralCategory::Lo), // 2 chars
    (0xFB43, 0xFB44, GeneralCategory::Lo), // 2 chars
    (0xFB46, 0xFBB1, GeneralCategory::Lo), // 108 chars
    (0xFBB2, 0xFBC2, GeneralCategory::Sk), // 17 chars
    (0xFBC3, 0xFBD2, GeneralCategory::So), // 16 chars
    (0xFBD3, 0xFD3D, GeneralCategory::Lo), // 363 chars
    (0xFD3E, 0xFD3E, GeneralCategory::Pe),
    (0xFD3F, 0xFD3F, GeneralCategory::Ps),
    (0xFD40, 0xFD4F, GeneralCategory::So), // 16 chars
    (0xFD50, 0xFD8F, GeneralCategory::Lo), // 64 chars
    (0xFD90, 0xFD91, GeneralCategory::So), // 2 chars
    (0xFD92, 0xFDC7, GeneralCategory::Lo), // 54 chars
    (0xFDC8, 0xFDCF, GeneralCategory::So), // 8 chars
    (0xFDF0, 0xFDFB, GeneralCategory::Lo), // 12 chars
    (0xFDFC, 0xFDFC, GeneralCategory::Sc),
    (0xFDFD, 0xFDFF, GeneralCategory::So), // 3 chars
    (0xFE00, 0xFE0F, GeneralCategory::Mn), // 16 chars
    (0xFE10, 0xFE16, GeneralCategory::Po), // 7 chars
    (0xFE17, 0xFE17, GeneralCategory::Ps),
    (0xFE18, 0xFE18, GeneralCategory::Pe),
    (0xFE19, 0xFE19, GeneralCategory::Po),
    (0xFE20, 0xFE2F, GeneralCategory::Mn), // 16 chars
    (0xFE30, 0xFE30, GeneralCategory::Po),
    (0xFE31, 0xFE32, GeneralCategory::Pd), // 2 chars
    (0xFE33, 0xFE34, GeneralCategory::Pc), // 2 chars
    (0xFE35, 0xFE35, GeneralCategory::Ps),
    (0xFE36, 0xFE36, GeneralCategory::Pe),
    (0xFE37, 0xFE37, GeneralCategory::Ps),
    (0xFE38, 0xFE38, GeneralCategory::Pe),
    (0xFE39, 0xFE39, GeneralCategory::Ps),
    (0xFE3A, 0xFE3A, GeneralCategory::Pe),
    (0xFE3B, 0xFE3B, GeneralCategory::Ps),
    (0xFE3C, 0xFE3C, GeneralCategory::Pe),
    (0xFE3D, 0xFE3D, GeneralCategory::Ps),
    (0xFE3E, 0xFE3E, GeneralCategory::Pe),
    (0xFE3F, 0xFE3F, GeneralCategory::Ps),
    (0xFE40, 0xFE40, GeneralCategory::Pe),
    (0xFE41, 0xFE41, GeneralCategory::Ps),
    (0xFE42, 0xFE42, GeneralCategory::Pe),
    (0xFE43, 0xFE43, GeneralCategory::Ps),
    (0xFE44, 0xFE44, GeneralCategory::Pe),
    (0xFE45, 0xFE46, GeneralCategory::Po), // 2 chars
    (0xFE47, 0xFE47, GeneralCategory::Ps),
    (0xFE48, 0xFE48, GeneralCategory::Pe),
    (0xFE49, 0xFE4C, GeneralCategory::Po), // 4 chars
    (0xFE4D, 0xFE4F, GeneralCategory::Pc), // 3 chars
    (0xFE50, 0xFE52, GeneralCategory::Po), // 3 chars
    (0xFE54, 0xFE57, GeneralCategory::Po), // 4 chars
    (0xFE58, 0xFE58, GeneralCategory::Pd),
    (0xFE59, 0xFE59, GeneralCategory::Ps),
    (0xFE5A, 0xFE5A, GeneralCategory::Pe),
    (0xFE5B, 0xFE5B, GeneralCategory::Ps),
    (0xFE5C, 0xFE5C, GeneralCategory::Pe),
    (0xFE5D, 0xFE5D, GeneralCategory::Ps),
    (0xFE5E, 0xFE5E, GeneralCategory::Pe),
    (0xFE5F, 0xFE61, GeneralCategory::Po), // 3 chars
    (0xFE62, 0xFE62, GeneralCategory::Sm),
    (0xFE63, 0xFE63, GeneralCategory::Pd),
    (0xFE64, 0xFE66, GeneralCategory::Sm), // 3 chars
    (0xFE68, 0xFE68, GeneralCategory::Po),
    (0xFE69, 0xFE69, GeneralCategory::Sc),
    (0xFE6A, 0xFE6B, GeneralCategory::Po), // 2 chars
    (0xFE70, 0xFE74, GeneralCategory::Lo), // 5 chars
    (0xFE76, 0xFEFC, GeneralCategory::Lo), // 135 chars
    (0xFEFF, 0xFEFF, GeneralCategory::Cf),
    (0xFF01, 0xFF03, GeneralCategory::Po), // 3 chars
    (0xFF04, 0xFF04, GeneralCategory::Sc),
    (0xFF05, 0xFF07, GeneralCategory::Po), // 3 chars
    (0xFF08, 0xFF08, GeneralCategory::Ps),
    (0xFF09, 0xFF09, GeneralCategory::Pe),
    (0xFF0A, 0xFF0A, GeneralCategory::Po),
    (0xFF0B, 0xFF0B, GeneralCategory::Sm),
    (0xFF0C, 0xFF0C, GeneralCategory::Po),
    (0xFF0D, 0xFF0D, GeneralCategory::Pd),
    (0xFF0E, 0xFF0F, GeneralCategory::Po), // 2 chars
    (0xFF10, 0xFF19, GeneralCategory::Nd), // 10 chars
    (0xFF1A, 0xFF1B, GeneralCategory::Po), // 2 chars
    (0xFF1C, 0xFF1E, GeneralCategory::Sm), // 3 chars
    (0xFF1F, 0xFF20, GeneralCategory::Po), // 2 chars
    (0xFF21, 0xFF3A, GeneralCategory::Lu), // 26 chars
    (0xFF3B, 0xFF3B, GeneralCategory::Ps),
    (0xFF3C, 0xFF3C, GeneralCategory::Po),
    (0xFF3D, 0xFF3D, GeneralCategory::Pe),
    (0xFF3E, 0xFF3E, GeneralCategory::Sk),
    (0xFF3F, 0xFF3F, GeneralCategory::Pc),
    (0xFF40, 0xFF40, GeneralCategory::Sk),
    (0xFF41, 0xFF5A, GeneralCategory::Ll), // 26 chars
    (0xFF5B, 0xFF5B, GeneralCategory::Ps),
    (0xFF5C, 0xFF5C, GeneralCategory::Sm),
    (0xFF5D, 0xFF5D, GeneralCategory::Pe),
    (0xFF5E, 0xFF5E, GeneralCategory::Sm),
    (0xFF5F, 0xFF5F, GeneralCategory::Ps),
    (0xFF60, 0xFF60, GeneralCategory::Pe),
    (0xFF61, 0xFF61, GeneralCategory::Po),
    (0xFF62, 0xFF62, GeneralCategory::Ps),
    (0xFF63, 0xFF63, GeneralCategory::Pe),
    (0xFF64, 0xFF65, GeneralCategory::Po), // 2 chars
    (0xFF66, 0xFF6F, GeneralCategory::Lo), // 10 chars
    (0xFF70, 0xFF70, GeneralCategory::Lm),
    (0xFF71, 0xFF9D, GeneralCategory::Lo), // 45 chars
    (0xFF9E, 0xFF9F, GeneralCategory::Lm), // 2 chars
    (0xFFA0, 0xFFBE, GeneralCategory::Lo), // 31 chars
    (0xFFC2, 0xFFC7, GeneralCategory::Lo), // 6 chars
    (0xFFCA, 0xFFCF, GeneralCategory::Lo), // 6 chars
    (0xFFD2, 0xFFD7, GeneralCategory::Lo), // 6 chars
    (0xFFDA, 0xFFDC, GeneralCategory::Lo), // 3 chars
    (0xFFE0, 0xFFE1, GeneralCategory::Sc), // 2 chars
    (0xFFE2, 0xFFE2, GeneralCategory::Sm),
    (0xFFE3, 0xFFE3, GeneralCategory::Sk),
    (0xFFE4, 0xFFE4, GeneralCategory::So),
    (0xFFE5, 0xFFE6, GeneralCategory::Sc), // 2 chars
    (0xFFE8, 0xFFE8, GeneralCategory::So),
    (0xFFE9, 0xFFEC, GeneralCategory::Sm),   // 4 chars
    (0xFFED, 0xFFEE, GeneralCategory::So),   // 2 chars
    (0xFFF9, 0xFFFB, GeneralCategory::Cf),   // 3 chars
    (0xFFFC, 0xFFFD, GeneralCategory::So),   // 2 chars
    (0x10000, 0x1000B, GeneralCategory::Lo), // 12 chars
    (0x1000D, 0x10026, GeneralCategory::Lo), // 26 chars
    (0x10028, 0x1003A, GeneralCategory::Lo), // 19 chars
    (0x1003C, 0x1003D, GeneralCategory::Lo), // 2 chars
    (0x1003F, 0x1004D, GeneralCategory::Lo), // 15 chars
    (0x10050, 0x1005D, GeneralCategory::Lo), // 14 chars
    (0x10080, 0x100FA, GeneralCategory::Lo), // 123 chars
    (0x10100, 0x10102, GeneralCategory::Po), // 3 chars
    (0x10107, 0x10133, GeneralCategory::No), // 45 chars
    (0x10137, 0x1013F, GeneralCategory::So), // 9 chars
    (0x10140, 0x10174, GeneralCategory::Nl), // 53 chars
    (0x10175, 0x10178, GeneralCategory::No), // 4 chars
    (0x10179, 0x10189, GeneralCategory::So), // 17 chars
    (0x1018A, 0x1018B, GeneralCategory::No), // 2 chars
    (0x1018C, 0x1018E, GeneralCategory::So), // 3 chars
    (0x10190, 0x1019C, GeneralCategory::So), // 13 chars
    (0x101A0, 0x101A0, GeneralCategory::So),
    (0x101D0, 0x101FC, GeneralCategory::So), // 45 chars
    (0x101FD, 0x101FD, GeneralCategory::Mn),
    (0x10280, 0x1029C, GeneralCategory::Lo), // 29 chars
    (0x102A0, 0x102D0, GeneralCategory::Lo), // 49 chars
    (0x102E0, 0x102E0, GeneralCategory::Mn),
    (0x102E1, 0x102FB, GeneralCategory::No), // 27 chars
    (0x10300, 0x1031F, GeneralCategory::Lo), // 32 chars
    (0x10320, 0x10323, GeneralCategory::No), // 4 chars
    (0x1032D, 0x10340, GeneralCategory::Lo), // 20 chars
    (0x10341, 0x10341, GeneralCategory::Nl),
    (0x10342, 0x10349, GeneralCategory::Lo), // 8 chars
    (0x1034A, 0x1034A, GeneralCategory::Nl),
    (0x10350, 0x10375, GeneralCategory::Lo), // 38 chars
    (0x10376, 0x1037A, GeneralCategory::Mn), // 5 chars
    (0x10380, 0x1039D, GeneralCategory::Lo), // 30 chars
    (0x1039F, 0x1039F, GeneralCategory::Po),
    (0x103A0, 0x103C3, GeneralCategory::Lo), // 36 chars
    (0x103C8, 0x103CF, GeneralCategory::Lo), // 8 chars
    (0x103D0, 0x103D0, GeneralCategory::Po),
    (0x103D1, 0x103D5, GeneralCategory::Nl), // 5 chars
    (0x10400, 0x10427, GeneralCategory::Lu), // 40 chars
    (0x10428, 0x1044F, GeneralCategory::Ll), // 40 chars
    (0x10450, 0x1049D, GeneralCategory::Lo), // 78 chars
    (0x104A0, 0x104A9, GeneralCategory::Nd), // 10 chars
    (0x104B0, 0x104D3, GeneralCategory::Lu), // 36 chars
    (0x104D8, 0x104FB, GeneralCategory::Ll), // 36 chars
    (0x10500, 0x10527, GeneralCategory::Lo), // 40 chars
    (0x10530, 0x10563, GeneralCategory::Lo), // 52 chars
    (0x1056F, 0x1056F, GeneralCategory::Po),
    (0x10570, 0x1057A, GeneralCategory::Lu), // 11 chars
    (0x1057C, 0x1058A, GeneralCategory::Lu), // 15 chars
    (0x1058C, 0x10592, GeneralCategory::Lu), // 7 chars
    (0x10594, 0x10595, GeneralCategory::Lu), // 2 chars
    (0x10597, 0x105A1, GeneralCategory::Ll), // 11 chars
    (0x105A3, 0x105B1, GeneralCategory::Ll), // 15 chars
    (0x105B3, 0x105B9, GeneralCategory::Ll), // 7 chars
    (0x105BB, 0x105BC, GeneralCategory::Ll), // 2 chars
    (0x105C0, 0x105F3, GeneralCategory::Lo), // 52 chars
    (0x10600, 0x10736, GeneralCategory::Lo), // 311 chars
    (0x10740, 0x10755, GeneralCategory::Lo), // 22 chars
    (0x10760, 0x10767, GeneralCategory::Lo), // 8 chars
    (0x10780, 0x10785, GeneralCategory::Lm), // 6 chars
    (0x10787, 0x107B0, GeneralCategory::Lm), // 42 chars
    (0x107B2, 0x107BA, GeneralCategory::Lm), // 9 chars
    (0x10800, 0x10805, GeneralCategory::Lo), // 6 chars
    (0x10808, 0x10808, GeneralCategory::Lo),
    (0x1080A, 0x10835, GeneralCategory::Lo), // 44 chars
    (0x10837, 0x10838, GeneralCategory::Lo), // 2 chars
    (0x1083C, 0x1083C, GeneralCategory::Lo),
    (0x1083F, 0x10855, GeneralCategory::Lo), // 23 chars
    (0x10857, 0x10857, GeneralCategory::Po),
    (0x10858, 0x1085F, GeneralCategory::No), // 8 chars
    (0x10860, 0x10876, GeneralCategory::Lo), // 23 chars
    (0x10877, 0x10878, GeneralCategory::So), // 2 chars
    (0x10879, 0x1087F, GeneralCategory::No), // 7 chars
    (0x10880, 0x1089E, GeneralCategory::Lo), // 31 chars
    (0x108A7, 0x108AF, GeneralCategory::No), // 9 chars
    (0x108E0, 0x108F2, GeneralCategory::Lo), // 19 chars
    (0x108F4, 0x108F5, GeneralCategory::Lo), // 2 chars
    (0x108FB, 0x108FF, GeneralCategory::No), // 5 chars
    (0x10900, 0x10915, GeneralCategory::Lo), // 22 chars
    (0x10916, 0x1091B, GeneralCategory::No), // 6 chars
    (0x1091F, 0x1091F, GeneralCategory::Po),
    (0x10920, 0x10939, GeneralCategory::Lo), // 26 chars
    (0x1093F, 0x1093F, GeneralCategory::Po),
    (0x10940, 0x10959, GeneralCategory::Lo), // 26 chars
    (0x10980, 0x109B7, GeneralCategory::Lo), // 56 chars
    (0x109BC, 0x109BD, GeneralCategory::No), // 2 chars
    (0x109BE, 0x109BF, GeneralCategory::Lo), // 2 chars
    (0x109C0, 0x109CF, GeneralCategory::No), // 16 chars
    (0x109D2, 0x109FF, GeneralCategory::No), // 46 chars
    (0x10A00, 0x10A00, GeneralCategory::Lo),
    (0x10A01, 0x10A03, GeneralCategory::Mn), // 3 chars
    (0x10A05, 0x10A06, GeneralCategory::Mn), // 2 chars
    (0x10A0C, 0x10A0F, GeneralCategory::Mn), // 4 chars
    (0x10A10, 0x10A13, GeneralCategory::Lo), // 4 chars
    (0x10A15, 0x10A17, GeneralCategory::Lo), // 3 chars
    (0x10A19, 0x10A35, GeneralCategory::Lo), // 29 chars
    (0x10A38, 0x10A3A, GeneralCategory::Mn), // 3 chars
    (0x10A3F, 0x10A3F, GeneralCategory::Mn),
    (0x10A40, 0x10A48, GeneralCategory::No), // 9 chars
    (0x10A50, 0x10A58, GeneralCategory::Po), // 9 chars
    (0x10A60, 0x10A7C, GeneralCategory::Lo), // 29 chars
    (0x10A7D, 0x10A7E, GeneralCategory::No), // 2 chars
    (0x10A7F, 0x10A7F, GeneralCategory::Po),
    (0x10A80, 0x10A9C, GeneralCategory::Lo), // 29 chars
    (0x10A9D, 0x10A9F, GeneralCategory::No), // 3 chars
    (0x10AC0, 0x10AC7, GeneralCategory::Lo), // 8 chars
    (0x10AC8, 0x10AC8, GeneralCategory::So),
    (0x10AC9, 0x10AE4, GeneralCategory::Lo), // 28 chars
    (0x10AE5, 0x10AE6, GeneralCategory::Mn), // 2 chars
    (0x10AEB, 0x10AEF, GeneralCategory::No), // 5 chars
    (0x10AF0, 0x10AF6, GeneralCategory::Po), // 7 chars
    (0x10B00, 0x10B35, GeneralCategory::Lo), // 54 chars
    (0x10B39, 0x10B3F, GeneralCategory::Po), // 7 chars
    (0x10B40, 0x10B55, GeneralCategory::Lo), // 22 chars
    (0x10B58, 0x10B5F, GeneralCategory::No), // 8 chars
    (0x10B60, 0x10B72, GeneralCategory::Lo), // 19 chars
    (0x10B78, 0x10B7F, GeneralCategory::No), // 8 chars
    (0x10B80, 0x10B91, GeneralCategory::Lo), // 18 chars
    (0x10B99, 0x10B9C, GeneralCategory::Po), // 4 chars
    (0x10BA9, 0x10BAF, GeneralCategory::No), // 7 chars
    (0x10C00, 0x10C48, GeneralCategory::Lo), // 73 chars
    (0x10C80, 0x10CB2, GeneralCategory::Lu), // 51 chars
    (0x10CC0, 0x10CF2, GeneralCategory::Ll), // 51 chars
    (0x10CFA, 0x10CFF, GeneralCategory::No), // 6 chars
    (0x10D00, 0x10D23, GeneralCategory::Lo), // 36 chars
    (0x10D24, 0x10D27, GeneralCategory::Mn), // 4 chars
    (0x10D30, 0x10D39, GeneralCategory::Nd), // 10 chars
    (0x10D40, 0x10D49, GeneralCategory::Nd), // 10 chars
    (0x10D4A, 0x10D4D, GeneralCategory::Lo), // 4 chars
    (0x10D4E, 0x10D4E, GeneralCategory::Lm),
    (0x10D4F, 0x10D4F, GeneralCategory::Lo),
    (0x10D50, 0x10D65, GeneralCategory::Lu), // 22 chars
    (0x10D69, 0x10D6D, GeneralCategory::Mn), // 5 chars
    (0x10D6E, 0x10D6E, GeneralCategory::Pd),
    (0x10D6F, 0x10D6F, GeneralCategory::Lm),
    (0x10D70, 0x10D85, GeneralCategory::Ll), // 22 chars
    (0x10D8E, 0x10D8F, GeneralCategory::Sm), // 2 chars
    (0x10E60, 0x10E7E, GeneralCategory::No), // 31 chars
    (0x10E80, 0x10EA9, GeneralCategory::Lo), // 42 chars
    (0x10EAB, 0x10EAC, GeneralCategory::Mn), // 2 chars
    (0x10EAD, 0x10EAD, GeneralCategory::Pd),
    (0x10EB0, 0x10EB1, GeneralCategory::Lo), // 2 chars
    (0x10EC2, 0x10EC4, GeneralCategory::Lo), // 3 chars
    (0x10EC5, 0x10EC5, GeneralCategory::Lm),
    (0x10EC6, 0x10EC7, GeneralCategory::Lo), // 2 chars
    (0x10ED0, 0x10ED0, GeneralCategory::Po),
    (0x10ED1, 0x10ED8, GeneralCategory::So), // 8 chars
    (0x10EFA, 0x10EFF, GeneralCategory::Mn), // 6 chars
    (0x10F00, 0x10F1C, GeneralCategory::Lo), // 29 chars
    (0x10F1D, 0x10F26, GeneralCategory::No), // 10 chars
    (0x10F27, 0x10F27, GeneralCategory::Lo),
    (0x10F30, 0x10F45, GeneralCategory::Lo), // 22 chars
    (0x10F46, 0x10F50, GeneralCategory::Mn), // 11 chars
    (0x10F51, 0x10F54, GeneralCategory::No), // 4 chars
    (0x10F55, 0x10F59, GeneralCategory::Po), // 5 chars
    (0x10F70, 0x10F81, GeneralCategory::Lo), // 18 chars
    (0x10F82, 0x10F85, GeneralCategory::Mn), // 4 chars
    (0x10F86, 0x10F89, GeneralCategory::Po), // 4 chars
    (0x10FB0, 0x10FC4, GeneralCategory::Lo), // 21 chars
    (0x10FC5, 0x10FCB, GeneralCategory::No), // 7 chars
    (0x10FE0, 0x10FF6, GeneralCategory::Lo), // 23 chars
    (0x11000, 0x11000, GeneralCategory::Mc),
    (0x11001, 0x11001, GeneralCategory::Mn),
    (0x11002, 0x11002, GeneralCategory::Mc),
    (0x11003, 0x11037, GeneralCategory::Lo), // 53 chars
    (0x11038, 0x11046, GeneralCategory::Mn), // 15 chars
    (0x11047, 0x1104D, GeneralCategory::Po), // 7 chars
    (0x11052, 0x11065, GeneralCategory::No), // 20 chars
    (0x11066, 0x1106F, GeneralCategory::Nd), // 10 chars
    (0x11070, 0x11070, GeneralCategory::Mn),
    (0x11071, 0x11072, GeneralCategory::Lo), // 2 chars
    (0x11073, 0x11074, GeneralCategory::Mn), // 2 chars
    (0x11075, 0x11075, GeneralCategory::Lo),
    (0x1107F, 0x11081, GeneralCategory::Mn), // 3 chars
    (0x11082, 0x11082, GeneralCategory::Mc),
    (0x11083, 0x110AF, GeneralCategory::Lo), // 45 chars
    (0x110B0, 0x110B2, GeneralCategory::Mc), // 3 chars
    (0x110B3, 0x110B6, GeneralCategory::Mn), // 4 chars
    (0x110B7, 0x110B8, GeneralCategory::Mc), // 2 chars
    (0x110B9, 0x110BA, GeneralCategory::Mn), // 2 chars
    (0x110BB, 0x110BC, GeneralCategory::Po), // 2 chars
    (0x110BD, 0x110BD, GeneralCategory::Cf),
    (0x110BE, 0x110C1, GeneralCategory::Po), // 4 chars
    (0x110C2, 0x110C2, GeneralCategory::Mn),
    (0x110CD, 0x110CD, GeneralCategory::Cf),
    (0x110D0, 0x110E8, GeneralCategory::Lo), // 25 chars
    (0x110F0, 0x110F9, GeneralCategory::Nd), // 10 chars
    (0x11100, 0x11102, GeneralCategory::Mn), // 3 chars
    (0x11103, 0x11126, GeneralCategory::Lo), // 36 chars
    (0x11127, 0x1112B, GeneralCategory::Mn), // 5 chars
    (0x1112C, 0x1112C, GeneralCategory::Mc),
    (0x1112D, 0x11134, GeneralCategory::Mn), // 8 chars
    (0x11136, 0x1113F, GeneralCategory::Nd), // 10 chars
    (0x11140, 0x11143, GeneralCategory::Po), // 4 chars
    (0x11144, 0x11144, GeneralCategory::Lo),
    (0x11145, 0x11146, GeneralCategory::Mc), // 2 chars
    (0x11147, 0x11147, GeneralCategory::Lo),
    (0x11150, 0x11172, GeneralCategory::Lo), // 35 chars
    (0x11173, 0x11173, GeneralCategory::Mn),
    (0x11174, 0x11175, GeneralCategory::Po), // 2 chars
    (0x11176, 0x11176, GeneralCategory::Lo),
    (0x11180, 0x11181, GeneralCategory::Mn), // 2 chars
    (0x11182, 0x11182, GeneralCategory::Mc),
    (0x11183, 0x111B2, GeneralCategory::Lo), // 48 chars
    (0x111B3, 0x111B5, GeneralCategory::Mc), // 3 chars
    (0x111B6, 0x111BE, GeneralCategory::Mn), // 9 chars
    (0x111BF, 0x111C0, GeneralCategory::Mc), // 2 chars
    (0x111C1, 0x111C4, GeneralCategory::Lo), // 4 chars
    (0x111C5, 0x111C8, GeneralCategory::Po), // 4 chars
    (0x111C9, 0x111CC, GeneralCategory::Mn), // 4 chars
    (0x111CD, 0x111CD, GeneralCategory::Po),
    (0x111CE, 0x111CE, GeneralCategory::Mc),
    (0x111CF, 0x111CF, GeneralCategory::Mn),
    (0x111D0, 0x111D9, GeneralCategory::Nd), // 10 chars
    (0x111DA, 0x111DA, GeneralCategory::Lo),
    (0x111DB, 0x111DB, GeneralCategory::Po),
    (0x111DC, 0x111DC, GeneralCategory::Lo),
    (0x111DD, 0x111DF, GeneralCategory::Po), // 3 chars
    (0x111E1, 0x111F4, GeneralCategory::No), // 20 chars
    (0x11200, 0x11211, GeneralCategory::Lo), // 18 chars
    (0x11213, 0x1122B, GeneralCategory::Lo), // 25 chars
    (0x1122C, 0x1122E, GeneralCategory::Mc), // 3 chars
    (0x1122F, 0x11231, GeneralCategory::Mn), // 3 chars
    (0x11232, 0x11233, GeneralCategory::Mc), // 2 chars
    (0x11234, 0x11234, GeneralCategory::Mn),
    (0x11235, 0x11235, GeneralCategory::Mc),
    (0x11236, 0x11237, GeneralCategory::Mn), // 2 chars
    (0x11238, 0x1123D, GeneralCategory::Po), // 6 chars
    (0x1123E, 0x1123E, GeneralCategory::Mn),
    (0x1123F, 0x11240, GeneralCategory::Lo), // 2 chars
    (0x11241, 0x11241, GeneralCategory::Mn),
    (0x11280, 0x11286, GeneralCategory::Lo), // 7 chars
    (0x11288, 0x11288, GeneralCategory::Lo),
    (0x1128A, 0x1128D, GeneralCategory::Lo), // 4 chars
    (0x1128F, 0x1129D, GeneralCategory::Lo), // 15 chars
    (0x1129F, 0x112A8, GeneralCategory::Lo), // 10 chars
    (0x112A9, 0x112A9, GeneralCategory::Po),
    (0x112B0, 0x112DE, GeneralCategory::Lo), // 47 chars
    (0x112DF, 0x112DF, GeneralCategory::Mn),
    (0x112E0, 0x112E2, GeneralCategory::Mc), // 3 chars
    (0x112E3, 0x112EA, GeneralCategory::Mn), // 8 chars
    (0x112F0, 0x112F9, GeneralCategory::Nd), // 10 chars
    (0x11300, 0x11301, GeneralCategory::Mn), // 2 chars
    (0x11302, 0x11303, GeneralCategory::Mc), // 2 chars
    (0x11305, 0x1130C, GeneralCategory::Lo), // 8 chars
    (0x1130F, 0x11310, GeneralCategory::Lo), // 2 chars
    (0x11313, 0x11328, GeneralCategory::Lo), // 22 chars
    (0x1132A, 0x11330, GeneralCategory::Lo), // 7 chars
    (0x11332, 0x11333, GeneralCategory::Lo), // 2 chars
    (0x11335, 0x11339, GeneralCategory::Lo), // 5 chars
    (0x1133B, 0x1133C, GeneralCategory::Mn), // 2 chars
    (0x1133D, 0x1133D, GeneralCategory::Lo),
    (0x1133E, 0x1133F, GeneralCategory::Mc), // 2 chars
    (0x11340, 0x11340, GeneralCategory::Mn),
    (0x11341, 0x11344, GeneralCategory::Mc), // 4 chars
    (0x11347, 0x11348, GeneralCategory::Mc), // 2 chars
    (0x1134B, 0x1134D, GeneralCategory::Mc), // 3 chars
    (0x11350, 0x11350, GeneralCategory::Lo),
    (0x11357, 0x11357, GeneralCategory::Mc),
    (0x1135D, 0x11361, GeneralCategory::Lo), // 5 chars
    (0x11362, 0x11363, GeneralCategory::Mc), // 2 chars
    (0x11366, 0x1136C, GeneralCategory::Mn), // 7 chars
    (0x11370, 0x11374, GeneralCategory::Mn), // 5 chars
    (0x11380, 0x11389, GeneralCategory::Lo), // 10 chars
    (0x1138B, 0x1138B, GeneralCategory::Lo),
    (0x1138E, 0x1138E, GeneralCategory::Lo),
    (0x11390, 0x113B5, GeneralCategory::Lo), // 38 chars
    (0x113B7, 0x113B7, GeneralCategory::Lo),
    (0x113B8, 0x113BA, GeneralCategory::Mc), // 3 chars
    (0x113BB, 0x113C0, GeneralCategory::Mn), // 6 chars
    (0x113C2, 0x113C2, GeneralCategory::Mc),
    (0x113C5, 0x113C5, GeneralCategory::Mc),
    (0x113C7, 0x113CA, GeneralCategory::Mc), // 4 chars
    (0x113CC, 0x113CD, GeneralCategory::Mc), // 2 chars
    (0x113CE, 0x113CE, GeneralCategory::Mn),
    (0x113CF, 0x113CF, GeneralCategory::Mc),
    (0x113D0, 0x113D0, GeneralCategory::Mn),
    (0x113D1, 0x113D1, GeneralCategory::Lo),
    (0x113D2, 0x113D2, GeneralCategory::Mn),
    (0x113D3, 0x113D3, GeneralCategory::Lo),
    (0x113D4, 0x113D5, GeneralCategory::Po), // 2 chars
    (0x113D7, 0x113D8, GeneralCategory::Po), // 2 chars
    (0x113E1, 0x113E2, GeneralCategory::Mn), // 2 chars
    (0x11400, 0x11434, GeneralCategory::Lo), // 53 chars
    (0x11435, 0x11437, GeneralCategory::Mc), // 3 chars
    (0x11438, 0x1143F, GeneralCategory::Mn), // 8 chars
    (0x11440, 0x11441, GeneralCategory::Mc), // 2 chars
    (0x11442, 0x11444, GeneralCategory::Mn), // 3 chars
    (0x11445, 0x11445, GeneralCategory::Mc),
    (0x11446, 0x11446, GeneralCategory::Mn),
    (0x11447, 0x1144A, GeneralCategory::Lo), // 4 chars
    (0x1144B, 0x1144F, GeneralCategory::Po), // 5 chars
    (0x11450, 0x11459, GeneralCategory::Nd), // 10 chars
    (0x1145A, 0x1145B, GeneralCategory::Po), // 2 chars
    (0x1145D, 0x1145D, GeneralCategory::Po),
    (0x1145E, 0x1145E, GeneralCategory::Mn),
    (0x1145F, 0x11461, GeneralCategory::Lo), // 3 chars
    (0x11480, 0x114AF, GeneralCategory::Lo), // 48 chars
    (0x114B0, 0x114B2, GeneralCategory::Mc), // 3 chars
    (0x114B3, 0x114B8, GeneralCategory::Mn), // 6 chars
    (0x114B9, 0x114B9, GeneralCategory::Mc),
    (0x114BA, 0x114BA, GeneralCategory::Mn),
    (0x114BB, 0x114BE, GeneralCategory::Mc), // 4 chars
    (0x114BF, 0x114C0, GeneralCategory::Mn), // 2 chars
    (0x114C1, 0x114C1, GeneralCategory::Mc),
    (0x114C2, 0x114C3, GeneralCategory::Mn), // 2 chars
    (0x114C4, 0x114C5, GeneralCategory::Lo), // 2 chars
    (0x114C6, 0x114C6, GeneralCategory::Po),
    (0x114C7, 0x114C7, GeneralCategory::Lo),
    (0x114D0, 0x114D9, GeneralCategory::Nd), // 10 chars
    (0x11580, 0x115AE, GeneralCategory::Lo), // 47 chars
    (0x115AF, 0x115B1, GeneralCategory::Mc), // 3 chars
    (0x115B2, 0x115B5, GeneralCategory::Mn), // 4 chars
    (0x115B8, 0x115BB, GeneralCategory::Mc), // 4 chars
    (0x115BC, 0x115BD, GeneralCategory::Mn), // 2 chars
    (0x115BE, 0x115BE, GeneralCategory::Mc),
    (0x115BF, 0x115C0, GeneralCategory::Mn), // 2 chars
    (0x115C1, 0x115D7, GeneralCategory::Po), // 23 chars
    (0x115D8, 0x115DB, GeneralCategory::Lo), // 4 chars
    (0x115DC, 0x115DD, GeneralCategory::Mn), // 2 chars
    (0x11600, 0x1162F, GeneralCategory::Lo), // 48 chars
    (0x11630, 0x11632, GeneralCategory::Mc), // 3 chars
    (0x11633, 0x1163A, GeneralCategory::Mn), // 8 chars
    (0x1163B, 0x1163C, GeneralCategory::Mc), // 2 chars
    (0x1163D, 0x1163D, GeneralCategory::Mn),
    (0x1163E, 0x1163E, GeneralCategory::Mc),
    (0x1163F, 0x11640, GeneralCategory::Mn), // 2 chars
    (0x11641, 0x11643, GeneralCategory::Po), // 3 chars
    (0x11644, 0x11644, GeneralCategory::Lo),
    (0x11650, 0x11659, GeneralCategory::Nd), // 10 chars
    (0x11660, 0x1166C, GeneralCategory::Po), // 13 chars
    (0x11680, 0x116AA, GeneralCategory::Lo), // 43 chars
    (0x116AB, 0x116AB, GeneralCategory::Mn),
    (0x116AC, 0x116AC, GeneralCategory::Mc),
    (0x116AD, 0x116AD, GeneralCategory::Mn),
    (0x116AE, 0x116AF, GeneralCategory::Mc), // 2 chars
    (0x116B0, 0x116B5, GeneralCategory::Mn), // 6 chars
    (0x116B6, 0x116B6, GeneralCategory::Mc),
    (0x116B7, 0x116B7, GeneralCategory::Mn),
    (0x116B8, 0x116B8, GeneralCategory::Lo),
    (0x116B9, 0x116B9, GeneralCategory::Po),
    (0x116C0, 0x116C9, GeneralCategory::Nd), // 10 chars
    (0x116D0, 0x116E3, GeneralCategory::Nd), // 20 chars
    (0x11700, 0x1171A, GeneralCategory::Lo), // 27 chars
    (0x1171D, 0x1171D, GeneralCategory::Mn),
    (0x1171E, 0x1171E, GeneralCategory::Mc),
    (0x1171F, 0x1171F, GeneralCategory::Mn),
    (0x11720, 0x11721, GeneralCategory::Mc), // 2 chars
    (0x11722, 0x11725, GeneralCategory::Mn), // 4 chars
    (0x11726, 0x11726, GeneralCategory::Mc),
    (0x11727, 0x1172B, GeneralCategory::Mn), // 5 chars
    (0x11730, 0x11739, GeneralCategory::Nd), // 10 chars
    (0x1173A, 0x1173B, GeneralCategory::No), // 2 chars
    (0x1173C, 0x1173E, GeneralCategory::Po), // 3 chars
    (0x1173F, 0x1173F, GeneralCategory::So),
    (0x11740, 0x11746, GeneralCategory::Lo), // 7 chars
    (0x11800, 0x1182B, GeneralCategory::Lo), // 44 chars
    (0x1182C, 0x1182E, GeneralCategory::Mc), // 3 chars
    (0x1182F, 0x11837, GeneralCategory::Mn), // 9 chars
    (0x11838, 0x11838, GeneralCategory::Mc),
    (0x11839, 0x1183A, GeneralCategory::Mn), // 2 chars
    (0x1183B, 0x1183B, GeneralCategory::Po),
    (0x118A0, 0x118BF, GeneralCategory::Lu), // 32 chars
    (0x118C0, 0x118DF, GeneralCategory::Ll), // 32 chars
    (0x118E0, 0x118E9, GeneralCategory::Nd), // 10 chars
    (0x118EA, 0x118F2, GeneralCategory::No), // 9 chars
    (0x118FF, 0x11906, GeneralCategory::Lo), // 8 chars
    (0x11909, 0x11909, GeneralCategory::Lo),
    (0x1190C, 0x11913, GeneralCategory::Lo), // 8 chars
    (0x11915, 0x11916, GeneralCategory::Lo), // 2 chars
    (0x11918, 0x1192F, GeneralCategory::Lo), // 24 chars
    (0x11930, 0x11935, GeneralCategory::Mc), // 6 chars
    (0x11937, 0x11938, GeneralCategory::Mc), // 2 chars
    (0x1193B, 0x1193C, GeneralCategory::Mn), // 2 chars
    (0x1193D, 0x1193D, GeneralCategory::Mc),
    (0x1193E, 0x1193E, GeneralCategory::Mn),
    (0x1193F, 0x1193F, GeneralCategory::Lo),
    (0x11940, 0x11940, GeneralCategory::Mc),
    (0x11941, 0x11941, GeneralCategory::Lo),
    (0x11942, 0x11942, GeneralCategory::Mc),
    (0x11943, 0x11943, GeneralCategory::Mn),
    (0x11944, 0x11946, GeneralCategory::Po), // 3 chars
    (0x11950, 0x11959, GeneralCategory::Nd), // 10 chars
    (0x119A0, 0x119A7, GeneralCategory::Lo), // 8 chars
    (0x119AA, 0x119D0, GeneralCategory::Lo), // 39 chars
    (0x119D1, 0x119D3, GeneralCategory::Mc), // 3 chars
    (0x119D4, 0x119D7, GeneralCategory::Mn), // 4 chars
    (0x119DA, 0x119DB, GeneralCategory::Mn), // 2 chars
    (0x119DC, 0x119DF, GeneralCategory::Mc), // 4 chars
    (0x119E0, 0x119E0, GeneralCategory::Mn),
    (0x119E1, 0x119E1, GeneralCategory::Lo),
    (0x119E2, 0x119E2, GeneralCategory::Po),
    (0x119E3, 0x119E3, GeneralCategory::Lo),
    (0x119E4, 0x119E4, GeneralCategory::Mc),
    (0x11A00, 0x11A00, GeneralCategory::Lo),
    (0x11A01, 0x11A0A, GeneralCategory::Mn), // 10 chars
    (0x11A0B, 0x11A32, GeneralCategory::Lo), // 40 chars
    (0x11A33, 0x11A38, GeneralCategory::Mn), // 6 chars
    (0x11A39, 0x11A39, GeneralCategory::Mc),
    (0x11A3A, 0x11A3A, GeneralCategory::Lo),
    (0x11A3B, 0x11A3E, GeneralCategory::Mn), // 4 chars
    (0x11A3F, 0x11A46, GeneralCategory::Po), // 8 chars
    (0x11A47, 0x11A47, GeneralCategory::Mn),
    (0x11A50, 0x11A50, GeneralCategory::Lo),
    (0x11A51, 0x11A56, GeneralCategory::Mn), // 6 chars
    (0x11A57, 0x11A58, GeneralCategory::Mc), // 2 chars
    (0x11A59, 0x11A5B, GeneralCategory::Mn), // 3 chars
    (0x11A5C, 0x11A89, GeneralCategory::Lo), // 46 chars
    (0x11A8A, 0x11A96, GeneralCategory::Mn), // 13 chars
    (0x11A97, 0x11A97, GeneralCategory::Mc),
    (0x11A98, 0x11A99, GeneralCategory::Mn), // 2 chars
    (0x11A9A, 0x11A9C, GeneralCategory::Po), // 3 chars
    (0x11A9D, 0x11A9D, GeneralCategory::Lo),
    (0x11A9E, 0x11AA2, GeneralCategory::Po), // 5 chars
    (0x11AB0, 0x11AF8, GeneralCategory::Lo), // 73 chars
    (0x11B00, 0x11B09, GeneralCategory::Po), // 10 chars
    (0x11B60, 0x11B60, GeneralCategory::Mn),
    (0x11B61, 0x11B61, GeneralCategory::Mc),
    (0x11B62, 0x11B64, GeneralCategory::Mn), // 3 chars
    (0x11B65, 0x11B65, GeneralCategory::Mc),
    (0x11B66, 0x11B66, GeneralCategory::Mn),
    (0x11B67, 0x11B67, GeneralCategory::Mc),
    (0x11BC0, 0x11BE0, GeneralCategory::Lo), // 33 chars
    (0x11BE1, 0x11BE1, GeneralCategory::Po),
    (0x11BF0, 0x11BF9, GeneralCategory::Nd), // 10 chars
    (0x11C00, 0x11C08, GeneralCategory::Lo), // 9 chars
    (0x11C0A, 0x11C2E, GeneralCategory::Lo), // 37 chars
    (0x11C2F, 0x11C2F, GeneralCategory::Mc),
    (0x11C30, 0x11C36, GeneralCategory::Mn), // 7 chars
    (0x11C38, 0x11C3D, GeneralCategory::Mn), // 6 chars
    (0x11C3E, 0x11C3E, GeneralCategory::Mc),
    (0x11C3F, 0x11C3F, GeneralCategory::Mn),
    (0x11C40, 0x11C40, GeneralCategory::Lo),
    (0x11C41, 0x11C45, GeneralCategory::Po), // 5 chars
    (0x11C50, 0x11C59, GeneralCategory::Nd), // 10 chars
    (0x11C5A, 0x11C6C, GeneralCategory::No), // 19 chars
    (0x11C70, 0x11C71, GeneralCategory::Po), // 2 chars
    (0x11C72, 0x11C8F, GeneralCategory::Lo), // 30 chars
    (0x11C92, 0x11CA7, GeneralCategory::Mn), // 22 chars
    (0x11CA9, 0x11CA9, GeneralCategory::Mc),
    (0x11CAA, 0x11CB0, GeneralCategory::Mn), // 7 chars
    (0x11CB1, 0x11CB1, GeneralCategory::Mc),
    (0x11CB2, 0x11CB3, GeneralCategory::Mn), // 2 chars
    (0x11CB4, 0x11CB4, GeneralCategory::Mc),
    (0x11CB5, 0x11CB6, GeneralCategory::Mn), // 2 chars
    (0x11D00, 0x11D06, GeneralCategory::Lo), // 7 chars
    (0x11D08, 0x11D09, GeneralCategory::Lo), // 2 chars
    (0x11D0B, 0x11D30, GeneralCategory::Lo), // 38 chars
    (0x11D31, 0x11D36, GeneralCategory::Mn), // 6 chars
    (0x11D3A, 0x11D3A, GeneralCategory::Mn),
    (0x11D3C, 0x11D3D, GeneralCategory::Mn), // 2 chars
    (0x11D3F, 0x11D45, GeneralCategory::Mn), // 7 chars
    (0x11D46, 0x11D46, GeneralCategory::Lo),
    (0x11D47, 0x11D47, GeneralCategory::Mn),
    (0x11D50, 0x11D59, GeneralCategory::Nd), // 10 chars
    (0x11D60, 0x11D65, GeneralCategory::Lo), // 6 chars
    (0x11D67, 0x11D68, GeneralCategory::Lo), // 2 chars
    (0x11D6A, 0x11D89, GeneralCategory::Lo), // 32 chars
    (0x11D8A, 0x11D8E, GeneralCategory::Mc), // 5 chars
    (0x11D90, 0x11D91, GeneralCategory::Mn), // 2 chars
    (0x11D93, 0x11D94, GeneralCategory::Mc), // 2 chars
    (0x11D95, 0x11D95, GeneralCategory::Mn),
    (0x11D96, 0x11D96, GeneralCategory::Mc),
    (0x11D97, 0x11D97, GeneralCategory::Mn),
    (0x11D98, 0x11D98, GeneralCategory::Lo),
    (0x11DA0, 0x11DA9, GeneralCategory::Nd), // 10 chars
    (0x11DB0, 0x11DD8, GeneralCategory::Lo), // 41 chars
    (0x11DD9, 0x11DD9, GeneralCategory::Lm),
    (0x11DDA, 0x11DDB, GeneralCategory::Lo), // 2 chars
    (0x11DE0, 0x11DE9, GeneralCategory::Nd), // 10 chars
    (0x11EE0, 0x11EF2, GeneralCategory::Lo), // 19 chars
    (0x11EF3, 0x11EF4, GeneralCategory::Mn), // 2 chars
    (0x11EF5, 0x11EF6, GeneralCategory::Mc), // 2 chars
    (0x11EF7, 0x11EF8, GeneralCategory::Po), // 2 chars
    (0x11F00, 0x11F01, GeneralCategory::Mn), // 2 chars
    (0x11F02, 0x11F02, GeneralCategory::Lo),
    (0x11F03, 0x11F03, GeneralCategory::Mc),
    (0x11F04, 0x11F10, GeneralCategory::Lo), // 13 chars
    (0x11F12, 0x11F33, GeneralCategory::Lo), // 34 chars
    (0x11F34, 0x11F35, GeneralCategory::Mc), // 2 chars
    (0x11F36, 0x11F3A, GeneralCategory::Mn), // 5 chars
    (0x11F3E, 0x11F3F, GeneralCategory::Mc), // 2 chars
    (0x11F40, 0x11F40, GeneralCategory::Mn),
    (0x11F41, 0x11F41, GeneralCategory::Mc),
    (0x11F42, 0x11F42, GeneralCategory::Mn),
    (0x11F43, 0x11F4F, GeneralCategory::Po), // 13 chars
    (0x11F50, 0x11F59, GeneralCategory::Nd), // 10 chars
    (0x11F5A, 0x11F5A, GeneralCategory::Mn),
    (0x11FB0, 0x11FB0, GeneralCategory::Lo),
    (0x11FC0, 0x11FD4, GeneralCategory::No), // 21 chars
    (0x11FD5, 0x11FDC, GeneralCategory::So), // 8 chars
    (0x11FDD, 0x11FE0, GeneralCategory::Sc), // 4 chars
    (0x11FE1, 0x11FF1, GeneralCategory::So), // 17 chars
    (0x11FFF, 0x11FFF, GeneralCategory::Po),
    (0x12000, 0x12399, GeneralCategory::Lo), // 922 chars
    (0x12400, 0x1246E, GeneralCategory::Nl), // 111 chars
    (0x12470, 0x12474, GeneralCategory::Po), // 5 chars
    (0x12480, 0x12543, GeneralCategory::Lo), // 196 chars
    (0x12F90, 0x12FF0, GeneralCategory::Lo), // 97 chars
    (0x12FF1, 0x12FF2, GeneralCategory::Po), // 2 chars
    (0x13000, 0x1342F, GeneralCategory::Lo), // 1072 chars
    (0x13430, 0x1343F, GeneralCategory::Cf), // 16 chars
    (0x13440, 0x13440, GeneralCategory::Mn),
    (0x13441, 0x13446, GeneralCategory::Lo), // 6 chars
    (0x13447, 0x13455, GeneralCategory::Mn), // 15 chars
    (0x13460, 0x143FA, GeneralCategory::Lo), // 3995 chars
    (0x14400, 0x14646, GeneralCategory::Lo), // 583 chars
    (0x16100, 0x1611D, GeneralCategory::Lo), // 30 chars
    (0x1611E, 0x16129, GeneralCategory::Mn), // 12 chars
    (0x1612A, 0x1612C, GeneralCategory::Mc), // 3 chars
    (0x1612D, 0x1612F, GeneralCategory::Mn), // 3 chars
    (0x16130, 0x16139, GeneralCategory::Nd), // 10 chars
    (0x16800, 0x16A38, GeneralCategory::Lo), // 569 chars
    (0x16A40, 0x16A5E, GeneralCategory::Lo), // 31 chars
    (0x16A60, 0x16A69, GeneralCategory::Nd), // 10 chars
    (0x16A6E, 0x16A6F, GeneralCategory::Po), // 2 chars
    (0x16A70, 0x16ABE, GeneralCategory::Lo), // 79 chars
    (0x16AC0, 0x16AC9, GeneralCategory::Nd), // 10 chars
    (0x16AD0, 0x16AED, GeneralCategory::Lo), // 30 chars
    (0x16AF0, 0x16AF4, GeneralCategory::Mn), // 5 chars
    (0x16AF5, 0x16AF5, GeneralCategory::Po),
    (0x16B00, 0x16B2F, GeneralCategory::Lo), // 48 chars
    (0x16B30, 0x16B36, GeneralCategory::Mn), // 7 chars
    (0x16B37, 0x16B3B, GeneralCategory::Po), // 5 chars
    (0x16B3C, 0x16B3F, GeneralCategory::So), // 4 chars
    (0x16B40, 0x16B43, GeneralCategory::Lm), // 4 chars
    (0x16B44, 0x16B44, GeneralCategory::Po),
    (0x16B45, 0x16B45, GeneralCategory::So),
    (0x16B50, 0x16B59, GeneralCategory::Nd), // 10 chars
    (0x16B5B, 0x16B61, GeneralCategory::No), // 7 chars
    (0x16B63, 0x16B77, GeneralCategory::Lo), // 21 chars
    (0x16B7D, 0x16B8F, GeneralCategory::Lo), // 19 chars
    (0x16D40, 0x16D42, GeneralCategory::Lm), // 3 chars
    (0x16D43, 0x16D6A, GeneralCategory::Lo), // 40 chars
    (0x16D6B, 0x16D6C, GeneralCategory::Lm), // 2 chars
    (0x16D6D, 0x16D6F, GeneralCategory::Po), // 3 chars
    (0x16D70, 0x16D79, GeneralCategory::Nd), // 10 chars
    (0x16E40, 0x16E5F, GeneralCategory::Lu), // 32 chars
    (0x16E60, 0x16E7F, GeneralCategory::Ll), // 32 chars
    (0x16E80, 0x16E96, GeneralCategory::No), // 23 chars
    (0x16E97, 0x16E9A, GeneralCategory::Po), // 4 chars
    (0x16EA0, 0x16EB8, GeneralCategory::Lu), // 25 chars
    (0x16EBB, 0x16ED3, GeneralCategory::Ll), // 25 chars
    (0x16F00, 0x16F4A, GeneralCategory::Lo), // 75 chars
    (0x16F4F, 0x16F4F, GeneralCategory::Mn),
    (0x16F50, 0x16F50, GeneralCategory::Lo),
    (0x16F51, 0x16F87, GeneralCategory::Mc), // 55 chars
    (0x16F8F, 0x16F92, GeneralCategory::Mn), // 4 chars
    (0x16F93, 0x16F9F, GeneralCategory::Lm), // 13 chars
    (0x16FE0, 0x16FE1, GeneralCategory::Lm), // 2 chars
    (0x16FE2, 0x16FE2, GeneralCategory::Po),
    (0x16FE3, 0x16FE3, GeneralCategory::Lm),
    (0x16FE4, 0x16FE4, GeneralCategory::Mn),
    (0x16FF0, 0x16FF1, GeneralCategory::Mc), // 2 chars
    (0x16FF2, 0x16FF3, GeneralCategory::Lm), // 2 chars
    (0x16FF4, 0x16FF6, GeneralCategory::Nl), // 3 chars
    (0x17000, 0x18CD5, GeneralCategory::Lo), // 7382 chars
    (0x18CFF, 0x18D1E, GeneralCategory::Lo), // 32 chars
    (0x18D80, 0x18DF2, GeneralCategory::Lo), // 115 chars
    (0x1AFF0, 0x1AFF3, GeneralCategory::Lm), // 4 chars
    (0x1AFF5, 0x1AFFB, GeneralCategory::Lm), // 7 chars
    (0x1AFFD, 0x1AFFE, GeneralCategory::Lm), // 2 chars
    (0x1B000, 0x1B122, GeneralCategory::Lo), // 291 chars
    (0x1B132, 0x1B132, GeneralCategory::Lo),
    (0x1B150, 0x1B152, GeneralCategory::Lo), // 3 chars
    (0x1B155, 0x1B155, GeneralCategory::Lo),
    (0x1B164, 0x1B167, GeneralCategory::Lo), // 4 chars
    (0x1B170, 0x1B2FB, GeneralCategory::Lo), // 396 chars
    (0x1BC00, 0x1BC6A, GeneralCategory::Lo), // 107 chars
    (0x1BC70, 0x1BC7C, GeneralCategory::Lo), // 13 chars
    (0x1BC80, 0x1BC88, GeneralCategory::Lo), // 9 chars
    (0x1BC90, 0x1BC99, GeneralCategory::Lo), // 10 chars
    (0x1BC9C, 0x1BC9C, GeneralCategory::So),
    (0x1BC9D, 0x1BC9E, GeneralCategory::Mn), // 2 chars
    (0x1BC9F, 0x1BC9F, GeneralCategory::Po),
    (0x1BCA0, 0x1BCA3, GeneralCategory::Cf), // 4 chars
    (0x1CC00, 0x1CCEF, GeneralCategory::So), // 240 chars
    (0x1CCF0, 0x1CCF9, GeneralCategory::Nd), // 10 chars
    (0x1CCFA, 0x1CCFC, GeneralCategory::So), // 3 chars
    (0x1CD00, 0x1CEB3, GeneralCategory::So), // 436 chars
    (0x1CEBA, 0x1CED0, GeneralCategory::So), // 23 chars
    (0x1CEE0, 0x1CEEF, GeneralCategory::So), // 16 chars
    (0x1CEF0, 0x1CEF0, GeneralCategory::Sm),
    (0x1CF00, 0x1CF2D, GeneralCategory::Mn), // 46 chars
    (0x1CF30, 0x1CF46, GeneralCategory::Mn), // 23 chars
    (0x1CF50, 0x1CFC3, GeneralCategory::So), // 116 chars
    (0x1D000, 0x1D0F5, GeneralCategory::So), // 246 chars
    (0x1D100, 0x1D126, GeneralCategory::So), // 39 chars
    (0x1D129, 0x1D164, GeneralCategory::So), // 60 chars
    (0x1D165, 0x1D166, GeneralCategory::Mc), // 2 chars
    (0x1D167, 0x1D169, GeneralCategory::Mn), // 3 chars
    (0x1D16A, 0x1D16C, GeneralCategory::So), // 3 chars
    (0x1D16D, 0x1D172, GeneralCategory::Mc), // 6 chars
    (0x1D173, 0x1D17A, GeneralCategory::Cf), // 8 chars
    (0x1D17B, 0x1D182, GeneralCategory::Mn), // 8 chars
    (0x1D183, 0x1D184, GeneralCategory::So), // 2 chars
    (0x1D185, 0x1D18B, GeneralCategory::Mn), // 7 chars
    (0x1D18C, 0x1D1A9, GeneralCategory::So), // 30 chars
    (0x1D1AA, 0x1D1AD, GeneralCategory::Mn), // 4 chars
    (0x1D1AE, 0x1D1EA, GeneralCategory::So), // 61 chars
    (0x1D200, 0x1D241, GeneralCategory::So), // 66 chars
    (0x1D242, 0x1D244, GeneralCategory::Mn), // 3 chars
    (0x1D245, 0x1D245, GeneralCategory::So),
    (0x1D2C0, 0x1D2D3, GeneralCategory::No), // 20 chars
    (0x1D2E0, 0x1D2F3, GeneralCategory::No), // 20 chars
    (0x1D300, 0x1D356, GeneralCategory::So), // 87 chars
    (0x1D360, 0x1D378, GeneralCategory::No), // 25 chars
    (0x1D400, 0x1D419, GeneralCategory::Lu), // 26 chars
    (0x1D41A, 0x1D433, GeneralCategory::Ll), // 26 chars
    (0x1D434, 0x1D44D, GeneralCategory::Lu), // 26 chars
    (0x1D44E, 0x1D454, GeneralCategory::Ll), // 7 chars
    (0x1D456, 0x1D467, GeneralCategory::Ll), // 18 chars
    (0x1D468, 0x1D481, GeneralCategory::Lu), // 26 chars
    (0x1D482, 0x1D49B, GeneralCategory::Ll), // 26 chars
    (0x1D49C, 0x1D49C, GeneralCategory::Lu),
    (0x1D49E, 0x1D49F, GeneralCategory::Lu), // 2 chars
    (0x1D4A2, 0x1D4A2, GeneralCategory::Lu),
    (0x1D4A5, 0x1D4A6, GeneralCategory::Lu), // 2 chars
    (0x1D4A9, 0x1D4AC, GeneralCategory::Lu), // 4 chars
    (0x1D4AE, 0x1D4B5, GeneralCategory::Lu), // 8 chars
    (0x1D4B6, 0x1D4B9, GeneralCategory::Ll), // 4 chars
    (0x1D4BB, 0x1D4BB, GeneralCategory::Ll),
    (0x1D4BD, 0x1D4C3, GeneralCategory::Ll), // 7 chars
    (0x1D4C5, 0x1D4CF, GeneralCategory::Ll), // 11 chars
    (0x1D4D0, 0x1D4E9, GeneralCategory::Lu), // 26 chars
    (0x1D4EA, 0x1D503, GeneralCategory::Ll), // 26 chars
    (0x1D504, 0x1D505, GeneralCategory::Lu), // 2 chars
    (0x1D507, 0x1D50A, GeneralCategory::Lu), // 4 chars
    (0x1D50D, 0x1D514, GeneralCategory::Lu), // 8 chars
    (0x1D516, 0x1D51C, GeneralCategory::Lu), // 7 chars
    (0x1D51E, 0x1D537, GeneralCategory::Ll), // 26 chars
    (0x1D538, 0x1D539, GeneralCategory::Lu), // 2 chars
    (0x1D53B, 0x1D53E, GeneralCategory::Lu), // 4 chars
    (0x1D540, 0x1D544, GeneralCategory::Lu), // 5 chars
    (0x1D546, 0x1D546, GeneralCategory::Lu),
    (0x1D54A, 0x1D550, GeneralCategory::Lu), // 7 chars
    (0x1D552, 0x1D56B, GeneralCategory::Ll), // 26 chars
    (0x1D56C, 0x1D585, GeneralCategory::Lu), // 26 chars
    (0x1D586, 0x1D59F, GeneralCategory::Ll), // 26 chars
    (0x1D5A0, 0x1D5B9, GeneralCategory::Lu), // 26 chars
    (0x1D5BA, 0x1D5D3, GeneralCategory::Ll), // 26 chars
    (0x1D5D4, 0x1D5ED, GeneralCategory::Lu), // 26 chars
    (0x1D5EE, 0x1D607, GeneralCategory::Ll), // 26 chars
    (0x1D608, 0x1D621, GeneralCategory::Lu), // 26 chars
    (0x1D622, 0x1D63B, GeneralCategory::Ll), // 26 chars
    (0x1D63C, 0x1D655, GeneralCategory::Lu), // 26 chars
    (0x1D656, 0x1D66F, GeneralCategory::Ll), // 26 chars
    (0x1D670, 0x1D689, GeneralCategory::Lu), // 26 chars
    (0x1D68A, 0x1D6A5, GeneralCategory::Ll), // 28 chars
    (0x1D6A8, 0x1D6C0, GeneralCategory::Lu), // 25 chars
    (0x1D6C1, 0x1D6C1, GeneralCategory::Sm),
    (0x1D6C2, 0x1D6DA, GeneralCategory::Ll), // 25 chars
    (0x1D6DB, 0x1D6DB, GeneralCategory::Sm),
    (0x1D6DC, 0x1D6E1, GeneralCategory::Ll), // 6 chars
    (0x1D6E2, 0x1D6FA, GeneralCategory::Lu), // 25 chars
    (0x1D6FB, 0x1D6FB, GeneralCategory::Sm),
    (0x1D6FC, 0x1D714, GeneralCategory::Ll), // 25 chars
    (0x1D715, 0x1D715, GeneralCategory::Sm),
    (0x1D716, 0x1D71B, GeneralCategory::Ll), // 6 chars
    (0x1D71C, 0x1D734, GeneralCategory::Lu), // 25 chars
    (0x1D735, 0x1D735, GeneralCategory::Sm),
    (0x1D736, 0x1D74E, GeneralCategory::Ll), // 25 chars
    (0x1D74F, 0x1D74F, GeneralCategory::Sm),
    (0x1D750, 0x1D755, GeneralCategory::Ll), // 6 chars
    (0x1D756, 0x1D76E, GeneralCategory::Lu), // 25 chars
    (0x1D76F, 0x1D76F, GeneralCategory::Sm),
    (0x1D770, 0x1D788, GeneralCategory::Ll), // 25 chars
    (0x1D789, 0x1D789, GeneralCategory::Sm),
    (0x1D78A, 0x1D78F, GeneralCategory::Ll), // 6 chars
    (0x1D790, 0x1D7A8, GeneralCategory::Lu), // 25 chars
    (0x1D7A9, 0x1D7A9, GeneralCategory::Sm),
    (0x1D7AA, 0x1D7C2, GeneralCategory::Ll), // 25 chars
    (0x1D7C3, 0x1D7C3, GeneralCategory::Sm),
    (0x1D7C4, 0x1D7C9, GeneralCategory::Ll), // 6 chars
    (0x1D7CA, 0x1D7CA, GeneralCategory::Lu),
    (0x1D7CB, 0x1D7CB, GeneralCategory::Ll),
    (0x1D7CE, 0x1D7FF, GeneralCategory::Nd), // 50 chars
    (0x1D800, 0x1D9FF, GeneralCategory::So), // 512 chars
    (0x1DA00, 0x1DA36, GeneralCategory::Mn), // 55 chars
    (0x1DA37, 0x1DA3A, GeneralCategory::So), // 4 chars
    (0x1DA3B, 0x1DA6C, GeneralCategory::Mn), // 50 chars
    (0x1DA6D, 0x1DA74, GeneralCategory::So), // 8 chars
    (0x1DA75, 0x1DA75, GeneralCategory::Mn),
    (0x1DA76, 0x1DA83, GeneralCategory::So), // 14 chars
    (0x1DA84, 0x1DA84, GeneralCategory::Mn),
    (0x1DA85, 0x1DA86, GeneralCategory::So), // 2 chars
    (0x1DA87, 0x1DA8B, GeneralCategory::Po), // 5 chars
    (0x1DA9B, 0x1DA9F, GeneralCategory::Mn), // 5 chars
    (0x1DAA1, 0x1DAAF, GeneralCategory::Mn), // 15 chars
    (0x1DF00, 0x1DF09, GeneralCategory::Ll), // 10 chars
    (0x1DF0A, 0x1DF0A, GeneralCategory::Lo),
    (0x1DF0B, 0x1DF1E, GeneralCategory::Ll), // 20 chars
    (0x1DF25, 0x1DF2A, GeneralCategory::Ll), // 6 chars
    (0x1E000, 0x1E006, GeneralCategory::Mn), // 7 chars
    (0x1E008, 0x1E018, GeneralCategory::Mn), // 17 chars
    (0x1E01B, 0x1E021, GeneralCategory::Mn), // 7 chars
    (0x1E023, 0x1E024, GeneralCategory::Mn), // 2 chars
    (0x1E026, 0x1E02A, GeneralCategory::Mn), // 5 chars
    (0x1E030, 0x1E06D, GeneralCategory::Lm), // 62 chars
    (0x1E08F, 0x1E08F, GeneralCategory::Mn),
    (0x1E100, 0x1E12C, GeneralCategory::Lo), // 45 chars
    (0x1E130, 0x1E136, GeneralCategory::Mn), // 7 chars
    (0x1E137, 0x1E13D, GeneralCategory::Lm), // 7 chars
    (0x1E140, 0x1E149, GeneralCategory::Nd), // 10 chars
    (0x1E14E, 0x1E14E, GeneralCategory::Lo),
    (0x1E14F, 0x1E14F, GeneralCategory::So),
    (0x1E290, 0x1E2AD, GeneralCategory::Lo), // 30 chars
    (0x1E2AE, 0x1E2AE, GeneralCategory::Mn),
    (0x1E2C0, 0x1E2EB, GeneralCategory::Lo), // 44 chars
    (0x1E2EC, 0x1E2EF, GeneralCategory::Mn), // 4 chars
    (0x1E2F0, 0x1E2F9, GeneralCategory::Nd), // 10 chars
    (0x1E2FF, 0x1E2FF, GeneralCategory::Sc),
    (0x1E4D0, 0x1E4EA, GeneralCategory::Lo), // 27 chars
    (0x1E4EB, 0x1E4EB, GeneralCategory::Lm),
    (0x1E4EC, 0x1E4EF, GeneralCategory::Mn), // 4 chars
    (0x1E4F0, 0x1E4F9, GeneralCategory::Nd), // 10 chars
    (0x1E5D0, 0x1E5ED, GeneralCategory::Lo), // 30 chars
    (0x1E5EE, 0x1E5EF, GeneralCategory::Mn), // 2 chars
    (0x1E5F0, 0x1E5F0, GeneralCategory::Lo),
    (0x1E5F1, 0x1E5FA, GeneralCategory::Nd), // 10 chars
    (0x1E5FF, 0x1E5FF, GeneralCategory::Po),
    (0x1E6C0, 0x1E6DE, GeneralCategory::Lo), // 31 chars
    (0x1E6E0, 0x1E6E2, GeneralCategory::Lo), // 3 chars
    (0x1E6E3, 0x1E6E3, GeneralCategory::Mn),
    (0x1E6E4, 0x1E6E5, GeneralCategory::Lo), // 2 chars
    (0x1E6E6, 0x1E6E6, GeneralCategory::Mn),
    (0x1E6E7, 0x1E6ED, GeneralCategory::Lo), // 7 chars
    (0x1E6EE, 0x1E6EF, GeneralCategory::Mn), // 2 chars
    (0x1E6F0, 0x1E6F4, GeneralCategory::Lo), // 5 chars
    (0x1E6F5, 0x1E6F5, GeneralCategory::Mn),
    (0x1E6FE, 0x1E6FE, GeneralCategory::Lo),
    (0x1E6FF, 0x1E6FF, GeneralCategory::Lm),
    (0x1E7E0, 0x1E7E6, GeneralCategory::Lo), // 7 chars
    (0x1E7E8, 0x1E7EB, GeneralCategory::Lo), // 4 chars
    (0x1E7ED, 0x1E7EE, GeneralCategory::Lo), // 2 chars
    (0x1E7F0, 0x1E7FE, GeneralCategory::Lo), // 15 chars
    (0x1E800, 0x1E8C4, GeneralCategory::Lo), // 197 chars
    (0x1E8C7, 0x1E8CF, GeneralCategory::No), // 9 chars
    (0x1E8D0, 0x1E8D6, GeneralCategory::Mn), // 7 chars
    (0x1E900, 0x1E921, GeneralCategory::Lu), // 34 chars
    (0x1E922, 0x1E943, GeneralCategory::Ll), // 34 chars
    (0x1E944, 0x1E94A, GeneralCategory::Mn), // 7 chars
    (0x1E94B, 0x1E94B, GeneralCategory::Lm),
    (0x1E950, 0x1E959, GeneralCategory::Nd), // 10 chars
    (0x1E95E, 0x1E95F, GeneralCategory::Po), // 2 chars
    (0x1EC71, 0x1ECAB, GeneralCategory::No), // 59 chars
    (0x1ECAC, 0x1ECAC, GeneralCategory::So),
    (0x1ECAD, 0x1ECAF, GeneralCategory::No), // 3 chars
    (0x1ECB0, 0x1ECB0, GeneralCategory::Sc),
    (0x1ECB1, 0x1ECB4, GeneralCategory::No), // 4 chars
    (0x1ED01, 0x1ED2D, GeneralCategory::No), // 45 chars
    (0x1ED2E, 0x1ED2E, GeneralCategory::So),
    (0x1ED2F, 0x1ED3D, GeneralCategory::No), // 15 chars
    (0x1EE00, 0x1EE03, GeneralCategory::Lo), // 4 chars
    (0x1EE05, 0x1EE1F, GeneralCategory::Lo), // 27 chars
    (0x1EE21, 0x1EE22, GeneralCategory::Lo), // 2 chars
    (0x1EE24, 0x1EE24, GeneralCategory::Lo),
    (0x1EE27, 0x1EE27, GeneralCategory::Lo),
    (0x1EE29, 0x1EE32, GeneralCategory::Lo), // 10 chars
    (0x1EE34, 0x1EE37, GeneralCategory::Lo), // 4 chars
    (0x1EE39, 0x1EE39, GeneralCategory::Lo),
    (0x1EE3B, 0x1EE3B, GeneralCategory::Lo),
    (0x1EE42, 0x1EE42, GeneralCategory::Lo),
    (0x1EE47, 0x1EE47, GeneralCategory::Lo),
    (0x1EE49, 0x1EE49, GeneralCategory::Lo),
    (0x1EE4B, 0x1EE4B, GeneralCategory::Lo),
    (0x1EE4D, 0x1EE4F, GeneralCategory::Lo), // 3 chars
    (0x1EE51, 0x1EE52, GeneralCategory::Lo), // 2 chars
    (0x1EE54, 0x1EE54, GeneralCategory::Lo),
    (0x1EE57, 0x1EE57, GeneralCategory::Lo),
    (0x1EE59, 0x1EE59, GeneralCategory::Lo),
    (0x1EE5B, 0x1EE5B, GeneralCategory::Lo),
    (0x1EE5D, 0x1EE5D, GeneralCategory::Lo),
    (0x1EE5F, 0x1EE5F, GeneralCategory::Lo),
    (0x1EE61, 0x1EE62, GeneralCategory::Lo), // 2 chars
    (0x1EE64, 0x1EE64, GeneralCategory::Lo),
    (0x1EE67, 0x1EE6A, GeneralCategory::Lo), // 4 chars
    (0x1EE6C, 0x1EE72, GeneralCategory::Lo), // 7 chars
    (0x1EE74, 0x1EE77, GeneralCategory::Lo), // 4 chars
    (0x1EE79, 0x1EE7C, GeneralCategory::Lo), // 4 chars
    (0x1EE7E, 0x1EE7E, GeneralCategory::Lo),
    (0x1EE80, 0x1EE89, GeneralCategory::Lo), // 10 chars
    (0x1EE8B, 0x1EE9B, GeneralCategory::Lo), // 17 chars
    (0x1EEA1, 0x1EEA3, GeneralCategory::Lo), // 3 chars
    (0x1EEA5, 0x1EEA9, GeneralCategory::Lo), // 5 chars
    (0x1EEAB, 0x1EEBB, GeneralCategory::Lo), // 17 chars
    (0x1EEF0, 0x1EEF1, GeneralCategory::Sm), // 2 chars
    (0x1F000, 0x1F02B, GeneralCategory::So), // 44 chars
    (0x1F030, 0x1F093, GeneralCategory::So), // 100 chars
    (0x1F0A0, 0x1F0AE, GeneralCategory::So), // 15 chars
    (0x1F0B1, 0x1F0BF, GeneralCategory::So), // 15 chars
    (0x1F0C1, 0x1F0CF, GeneralCategory::So), // 15 chars
    (0x1F0D1, 0x1F0F5, GeneralCategory::So), // 37 chars
    (0x1F100, 0x1F10C, GeneralCategory::No), // 13 chars
    (0x1F10D, 0x1F1AD, GeneralCategory::So), // 161 chars
    (0x1F1E6, 0x1F202, GeneralCategory::So), // 29 chars
    (0x1F210, 0x1F23B, GeneralCategory::So), // 44 chars
    (0x1F240, 0x1F248, GeneralCategory::So), // 9 chars
    (0x1F250, 0x1F251, GeneralCategory::So), // 2 chars
    (0x1F260, 0x1F265, GeneralCategory::So), // 6 chars
    (0x1F300, 0x1F3FA, GeneralCategory::So), // 251 chars
    (0x1F3FB, 0x1F3FF, GeneralCategory::Sk), // 5 chars
    (0x1F400, 0x1F6D8, GeneralCategory::So), // 729 chars
    (0x1F6DC, 0x1F6EC, GeneralCategory::So), // 17 chars
    (0x1F6F0, 0x1F6FC, GeneralCategory::So), // 13 chars
    (0x1F700, 0x1F7D9, GeneralCategory::So), // 218 chars
    (0x1F7E0, 0x1F7EB, GeneralCategory::So), // 12 chars
    (0x1F7F0, 0x1F7F0, GeneralCategory::So),
    (0x1F800, 0x1F80B, GeneralCategory::So), // 12 chars
    (0x1F810, 0x1F847, GeneralCategory::So), // 56 chars
    (0x1F850, 0x1F859, GeneralCategory::So), // 10 chars
    (0x1F860, 0x1F887, GeneralCategory::So), // 40 chars
    (0x1F890, 0x1F8AD, GeneralCategory::So), // 30 chars
    (0x1F8B0, 0x1F8BB, GeneralCategory::So), // 12 chars
    (0x1F8C0, 0x1F8C1, GeneralCategory::So), // 2 chars
    (0x1F8D0, 0x1F8D8, GeneralCategory::Sm), // 9 chars
    (0x1F900, 0x1FA57, GeneralCategory::So), // 344 chars
    (0x1FA60, 0x1FA6D, GeneralCategory::So), // 14 chars
    (0x1FA70, 0x1FA7C, GeneralCategory::So), // 13 chars
    (0x1FA80, 0x1FA8A, GeneralCategory::So), // 11 chars
    (0x1FA8E, 0x1FAC6, GeneralCategory::So), // 57 chars
    (0x1FAC8, 0x1FAC8, GeneralCategory::So),
    (0x1FACD, 0x1FADC, GeneralCategory::So), // 16 chars
    (0x1FADF, 0x1FAEA, GeneralCategory::So), // 12 chars
    (0x1FAEF, 0x1FAF8, GeneralCategory::So), // 10 chars
    (0x1FB00, 0x1FB92, GeneralCategory::So), // 147 chars
    (0x1FB94, 0x1FBEF, GeneralCategory::So), // 92 chars
    (0x1FBF0, 0x1FBF9, GeneralCategory::Nd), // 10 chars
    (0x1FBFA, 0x1FBFA, GeneralCategory::So),
    (0x20000, 0x2A6DF, GeneralCategory::Lo), // 42720 chars
    (0x2A700, 0x2B81D, GeneralCategory::Lo), // 4382 chars
    (0x2B820, 0x2CEAD, GeneralCategory::Lo), // 5774 chars
    (0x2CEB0, 0x2EBE0, GeneralCategory::Lo), // 7473 chars
    (0x2EBF0, 0x2EE5D, GeneralCategory::Lo), // 622 chars
    (0x2F800, 0x2FA1D, GeneralCategory::Lo), // 542 chars
    (0x30000, 0x3134A, GeneralCategory::Lo), // 4939 chars
    (0x31350, 0x33479, GeneralCategory::Lo), // 8490 chars
    (0xE0001, 0xE0001, GeneralCategory::Cf),
    (0xE0020, 0xE007F, GeneralCategory::Cf),   // 96 chars
    (0xE0100, 0xE01EF, GeneralCategory::Mn),   // 240 chars
    (0xF0000, 0xFFFFD, GeneralCategory::Co),   // 65534 chars
    (0x100000, 0x10FFFD, GeneralCategory::Co), // 65534 chars
];

/// Бинарный поиск категории по кодпоинту — O(log n)
pub fn find_category(cp: u32) -> GeneralCategory {
    let mut lo = 0usize;
    let mut hi = 3409;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let (start, end, cat) = CATEGORY_RANGES[mid];
        if cp < start {
            hi = mid;
        } else if cp > end {
            lo = mid + 1;
        } else {
            return cat;
        }
    }
    GeneralCategory::Cn // Unassigned
}

/// Общее количество определённых символов
pub fn total_defined_chars() -> usize {
    299382
}

/// Количество сжатых диапазонов
pub fn category_range_count() -> usize {
    3409
}
