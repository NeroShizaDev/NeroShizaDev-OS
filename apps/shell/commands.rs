use super::runtime;
use super::state::{
    self, shell_buffer_eq_ascii, shell_buffer_snapshot, shell_buffer_to_ascii_lower,
};
use crate::print;
use crate::{
    apps, kernel_messages, locale, unicode, unicode_blocks, unicode_categories, unicode_scripts,
    validator, voodoo_engine,
};

const NHS_DEMO_PACKAGE: &[u8] = include_bytes!("../installer/nhsapps/demo.nhs");
const NHS_HELLO_PACKAGE: &[u8] = include_bytes!("../installer/nhsapps/hello.nhs");

const COMMAND_ALIAS_LEN: usize = 16;
type CommandAliasVector = [u32; COMMAND_ALIAS_LEN];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShellIntent {
    Unknown,
    Exit,
    Help,
    Clear,
    Status,
    Reboot,
    LocaleCycle,
    LocaleRu,
    LocaleEn,
    LocaleAr,
    ModeLore,
    ModeTech,
    Apps,
    WhoAmI,
    Manifest,
    Entropy,
    Rng,
    Voodoo,
}

struct ShellAlias {
    pattern: CommandAliasVector,
    intent: ShellIntent,
    canonical: &'static str,
}

const fn alias_vector(codepoints: &[u32]) -> CommandAliasVector {
    let mut vector = [0u32; COMMAND_ALIAS_LEN];
    let mut index = 0;
    while index < codepoints.len() && index < COMMAND_ALIAS_LEN {
        vector[index] = codepoints[index];
        index += 1;
    }
    vector
}

const SHELL_ALIASES: &[ShellAlias] = &[
    ShellAlias {
        pattern: alias_vector(&[0x0432, 0x044B, 0x0445, 0x043E, 0x0434]),
        intent: ShellIntent::Exit,
        canonical: "exit",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0441, 0x0432, 0x0430, 0x043B, 0x0438]),
        intent: ShellIntent::Exit,
        canonical: "exit",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0065, 0x0078, 0x0069, 0x0074]),
        intent: ShellIntent::Exit,
        canonical: "exit",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0071, 0x0075, 0x0069, 0x0074]),
        intent: ShellIntent::Exit,
        canonical: "exit",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0063, 0x0064, 0x0066, 0x006B, 0x0062]),
        intent: ShellIntent::Exit,
        canonical: "exit",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0064, 0x0073, 0x005B, 0x006A, 0x006C]),
        intent: ShellIntent::Exit,
        canonical: "exit",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0627, 0x062E, 0x0631, 0x062C]),
        intent: ShellIntent::Exit,
        canonical: "exit",
    },
    ShellAlias {
        pattern: alias_vector(&[0x10D2, 0x10D0, 0x10E1, 0x10D5, 0x10DA, 0x10D0]),
        intent: ShellIntent::Exit,
        canonical: "exit",
    },
    ShellAlias {
        pattern: alias_vector(&[0x51FA, 0x53E3]),
        intent: ShellIntent::Exit,
        canonical: "exit",
    },
    ShellAlias {
        pattern: alias_vector(&[0x7D42, 0x4E86]),
        intent: ShellIntent::Exit,
        canonical: "exit",
    },
    ShellAlias {
        pattern: alias_vector(&[0x062E, 0x0631, 0x0648, 0x062C]),
        intent: ShellIntent::Exit,
        canonical: "exit",
    },
    ShellAlias {
        pattern: alias_vector(&[0x043F, 0x043E, 0x043C, 0x043E, 0x0449, 0x044C]),
        intent: ShellIntent::Help,
        canonical: "help",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0068, 0x0065, 0x006C, 0x0070]),
        intent: ShellIntent::Help,
        canonical: "help",
    },
    ShellAlias {
        pattern: alias_vector(&[0x003F]),
        intent: ShellIntent::Help,
        canonical: "help",
    },
    ShellAlias {
        pattern: alias_vector(&[0x043F, 0x043E, 0x043C, 0x043E, 0x0433, 0x0438]),
        intent: ShellIntent::Help,
        canonical: "help",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0645, 0x0633, 0x0627, 0x0639, 0x062F, 0x0629]),
        intent: ShellIntent::Help,
        canonical: "help",
    },
    ShellAlias {
        pattern: alias_vector(&[
            0x043E, 0x0447, 0x0438, 0x0441, 0x0442, 0x0438, 0x0442, 0x044C,
        ]),
        intent: ShellIntent::Clear,
        canonical: "clear",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0063, 0x006C, 0x0073]),
        intent: ShellIntent::Clear,
        canonical: "clear",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0063, 0x006C, 0x0065, 0x0061, 0x0072]),
        intent: ShellIntent::Clear,
        canonical: "clear",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0645, 0x0633, 0x062D]),
        intent: ShellIntent::Clear,
        canonical: "clear",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0441, 0x0442, 0x0430, 0x0442, 0x0443, 0x0441]),
        intent: ShellIntent::Status,
        canonical: "status",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0073, 0x0074, 0x0061, 0x0074, 0x0075, 0x0073]),
        intent: ShellIntent::Status,
        canonical: "status",
    },
    ShellAlias {
        pattern: alias_vector(&[0x062D, 0x0627, 0x0644, 0x0629]),
        intent: ShellIntent::Status,
        canonical: "status",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0440, 0x0435, 0x0431, 0x0443, 0x0442]),
        intent: ShellIntent::Reboot,
        canonical: "reboot",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0072, 0x0065, 0x0062, 0x006F, 0x006F, 0x0074]),
        intent: ShellIntent::Reboot,
        canonical: "reboot",
    },
    ShellAlias {
        pattern: alias_vector(&[
            0x043F, 0x0435, 0x0440, 0x0435, 0x0437, 0x0430, 0x0433, 0x0440, 0x0443, 0x0437, 0x043A,
            0x0430,
        ]),
        intent: ShellIntent::Reboot,
        canonical: "reboot",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0627, 0x0639, 0x0627, 0x062F, 0x0629]),
        intent: ShellIntent::Reboot,
        canonical: "reboot",
    },
    ShellAlias {
        pattern: alias_vector(&[0x006C, 0x006F, 0x0063, 0x0061, 0x006C, 0x0065]),
        intent: ShellIntent::LocaleCycle,
        canonical: "locale",
    },
    ShellAlias {
        pattern: alias_vector(&[0x043B, 0x043E, 0x043A, 0x0430, 0x043B, 0x044C]),
        intent: ShellIntent::LocaleCycle,
        canonical: "locale",
    },
    ShellAlias {
        pattern: alias_vector(&[0x044F, 0x0437, 0x044B, 0x043A]),
        intent: ShellIntent::LocaleCycle,
        canonical: "locale",
    },
    ShellAlias {
        pattern: alias_vector(&[0x006C, 0x0061, 0x006E, 0x0067]),
        intent: ShellIntent::LocaleCycle,
        canonical: "locale",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0072, 0x0075]),
        intent: ShellIntent::LocaleRu,
        canonical: "ru",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0440, 0x0443, 0x0441]),
        intent: ShellIntent::LocaleRu,
        canonical: "ru",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0065, 0x006E]),
        intent: ShellIntent::LocaleEn,
        canonical: "en",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0065, 0x006E, 0x0067]),
        intent: ShellIntent::LocaleEn,
        canonical: "en",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0061, 0x0072]),
        intent: ShellIntent::LocaleAr,
        canonical: "ar",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0061, 0x0072, 0x0061, 0x0062]),
        intent: ShellIntent::LocaleAr,
        canonical: "ar",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0639, 0x0631, 0x0628, 0x064A]),
        intent: ShellIntent::LocaleAr,
        canonical: "ar",
    },
    ShellAlias {
        pattern: alias_vector(&[0x006C, 0x006F, 0x0072, 0x0065]),
        intent: ShellIntent::ModeLore,
        canonical: "lore",
    },
    ShellAlias {
        pattern: alias_vector(&[0x043B, 0x043E, 0x0440]),
        intent: ShellIntent::ModeLore,
        canonical: "lore",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0073, 0x0068, 0x0069, 0x007A, 0x0061]),
        intent: ShellIntent::ModeLore,
        canonical: "lore",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0074, 0x0065, 0x0063, 0x0068]),
        intent: ShellIntent::ModeTech,
        canonical: "tech",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0442, 0x0435, 0x0445]),
        intent: ShellIntent::ModeTech,
        canonical: "tech",
    },
    ShellAlias {
        pattern: alias_vector(&[
            0x0074, 0x0065, 0x0063, 0x0068, 0x006E, 0x0069, 0x0063, 0x0061, 0x006C,
        ]),
        intent: ShellIntent::ModeTech,
        canonical: "tech",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0061, 0x0070, 0x0070, 0x0073]),
        intent: ShellIntent::Apps,
        canonical: "apps",
    },
    ShellAlias {
        pattern: alias_vector(&[0x006D, 0x0065, 0x006E, 0x0075]),
        intent: ShellIntent::Apps,
        canonical: "apps",
    },
    ShellAlias {
        pattern: alias_vector(&[
            0x006C, 0x0061, 0x0075, 0x006E, 0x0063, 0x0068, 0x0065, 0x0072,
        ]),
        intent: ShellIntent::Apps,
        canonical: "apps",
    },
    ShellAlias {
        pattern: alias_vector(&[0x043F, 0x0440, 0x043E, 0x0433, 0x0438]),
        intent: ShellIntent::Apps,
        canonical: "apps",
    },
    ShellAlias {
        pattern: alias_vector(&[0x043C, 0x0435, 0x043D, 0x044E]),
        intent: ShellIntent::Apps,
        canonical: "apps",
    },
    ShellAlias {
        pattern: alias_vector(&[0x062A, 0x0637, 0x0628, 0x064A, 0x0642, 0x0627, 0x062A]),
        intent: ShellIntent::Apps,
        canonical: "apps",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0077, 0x0068, 0x006F, 0x0061, 0x006D, 0x0069]),
        intent: ShellIntent::WhoAmI,
        canonical: "whoami",
    },
    ShellAlias {
        pattern: alias_vector(&[0x043A, 0x0442, 0x043E]),
        intent: ShellIntent::WhoAmI,
        canonical: "whoami",
    },
    ShellAlias {
        pattern: alias_vector(&[
            0x043B, 0x0438, 0x0447, 0x043D, 0x043E, 0x0441, 0x0442, 0x044C,
        ]),
        intent: ShellIntent::WhoAmI,
        canonical: "whoami",
    },
    ShellAlias {
        pattern: alias_vector(&[
            0x006D, 0x0061, 0x006E, 0x0069, 0x0066, 0x0065, 0x0073, 0x0074,
        ]),
        intent: ShellIntent::Manifest,
        canonical: "manifest",
    },
    ShellAlias {
        pattern: alias_vector(&[
            0x043C, 0x0430, 0x043D, 0x0438, 0x0444, 0x0435, 0x0441, 0x0442,
        ]),
        intent: ShellIntent::Manifest,
        canonical: "manifest",
    },
    ShellAlias {
        pattern: alias_vector(&[0x006E, 0x0065, 0x0072, 0x006F]),
        intent: ShellIntent::Manifest,
        canonical: "manifest",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0073, 0x0068, 0x0069, 0x007A, 0x0061]),
        intent: ShellIntent::Manifest,
        canonical: "manifest",
    },
    ShellAlias {
        pattern: alias_vector(&[0x043D, 0x0435, 0x0439, 0x0440, 0x043E]),
        intent: ShellIntent::Manifest,
        canonical: "manifest",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0448, 0x0438, 0x0437, 0x0430]),
        intent: ShellIntent::Manifest,
        canonical: "manifest",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0065, 0x006E, 0x0074, 0x0072, 0x006F, 0x0070, 0x0079]),
        intent: ShellIntent::Entropy,
        canonical: "entropy",
    },
    ShellAlias {
        pattern: alias_vector(&[
            0x044D, 0x043D, 0x0442, 0x0440, 0x043E, 0x043F, 0x0438, 0x044F,
        ]),
        intent: ShellIntent::Entropy,
        canonical: "entropy",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0448, 0x0435, 0x043D, 0x043D, 0x043E, 0x043D]),
        intent: ShellIntent::Entropy,
        canonical: "entropy",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0073, 0x0068, 0x0061, 0x006E, 0x006E, 0x006F, 0x006E]),
        intent: ShellIntent::Entropy,
        canonical: "entropy",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0072, 0x006E, 0x0067]),
        intent: ShellIntent::Rng,
        canonical: "rng",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0072, 0x0061, 0x006E, 0x0064]),
        intent: ShellIntent::Rng,
        canonical: "rng",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0072, 0x0061, 0x006E, 0x0064, 0x006F, 0x006D]),
        intent: ShellIntent::Rng,
        canonical: "rng",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0440, 0x0430, 0x043D, 0x0434, 0x043E, 0x043C]),
        intent: ShellIntent::Rng,
        canonical: "rng",
    },
    ShellAlias {
        pattern: alias_vector(&[0x043A, 0x0443, 0x0431, 0x0438, 0x043A]),
        intent: ShellIntent::Rng,
        canonical: "rng",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0076, 0x006F, 0x006F, 0x0064, 0x006F, 0x006F]),
        intent: ShellIntent::Voodoo,
        canonical: "voodoo",
    },
    ShellAlias {
        pattern: alias_vector(&[0x0432, 0x0443, 0x0434, 0x0443]),
        intent: ShellIntent::Voodoo,
        canonical: "voodoo",
    },
    ShellAlias {
        pattern: alias_vector(&[
            0x0430, 0x043A, 0x0438, 0x043D, 0x0430, 0x0442, 0x043E, 0x0440,
        ]),
        intent: ShellIntent::Voodoo,
        canonical: "voodoo",
    },
    ShellAlias {
        pattern: alias_vector(&[0x006F, 0x0072, 0x0061, 0x0063, 0x006C, 0x0065]),
        intent: ShellIntent::Voodoo,
        canonical: "voodoo",
    },
    ShellAlias {
        pattern: alias_vector(&[0x043E, 0x0440, 0x0430, 0x043A, 0x0443, 0x043B]),
        intent: ShellIntent::Voodoo,
        canonical: "voodoo",
    },
    ShellAlias {
        pattern: alias_vector(&[0x74, 0x69, 0x78, 0x65]),
        intent: ShellIntent::Exit,
        canonical: "exit",
    },
    ShellAlias {
        pattern: alias_vector(&[0x74, 0x69, 0x75, 0x71]),
        intent: ShellIntent::Exit,
        canonical: "exit",
    },
    ShellAlias {
        pattern: alias_vector(&[0x70, 0x6C, 0x65, 0x68]),
        intent: ShellIntent::Help,
        canonical: "help",
    },
    ShellAlias {
        pattern: alias_vector(&[0x72, 0x61, 0x65, 0x6C, 0x63]),
        intent: ShellIntent::Clear,
        canonical: "clear",
    },
    ShellAlias {
        pattern: alias_vector(&[0x73, 0x75, 0x74, 0x61, 0x74, 0x73]),
        intent: ShellIntent::Status,
        canonical: "status",
    },
    ShellAlias {
        pattern: alias_vector(&[0x74, 0x6F, 0x6F, 0x62, 0x65, 0x72]),
        intent: ShellIntent::Reboot,
        canonical: "reboot",
    },
    ShellAlias {
        pattern: alias_vector(&[0x65, 0x6C, 0x61, 0x63, 0x6F, 0x6C]),
        intent: ShellIntent::LocaleCycle,
        canonical: "locale",
    },
    ShellAlias {
        pattern: alias_vector(&[0x67, 0x6E, 0x61, 0x6C]),
        intent: ShellIntent::LocaleCycle,
        canonical: "locale",
    },
    ShellAlias {
        pattern: alias_vector(&[0x75, 0x72]),
        intent: ShellIntent::LocaleRu,
        canonical: "ru",
    },
    ShellAlias {
        pattern: alias_vector(&[0x6E, 0x65]),
        intent: ShellIntent::LocaleEn,
        canonical: "en",
    },
    ShellAlias {
        pattern: alias_vector(&[0x67, 0x6E, 0x65]),
        intent: ShellIntent::LocaleEn,
        canonical: "en",
    },
    ShellAlias {
        pattern: alias_vector(&[0x72, 0x61]),
        intent: ShellIntent::LocaleAr,
        canonical: "ar",
    },
    ShellAlias {
        pattern: alias_vector(&[0x62, 0x61, 0x72, 0x61]),
        intent: ShellIntent::LocaleAr,
        canonical: "ar",
    },
    ShellAlias {
        pattern: alias_vector(&[0x65, 0x72, 0x6F, 0x6C]),
        intent: ShellIntent::ModeLore,
        canonical: "lore",
    },
    ShellAlias {
        pattern: alias_vector(&[0x68, 0x63, 0x65, 0x74]),
        intent: ShellIntent::ModeTech,
        canonical: "tech",
    },
    ShellAlias {
        pattern: alias_vector(&[0x6C, 0x61, 0x63, 0x69, 0x6E, 0x68, 0x63, 0x65, 0x74]),
        intent: ShellIntent::ModeTech,
        canonical: "tech",
    },
    ShellAlias {
        pattern: alias_vector(&[0x73, 0x61, 0x70, 0x70]),
        intent: ShellIntent::Apps,
        canonical: "apps",
    },
    ShellAlias {
        pattern: alias_vector(&[0x75, 0x6E, 0x65, 0x6D]),
        intent: ShellIntent::Apps,
        canonical: "apps",
    },
    ShellAlias {
        pattern: alias_vector(&[0x72, 0x65, 0x68, 0x63, 0x6E, 0x75, 0x61, 0x6C]),
        intent: ShellIntent::Apps,
        canonical: "apps",
    },
    ShellAlias {
        pattern: alias_vector(&[0x69, 0x6D, 0x61, 0x6F, 0x68, 0x77]),
        intent: ShellIntent::WhoAmI,
        canonical: "whoami",
    },
    ShellAlias {
        pattern: alias_vector(&[0x74, 0x73, 0x65, 0x66, 0x69, 0x6E, 0x61, 0x6D]),
        intent: ShellIntent::Manifest,
        canonical: "manifest",
    },
    ShellAlias {
        pattern: alias_vector(&[0x6F, 0x72, 0x65, 0x6E]),
        intent: ShellIntent::Manifest,
        canonical: "manifest",
    },
    ShellAlias {
        pattern: alias_vector(&[0x79, 0x70, 0x6F, 0x72, 0x74, 0x6E, 0x65]),
        intent: ShellIntent::Entropy,
        canonical: "entropy",
    },
    ShellAlias {
        pattern: alias_vector(&[0x6E, 0x6F, 0x6E, 0x6E, 0x61, 0x68, 0x73]),
        intent: ShellIntent::Entropy,
        canonical: "entropy",
    },
    ShellAlias {
        pattern: alias_vector(&[0x67, 0x6E, 0x72]),
        intent: ShellIntent::Rng,
        canonical: "rng",
    },
    ShellAlias {
        pattern: alias_vector(&[0x64, 0x6E, 0x61, 0x72]),
        intent: ShellIntent::Rng,
        canonical: "rng",
    },
    ShellAlias {
        pattern: alias_vector(&[0x6D, 0x6F, 0x64, 0x6E, 0x61, 0x72]),
        intent: ShellIntent::Rng,
        canonical: "rng",
    },
    ShellAlias {
        pattern: alias_vector(&[0x6F, 0x6F, 0x64, 0x6F, 0x6F, 0x76]),
        intent: ShellIntent::Voodoo,
        canonical: "voodoo",
    },
    ShellAlias {
        pattern: alias_vector(&[0x65, 0x6C, 0x63, 0x61, 0x72, 0x6F]),
        intent: ShellIntent::Voodoo,
        canonical: "voodoo",
    },
];

struct BuiltinCommand {
    name: &'static str,
    aliases: &'static [&'static str],
    category: &'static str,
    summary: &'static str,
    usage: &'static str,
    handler: fn() -> bool,
}

impl BuiltinCommand {
    fn matches(&self, token: &str) -> bool {
        self.name == token || self.aliases.iter().any(|alias| *alias == token)
    }
}

pub fn shell_dictionary_size() -> usize {
    SHELL_ALIASES.len()
}

pub fn shell_dictionary_bytes() -> usize {
    SHELL_ALIASES.len() * core::mem::size_of::<ShellAlias>()
}

fn lookup_shell_intent(input: &[u32]) -> ShellIntent {
    let mut vector = [0u32; COMMAND_ALIAS_LEN];
    let len = input.len().min(COMMAND_ALIAS_LEN);
    if len == 0 {
        return ShellIntent::Unknown;
    }

    let mut index = 0;
    while index < len {
        vector[index] = input[index];
        index += 1;
    }

    let first = vector[0];
    for alias in SHELL_ALIASES {
        if alias.pattern[0] == first && alias.pattern == vector {
            return alias.intent;
        }
    }

    ShellIntent::Unknown
}

fn shell_hamming_distance(left: &CommandAliasVector, right: &CommandAliasVector) -> u32 {
    let mut distance = 0u32;
    for index in 0..COMMAND_ALIAS_LEN {
        if left[index] != right[index] {
            distance += 1;
        }
        if left[index] == 0 && right[index] == 0 {
            break;
        }
    }
    distance
}

fn closest_shell_intent(input: &[u32]) -> Option<(&'static str, u32)> {
    const THRESHOLD: u32 = 4;

    let mut vector = [0u32; COMMAND_ALIAS_LEN];
    let len = input.len().min(COMMAND_ALIAS_LEN);
    let mut index = 0;
    while index < len {
        vector[index] = input[index];
        index += 1;
    }

    let mut best_name = "";
    let mut best_distance = u32::MAX;
    for alias in SHELL_ALIASES {
        let distance = shell_hamming_distance(&vector, &alias.pattern);
        if distance < best_distance {
            best_distance = distance;
            best_name = alias.canonical;
        }
    }

    if best_distance == 0 || best_distance > THRESHOLD {
        None
    } else {
        Some((best_name, best_distance))
    }
}

fn print_unknown_command_suggestion(name: &str, dist: u32) {
    match (locale::get_locale(), locale::get_mode()) {
        (kernel_messages::Locale::ArEg, kernel_messages::MessageMode::Technical) => {
            locale::print_localized_fmt(
                0x0E,
                format_args!("  هل تقصد: \"{}\"؟ (dist={})", name, dist),
            );
        }
        (kernel_messages::Locale::ArEg, kernel_messages::MessageMode::Lore) => {
            locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "  ربما كانت التعويذة المقصودة: \"{}\"؟ (dist={})",
                    name, dist
                ),
            );
        }
        (kernel_messages::Locale::EnUs, kernel_messages::MessageMode::Technical) => {
            locale::print_localized_fmt(
                0x0E,
                format_args!("  Did you mean: \"{}\"? (dist={})", name, dist),
            );
        }
        (kernel_messages::Locale::EnUs, kernel_messages::MessageMode::Lore) => {
            locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "  Perhaps the intended incantation was: \"{}\"? (dist={})",
                    name, dist
                ),
            );
        }
        (kernel_messages::Locale::RuRu, kernel_messages::MessageMode::Technical) => {
            locale::print_localized_fmt(
                0x0E,
                format_args!("  Возможно, имелось в виду: \"{}\"? (dist={})", name, dist),
            );
        }
        (kernel_messages::Locale::RuRu, kernel_messages::MessageMode::Lore) => {
            locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "  Возможно, требовалось это заклинание: \"{}\"? (dist={})",
                    name, dist
                ),
            );
        }
    }
}

fn print_whoami_result(name: &str) {
    match (locale::get_locale(), locale::get_mode()) {
        (kernel_messages::Locale::ArEg, kernel_messages::MessageMode::Technical) => {
            locale::print_localized_fmt(0x0B, format_args!("الجلسة الحالية: {}", name));
        }
        (kernel_messages::Locale::ArEg, kernel_messages::MessageMode::Lore) => {
            locale::print_localized_fmt(
                0x0B,
                format_args!("الوجه الذي تتكلم به المصفوفة الآن: {}", name),
            );
        }
        (kernel_messages::Locale::EnUs, kernel_messages::MessageMode::Technical) => {
            locale::print_localized_fmt(0x0B, format_args!("Current session resident: {}", name));
        }
        (kernel_messages::Locale::EnUs, kernel_messages::MessageMode::Lore) => {
            locale::print_localized_fmt(
                0x0B,
                format_args!("The matrix is currently speaking as: {}", name),
            );
        }
        (kernel_messages::Locale::RuRu, kernel_messages::MessageMode::Technical) => {
            locale::print_localized_fmt(0x0B, format_args!("Текущий резидент сеанса: {}", name));
        }
        (kernel_messages::Locale::RuRu, kernel_messages::MessageMode::Lore) => {
            locale::print_localized_fmt(0x0B, format_args!("На линии реальности сейчас: {}", name));
        }
    }
}

fn print_manifest_lines(lines: &[&str]) {
    for (index, line) in lines.iter().enumerate() {
        let color = match index {
            0 | 1 | 2 => 0x07,
            3 | 4 => 0x09,
            5 | 6 => 0x0D,
            7 => 0x07,
            _ => 0x08,
        };
        locale::print_localized_line(line, color);
    }
}

fn print_manifest_output() {
    match (locale::get_locale(), locale::get_mode()) {
        (kernel_messages::Locale::ArEg, kernel_messages::MessageMode::Technical) => {
            print_manifest_lines(&[
                "╔══════════════════════════════════════╗",
                "║      MANIFEST OF NERO & SHIZA       ║",
                "╚══════════════════════════════════════╝",
                "NERO: x87 FPU strict mathematics.",
                "      64-bit logic. Hallucination control.",
                "SHIZA: unrestricted creative expansion.",
                "       Psychotown. Hieroglyphs. Chaos.",
                "NeroShiza Records. Opening gateways.",
                "Jurisdiction: I.B.I.P., Psychotown.",
            ])
        }
        (kernel_messages::Locale::ArEg, kernel_messages::MessageMode::Lore) => {
            print_manifest_lines(&[
                "╔══════════════════════════════════════╗",
                "║      MANIFEST OF NERO & SHIZA       ║",
                "╚══════════════════════════════════════╝",
                "NERO: the cold bone-logic of x87.",
                "      The hand that keeps chaos measurable.",
                "SHIZA: the unleashed field of creation.",
                "       Psychotown. Glyphs. Sacred noise.",
                "NeroShiza Records. Portals are opening.",
                "Jurisdiction: I.B.I.P., Psychotown.",
            ])
        }
        (kernel_messages::Locale::EnUs, kernel_messages::MessageMode::Technical) => {
            print_manifest_lines(&[
                "╔══════════════════════════════════════╗",
                "║      MANIFEST OF NERO & SHIZA       ║",
                "╚══════════════════════════════════════╝",
                "NERO: strict x87 FPU mathematics.",
                "      64-bit logic. Hallucination control.",
                "SHIZA: unrestricted creative freedom.",
                "       Psychotown. Hieroglyphs. Chaos.",
                "NeroShiza Records. Opening portals.",
                "Jurisdiction: I.B.I.P., Psychotown.",
            ])
        }
        (kernel_messages::Locale::EnUs, kernel_messages::MessageMode::Lore) => {
            print_manifest_lines(&[
                "╔══════════════════════════════════════╗",
                "║      MANIFEST OF NERO & SHIZA       ║",
                "╚══════════════════════════════════════╝",
                "NERO: the hard mathematics of x87.",
                "      The 64-bit spine that rejects delusion.",
                "SHIZA: total creative release.",
                "       Psychotown. Glyphs. Chaos.",
                "NeroShiza Records. The portals are opening.",
                "Jurisdiction: I.B.I.P., Psychotown.",
            ])
        }
        (kernel_messages::Locale::RuRu, kernel_messages::MessageMode::Technical) => {
            print_manifest_lines(&[
                "╔══════════════════════════════════════╗",
                "║     МАНИФЕСТ NERO & SHIZA            ║",
                "╚══════════════════════════════════════╝",
                "NERO: строгая математика x87 FPU.",
                "      64-битная логика. Контроль галлюцинаций.",
                "SHIZA: неограниченная творческая свобода.",
                "       Психотаун. Иероглифы. Хаос.",
                "NeroShiza Records. Открытие порталов.",
                "Юрисдикция: И.Б.И.П., Психотаун.",
            ])
        }
        (kernel_messages::Locale::RuRu, kernel_messages::MessageMode::Lore) => {
            print_manifest_lines(&[
                "╔══════════════════════════════════════╗",
                "║     МАНИФЕСТ NERO & SHIZA            ║",
                "╚══════════════════════════════════════╝",
                "NERO: строгая математика x87 FPU.",
                "      64-битная ось, которая душит бред.",
                "SHIZA: абсолютная творческая свобода.",
                "       Психотаун. Иероглифы. Хаос.",
                "NeroShiza Records. Открываем порталы.",
                "Юрисдикция: И.Б.И.П., Психотаун.",
            ])
        }
    }
}

fn print_entropy_result(entropy: u64) {
    let whole = entropy / 1000;
    let frac = entropy % 1000;
    match (locale::get_locale(), locale::get_mode()) {
        (kernel_messages::Locale::ArEg, kernel_messages::MessageMode::Technical) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("Shannon entropy = {}.{:03} bits/symbol", whole, frac),
            );
        }
        (kernel_messages::Locale::ArEg, kernel_messages::MessageMode::Lore) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("Shannon chaos density = {}.{:03} bits/symbol", whole, frac),
            );
        }
        (kernel_messages::Locale::EnUs, kernel_messages::MessageMode::Technical) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("Shannon entropy = {}.{:03} bits/symbol", whole, frac),
            );
        }
        (kernel_messages::Locale::EnUs, kernel_messages::MessageMode::Lore) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("Shannon chaos density = {}.{:03} bits/symbol", whole, frac),
            );
        }
        (kernel_messages::Locale::RuRu, kernel_messages::MessageMode::Technical) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("Энтропия Шеннона = {}.{:03} бит/символ", whole, frac),
            );
        }
        (kernel_messages::Locale::RuRu, kernel_messages::MessageMode::Lore) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("Плотность хаоса Шеннона = {}.{:03} бит/символ", whole, frac),
            );
        }
    }
}

fn print_rng_result(random: u64, dice: u64) {
    match (locale::get_locale(), locale::get_mode()) {
        (kernel_messages::Locale::ArEg, kernel_messages::MessageMode::Technical) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("RDRAND sample: 0x{:016X} ({})", random, random),
            );
            locale::print_localized_fmt(0x0B, format_args!("d6 roll: {}", dice));
        }
        (kernel_messages::Locale::ArEg, kernel_messages::MessageMode::Lore) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("The matrix drew lot 0x{:016X} ({})", random, random),
            );
            locale::print_localized_fmt(0x0B, format_args!("The oracle rolled d6 = {}", dice));
        }
        (kernel_messages::Locale::EnUs, kernel_messages::MessageMode::Technical) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("RDRAND sample: 0x{:016X} ({})", random, random),
            );
            locale::print_localized_fmt(0x0B, format_args!("d6 roll: {}", dice));
        }
        (kernel_messages::Locale::EnUs, kernel_messages::MessageMode::Lore) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("The matrix drew lot 0x{:016X} ({})", random, random),
            );
            locale::print_localized_fmt(0x0B, format_args!("The oracle rolled d6 = {}", dice));
        }
        (kernel_messages::Locale::RuRu, kernel_messages::MessageMode::Technical) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("Выборка RDRAND: 0x{:016X} ({})", random, random),
            );
            locale::print_localized_fmt(0x0B, format_args!("Бросок d6: {}", dice));
        }
        (kernel_messages::Locale::RuRu, kernel_messages::MessageMode::Lore) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("Матрица вытянула жребий 0x{:016X} ({})", random, random),
            );
            locale::print_localized_fmt(0x0B, format_args!("Оракул бросил d6 = {}", dice));
        }
    }
}

const BUILTIN_COMMANDS: &[BuiltinCommand] = &[
    BuiltinCommand {
        name: "help",
        aliases: &["?"],
        category: "shell",
        summary: "show builtin commands",
        usage: "help [command]",
        handler: handle_help_command,
    },
    BuiltinCommand {
        name: "exit",
        aliases: &["quit"],
        category: "system",
        summary: "shutdown the system",
        usage: "exit",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "clear",
        aliases: &["cls"],
        category: "shell",
        summary: "clear the screen",
        usage: "clear",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "status",
        aliases: &[],
        category: "shell",
        summary: "show kernel and shell status",
        usage: "status",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "reboot",
        aliases: &[],
        category: "system",
        summary: "reboot the system",
        usage: "reboot",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "apps",
        aliases: &["menu", "launcher"],
        category: "apps",
        summary: "open the apps launcher",
        usage: "apps",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "locale",
        aliases: &["lang"],
        category: "locale",
        summary: "cycle locale or inspect locale commands",
        usage: "locale",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "ru",
        aliases: &[],
        category: "locale",
        summary: "set Russian locale",
        usage: "ru",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "en",
        aliases: &["eng"],
        category: "locale",
        summary: "set English locale",
        usage: "en",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "ar",
        aliases: &["arab"],
        category: "locale",
        summary: "set Arabic locale",
        usage: "ar",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "lore",
        aliases: &["shiza"],
        category: "locale",
        summary: "enable lore output mode",
        usage: "lore",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "tech",
        aliases: &["technical"],
        category: "locale",
        summary: "enable technical output mode",
        usage: "tech",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "whoami",
        aliases: &[],
        category: "ibip",
        summary: "print a random resident",
        usage: "whoami",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "manifest",
        aliases: &["nero"],
        category: "ibip",
        summary: "print NERO & SHIZA manifesto",
        usage: "manifest",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "entropy",
        aliases: &["shannon"],
        category: "ibip",
        summary: "show Shannon entropy for input",
        usage: "entropy",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "rng",
        aliases: &["rand", "random"],
        category: "ibip",
        summary: "print RDRAND sample and d6 roll",
        usage: "rng",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "voodoo",
        aliases: &["oracle"],
        category: "ibip",
        summary: "run the Bayesian oracle demo",
        usage: "voodoo",
        handler: handle_shell_intent_command,
    },
    BuiltinCommand {
        name: "irqdbg",
        aliases: &[],
        category: "debug",
        summary: "show irq guard status",
        usage: "irqdbg",
        handler: handle_irq_guard_debug_command,
    },
    BuiltinCommand {
        name: "irqguard",
        aliases: &[],
        category: "debug",
        summary: "control irq guard policy",
        usage: "irqguard on|off|reset",
        handler: handle_irq_guard_debug_command,
    },
    BuiltinCommand {
        name: "log",
        aliases: &["diag"],
        category: "debug",
        summary: "inspect and change runtime logging",
        usage: "log status|level ...|subsys ...|format ...|dedup ...|preset ...",
        handler: handle_log_debug_command,
    },
    BuiltinCommand {
        name: "install",
        aliases: &[],
        category: "apps",
        summary: "inspect built-in apps, NHS packages, and NSFS disk",
        usage: "install demo|hello|serial|list|disk|view <path>",
        handler: handle_nhs_shell_command,
    },
    BuiltinCommand {
        name: "nsfs",
        aliases: &["disk"],
        category: "apps",
        summary: "inspect the NSFS disk index or open a file from NSFS",
        usage: "nsfs [disk]|nsfs view <path>",
        handler: handle_nhs_shell_command,
    },
    BuiltinCommand {
        name: "view",
        aliases: &["cat", "read"],
        category: "apps",
        summary: "view a text file from the NSFS disk",
        usage: "view <path>",
        handler: handle_nhs_shell_command,
    },
    BuiltinCommand {
        name: "listen",
        aliases: &[],
        category: "apps",
        summary: "receive NHS package over serial",
        usage: "listen",
        handler: handle_nhs_shell_command,
    },
    BuiltinCommand {
        name: "uninstall",
        aliases: &[],
        category: "apps",
        summary: "remove installed NHS slot",
        usage: "uninstall <slot>",
        handler: handle_nhs_shell_command,
    },
];

fn shell_command_line() -> Option<([u8; 64], usize)> {
    let mut command = [0u8; 64];
    let len = shell_buffer_to_ascii_lower(&mut command)?;
    Some((command, len))
}

fn shell_command_str<'a>(command: &'a [u8; 64], len: usize) -> Option<&'a str> {
    core::str::from_utf8(&command[..len]).ok()
}

fn shell_codepoint_line() -> ([u32; 64], usize) {
    let mut command = [0u32; 64];
    let len = shell_buffer_snapshot(&mut command);
    (command, len)
}

fn find_builtin_command(token: &str) -> Option<&'static BuiltinCommand> {
    BUILTIN_COMMANDS
        .iter()
        .find(|command| command.matches(token))
}

fn print_command_details(command: &BuiltinCommand) {
    locale::print_localized_fmt(
        0x0B,
        format_args!(
            "[HELP] {} [{}] - {}",
            command.name, command.category, command.summary
        ),
    );
    locale::print_localized_fmt(0x0E, format_args!("[HELP] usage: {}", command.usage));
    if !command.aliases.is_empty() {
        locale::print_localized_fmt(
            0x08,
            format_args!("[HELP] aliases: {}", command.aliases.join(", ")),
        );
    }
}

fn print_shell_help_overview() {
    match locale::get_locale() {
        kernel_messages::Locale::ArEg => {
            locale::print_localized_line("=== محرك NeroShizaDev-OS Unicode ===", 0x0E);
            locale::print_localized_line("Unicode 17.0 / UTF-32 / UCS-4", 0x0E);
            locale::print_localized_fmt(
                0x0E,
                format_args!("الكتل:     {}", unicode_blocks::block_count()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("الخطوط:   {}", unicode_scripts::script_count()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("الرموز:   {}", unicode_categories::total_defined_chars()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("النطاقات: {}", unicode_categories::category_range_count()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "القاموس:   {} أوامر ({} بايت)",
                    shell_dictionary_size(),
                    shell_dictionary_bytes()
                ),
            );
            locale::print_localized_line("الأوامر:", 0x0B);
            locale::print_localized_line("  خروج / exit           - خروج", 0x0E);
            locale::print_localized_line("  مساعدة / help / ?     - مساعدة", 0x0E);
            locale::print_localized_line("  مسح / clear / cls     - تنظيف الشاشة", 0x0E);
            locale::print_localized_line("  حالة / status         - حالة النظام", 0x0E);
            locale::print_localized_line("  اعادة / reboot        - إعادة تشغيل", 0x0E);
            locale::print_localized_line("  apps / menu           - قائمة التطبيقات", 0x0E);
            locale::print_localized_line("  install demo|hello    - تثبيت حزمة NHS", 0x0E);
            locale::print_localized_line("  install serial        - تثبيت عبر COM1", 0x0E);
            locale::print_localized_line("  install list          - سجل تطبيقات NHS المثبتة", 0x0E);
            locale::print_localized_line("  install disk          - فهرس NSFS الخام", 0x0E);
            locale::print_localized_line("  install view <path>   - عرض ملف نصي من NSFS", 0x0E);
            locale::print_localized_line("  nsfs / view <path>    - أوامر مختصرة", 0x0E);
            locale::print_localized_line("  uninstall 0           - إزالة فتحة NHS", 0x0E);
            locale::print_localized_line("И.Б.И.П.:", 0x0B);
            locale::print_localized_line("  whoami / manifest     - شخصية / مانيفست", 0x0E);
            locale::print_localized_line("  entropy / shannon     - انتروبيا شانون", 0x0E);
            locale::print_localized_line("  rng / rand            - رقم عشوائي", 0x0E);
            locale::print_localized_line("  voodoo / oracle       - الحاسوب الباييزي", 0x0E);
            locale::print_localized_line("التنقل:", 0x0B);
            locale::print_localized_line("  ←/→       - تحريك المؤشر", 0x0E);
            locale::print_localized_line("  ↑/↓       - تاريخ الأوامر", 0x0E);
            locale::print_localized_line("  Home/End  - بداية/نهاية السطر", 0x0E);
            locale::print_localized_line("  PgUp/PgDn - التمرير", 0x0E);
            locale::print_localized_line("  Delete    - حذف رمز", 0x0E);
            locale::print_localized_line("الحافظة:", 0x0B);
            locale::print_localized_line("  Shift+←/→ - تحديد نص", 0x0E);
            locale::print_localized_line("  Ctrl+A    - تحديد الكل", 0x0E);
            locale::print_localized_line("  Ctrl+C/V  - نسخ/لصق", 0x0E);
            locale::print_localized_line("  Ctrl+X    - قص", 0x0E);
            locale::print_localized_line("  Ctrl+L    - مسح الشاشة", 0x0E);
            locale::print_localized_line("النظام:", 0x0B);
            locale::print_localized_line("  Esc        - إعادة تعيين المدخلات", 0x0E);
            locale::print_localized_line("  CapsLock   - تأكيد Enter البطيء", 0x0E);
            locale::print_localized_line("  ScrollLock - لوحة RUS/ENG", 0x0E);
            locale::print_localized_line("  Alt+F1..12 - تسجيل اختصار", 0x0E);
            locale::print_localized_line("  F1..F12    - تشغيل اختصار", 0x0E);
            locale::print_localized_line("اللغة:", 0x0B);
            locale::print_localized_line("  locale / ru / en / ar - تبديل اللغة", 0x0E);
            locale::print_localized_line("  lore / tech           - وضع الإخراج", 0x0E);
        }
        kernel_messages::Locale::EnUs => {
            locale::print_localized_line("=== NeroShizaDev-OS Unicode Engine ===", 0x0E);
            locale::print_localized_line("Unicode 17.0 / UTF-32 / UCS-4", 0x0E);
            locale::print_localized_fmt(
                0x0E,
                format_args!("Blocks:     {}", unicode_blocks::block_count()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("Scripts:    {}", unicode_scripts::script_count()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("Chars:      {}", unicode_categories::total_defined_chars()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("Ranges:     {}", unicode_categories::category_range_count()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "Dictionary: {} commands ({} bytes)",
                    shell_dictionary_size(),
                    shell_dictionary_bytes()
                ),
            );
            locale::print_localized_line("Commands (multi-language):", 0x0B);
            locale::print_localized_line("  exit/quit              - Shutdown", 0x0E);
            locale::print_localized_line("  help/?                 - Help", 0x0E);
            locale::print_localized_line("  clear/cls              - Clear screen", 0x0E);
            locale::print_localized_line("  status                 - Status + stats", 0x0E);
            locale::print_localized_line("  reboot                 - Reboot", 0x0E);
            locale::print_localized_line("  apps/menu              - Apps launcher", 0x0E);
            locale::print_localized_line("  install demo|hello     - Install NHS package", 0x0E);
            locale::print_localized_line("  install serial         - Receive .nhs via COM1", 0x0E);
            locale::print_localized_line(
                "  install list           - List installed NHS apps",
                0x0E,
            );
            locale::print_localized_line(
                "  install disk           - Show raw NSFS disk index",
                0x0E,
            );
            locale::print_localized_line("  install view <path>    - View NSFS text file", 0x0E);
            locale::print_localized_line("  nsfs / view <path>     - Short aliases", 0x0E);
            locale::print_localized_line("  uninstall 0            - Remove NHS slot", 0x0E);
            locale::print_localized_line("I.B.I.P.:", 0x0B);
            locale::print_localized_line("  whoami                 - Random resident", 0x0E);
            locale::print_localized_line(
                "  manifest / nero        - NERO & SHIZA philosophy",
                0x0E,
            );
            locale::print_localized_line(
                "  entropy / shannon      - Shannon entropy of input",
                0x0E,
            );
            locale::print_localized_line("  rng / rand             - RDRAND + d6 roll", 0x0E);
            locale::print_localized_line("  voodoo / oracle        - Bayesian oracle", 0x0E);
            locale::print_localized_line("Navigation:", 0x0B);
            locale::print_localized_line("  Left/Right  - Cursor move", 0x0E);
            locale::print_localized_line("  Up/Down     - Command history", 0x0E);
            locale::print_localized_line("  Home/End    - Line start/end", 0x0E);
            locale::print_localized_line("  PgUp/PgDn   - Screen scroll", 0x0E);
            locale::print_localized_line("  Delete      - Delete char", 0x0E);
            locale::print_localized_line("Selection and clipboard:", 0x0B);
            locale::print_localized_line("  Shift+Arrows - Select text", 0x0E);
            locale::print_localized_line("  Ctrl+A       - Select all", 0x0E);
            locale::print_localized_line("  Ctrl+C/V     - Copy/Paste", 0x0E);
            locale::print_localized_line("  Ctrl+X       - Cut", 0x0E);
            locale::print_localized_line("  Ctrl+L       - Clear screen", 0x0E);
            locale::print_localized_line("System:", 0x0B);
            locale::print_localized_line("  Esc        - Reset input", 0x0E);
            locale::print_localized_line("  CapsLock   - Slow Enter confirm", 0x0E);
            locale::print_localized_line("  ScrollLock - RUS/ENG keyboard", 0x0E);
            locale::print_localized_line("  Alt+F1..12 - Record hotkey", 0x0E);
            locale::print_localized_line("  F1..F12    - Run hotkey", 0x0E);
            locale::print_localized_line("Localization:", 0x0B);
            locale::print_localized_line("  locale       - RU->EN->AR->RU", 0x0E);
            locale::print_localized_line("  ru / en / ar - Set language", 0x0E);
            locale::print_localized_line("  lore / tech  - Output mode", 0x0E);
        }
        kernel_messages::Locale::RuRu => {
            locale::print_localized_line("=== NeroShizaDev-OS Unicode Engine ===", 0x0E);
            locale::print_localized_line("Unicode 17.0 / UTF-32 / UCS-4", 0x0E);
            locale::print_localized_fmt(
                0x0E,
                format_args!("Блоков:     {}", unicode_blocks::block_count()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("Скриптов:   {}", unicode_scripts::script_count()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("Символов:   {}", unicode_categories::total_defined_chars()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("Диапазонов: {}", unicode_categories::category_range_count()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "Словарь:    {} команд ({} байт)",
                    shell_dictionary_size(),
                    shell_dictionary_bytes()
                ),
            );
            locale::print_localized_line("Команды (любой язык):", 0x0B);
            locale::print_localized_line("  выход/exit/свали       - Выход", 0x0E);
            locale::print_localized_line("  помощь/help/?          - Помощь", 0x0E);
            locale::print_localized_line("  очистить/cls/clear     - Очистка", 0x0E);
            locale::print_localized_line("  статус/status          - Статус+стата", 0x0E);
            locale::print_localized_line("  ребут/reboot           - Ребут", 0x0E);
            locale::print_localized_line("  apps/menu/проги        - Лаунчер приложений", 0x0E);
            locale::print_localized_line("  install demo|hello     - Установить NHS-пакет", 0x0E);
            locale::print_localized_line(
                "  install serial         - Принять .nhs через COM1",
                0x0E,
            );
            locale::print_localized_line("  install list           - Реестр NHS-приложений", 0x0E);
            locale::print_localized_line(
                "  install disk           - Сырой индекс диска NSFS",
                0x0E,
            );
            locale::print_localized_line("  install view <path>    - Просмотр файла NSFS", 0x0E);
            locale::print_localized_line("  nsfs / view <path>     - Короткие алиасы", 0x0E);
            locale::print_localized_line("  uninstall 0            - Удалить NHS-слот", 0x0E);
            locale::print_localized_line("И.Б.И.П.:", 0x0B);
            locale::print_localized_line("  whoami/кто             - Случайный резидент", 0x0E);
            locale::print_localized_line("  manifest/нейро/шиза   - Манифест NERO & SHIZA", 0x0E);
            locale::print_localized_line("  entropy/шеннон         - Энтропия Шеннона ввода", 0x0E);
            locale::print_localized_line("  rng/рандом/кубик       - RDRAND + кубик d6", 0x0E);
            locale::print_localized_line("  voodoo/акинатор        - Байесовский оракул", 0x0E);
            locale::print_localized_line("Навигация:", 0x0B);
            locale::print_localized_line("  ←/→       - Курсор по строке", 0x0E);
            locale::print_localized_line("  ↑/↓       - История команд", 0x0E);
            locale::print_localized_line("  Home/End  - Начало/конец строки", 0x0E);
            locale::print_localized_line("  PgUp/PgDn - Прокрутка экрана", 0x0E);
            locale::print_localized_line("  Delete    - Удалить символ", 0x0E);
            locale::print_localized_line("Выделение и буфер:", 0x0B);
            locale::print_localized_line("  Shift+←/→ - Выделение текста", 0x0E);
            locale::print_localized_line("  Ctrl+A    - Выделить всё", 0x0E);
            locale::print_localized_line("  Ctrl+C/V  - Копировать/Вставить", 0x0E);
            locale::print_localized_line("  Ctrl+X    - Вырезать", 0x0E);
            locale::print_localized_line("  Ctrl+L    - Очистить экран", 0x0E);
            locale::print_localized_line("Системные:", 0x0B);
            locale::print_localized_line("  Esc       - Сброс ввода", 0x0E);
            locale::print_localized_line("  CapsLock  - Медленный Enter", 0x0E);
            locale::print_localized_line("  ScrollLock- RUS/ENG язык", 0x0E);
            locale::print_localized_line("  Alt+F1..12- Запись хоткея", 0x0E);
            locale::print_localized_line("  F1..F12   - Выполнить хоткей", 0x0E);
            locale::print_localized_line("Локализация:", 0x0B);
            locale::print_localized_line("  locale/локаль  - RU->EN->AR->RU", 0x0E);
            locale::print_localized_line("  ru / en / ar   - Установить язык", 0x0E);
            locale::print_localized_line("  lore/лор       - Режим NeroShizaDev", 0x0E);
            locale::print_localized_line("  tech/тех       - Инженерный режим", 0x0E);
        }
    }
}

fn print_shell_status() {
    state::bump_shell_command_stat(3);
    state::bump_shell_command_total();

    match locale::get_locale() {
        kernel_messages::Locale::ArEg => {
            locale::print_localized_line("=== حالة النواة ===", 0x0E);
            locale::print_localized_line("محرك يونيكود: UTF-32 / UCS-4 (v17.0)", 0x0E);
            locale::print_localized_line("نقطة الكود = 32 بت. دائماً.", 0x0E);
            locale::print_localized_fmt(
                0x0E,
                format_args!("الكتل:     {}", unicode_blocks::block_count()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("الخطوط:   {}", unicode_scripts::script_count()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("الرموز:   {}", unicode_categories::total_defined_chars()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("القاموس:   {} نية", shell_dictionary_size()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("الذاكرة:   [u32; 64] = {} بايت", 64 * 4),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("السجل:     {} أمر (الحد 32)", state::shell_history_count()),
            );
        }
        kernel_messages::Locale::EnUs => {
            locale::print_localized_line("=== Kernel Status ===", 0x0E);
            locale::print_localized_line("Unicode Engine: UTF-32 / UCS-4 (v17.0)", 0x0E);
            locale::print_localized_line("Codepoint = 32 bits. Always.", 0x0E);
            locale::print_localized_fmt(
                0x0E,
                format_args!("Blocks:     {} (full map)", unicode_blocks::block_count()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "Scripts:    {} (all languages)",
                    unicode_scripts::script_count()
                ),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("Chars:      {}", unicode_categories::total_defined_chars()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("Dictionary: {} intents", shell_dictionary_size()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("Buffer:     [u32; 64] = {} bytes", 64 * 4),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "History:    {} commands (max 32)",
                    state::shell_history_count()
                ),
            );
        }
        kernel_messages::Locale::RuRu => {
            locale::print_localized_line("=== Статус ядра ===", 0x0E);
            locale::print_localized_line("Unicode Engine: UTF-32 / UCS-4 (v17.0)", 0x0E);
            locale::print_localized_line("Кодпоинт = 32 бит. Всегда. Везде.", 0x0E);
            locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "Блоков:     {} (полная карта)",
                    unicode_blocks::block_count()
                ),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "Скриптов:   {} (все языки)",
                    unicode_scripts::script_count()
                ),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("Символов:   {}", unicode_categories::total_defined_chars()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("Словарь:    {} намерений", shell_dictionary_size()),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!("Буфер:      [u32; 64] = {} байт", 64 * 4),
            );
            locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "История:    {} команд (макс 32)",
                    state::shell_history_count()
                ),
            );
        }
    }

    locale::print_localized_fmt(
        0x0E,
        format_args!(
            "Locale: {} | Mode: {}",
            kernel_messages::locale_name(locale::get_locale()),
            kernel_messages::mode_name(locale::get_mode())
        ),
    );
    apps::rtc::display_status();

    let total = state::shell_command_total();
    let help = state::shell_command_stat(1);
    let clear = state::shell_command_stat(2);
    let status = state::shell_command_stat(3);
    let menger = state::shell_command_stat(5);
    let beep = state::shell_command_stat(6);
    let time = state::shell_command_stat(7);
    let unknown = state::shell_command_stat(8);

    match locale::get_locale() {
        kernel_messages::Locale::ArEg => {
            locale::print_localized_line("=== الإحصاءات ===", 0x0B);
            locale::print_localized_fmt(0x0E, format_args!("المجموع:    {}", total));
            locale::print_localized_fmt(0x0E, format_args!("  مساعدة:   {}", help));
            locale::print_localized_fmt(0x0E, format_args!("  منجر:     {}", menger));
            locale::print_localized_fmt(0x0E, format_args!("  صوت:      {}", beep));
            locale::print_localized_fmt(0x0E, format_args!("  وقت:      {}", time));
            locale::print_localized_fmt(0x0E, format_args!("  مسح:      {}", clear));
            locale::print_localized_fmt(0x0E, format_args!("  حالة:     {}", status));
            locale::print_localized_fmt(0x0E, format_args!("  مجهول:    {}", unknown));
        }
        kernel_messages::Locale::EnUs => {
            locale::print_localized_line("=== Statistics ===", 0x0B);
            locale::print_localized_fmt(0x0E, format_args!("Total:      {}", total));
            locale::print_localized_fmt(0x0E, format_args!("  help:      {}", help));
            locale::print_localized_fmt(0x0E, format_args!("  menger:    {}", menger));
            locale::print_localized_fmt(0x0E, format_args!("  beep:      {}", beep));
            locale::print_localized_fmt(0x0E, format_args!("  time:      {}", time));
            locale::print_localized_fmt(0x0E, format_args!("  clear:     {}", clear));
            locale::print_localized_fmt(0x0E, format_args!("  status:    {}", status));
            locale::print_localized_fmt(0x0E, format_args!("  unknown:   {}", unknown));
        }
        kernel_messages::Locale::RuRu => {
            locale::print_localized_line("=== Статистика ===", 0x0B);
            locale::print_localized_fmt(0x0E, format_args!("Всего:      {}", total));
            locale::print_localized_fmt(0x0E, format_args!("  помощь:    {}", help));
            locale::print_localized_fmt(0x0E, format_args!("  губка:     {}", menger));
            locale::print_localized_fmt(0x0E, format_args!("  звук:      {}", beep));
            locale::print_localized_fmt(0x0E, format_args!("  время:     {}", time));
            locale::print_localized_fmt(0x0E, format_args!("  очистить:  {}", clear));
            locale::print_localized_fmt(0x0E, format_args!("  статус:    {}", status));
            locale::print_localized_fmt(0x0E, format_args!("  неизвестно:{}", unknown));
        }
    }

    let mut has_hotkeys = false;
    for slot in 0..12usize {
        if state::shell_hotkey_len(slot) > 0 {
            has_hotkeys = true;
            break;
        }
    }

    if has_hotkeys {
        match locale::get_locale() {
            kernel_messages::Locale::ArEg => {
                locale::print_localized_line("=== مفاتيح سريعة ===", 0x0B)
            }
            kernel_messages::Locale::EnUs => locale::print_localized_line("=== Hotkeys ===", 0x0B),
            kernel_messages::Locale::RuRu => locale::print_localized_line("=== Хоткеи ===", 0x0B),
        }
        for slot in 0..12usize {
            let len = state::shell_hotkey_len(slot);
            if len > 0 {
                print!("  F{}: ", slot + 1);
                for index in 0..len {
                    let cp = state::shell_hotkey_codepoint(slot, index);
                    if let Some(ch) = char::from_u32(cp) {
                        print!("{}", ch);
                    }
                }
                locale::print_localized_line("", 0x0E);
            }
        }
    }
}

fn handle_shell_intent_command() -> bool {
    let (buffer, len) = shell_codepoint_line();
    if len == 0 {
        return false;
    }

    let buffer = &buffer[..len];
    match lookup_shell_intent(buffer) {
        ShellIntent::Exit => {
            state::bump_shell_command_stat(0);
            state::bump_shell_command_total();
            if crate::irq_guard::allow_heavy_operation() {
                kernel_messages::print_exit_phrase();
                unsafe {
                    x86_64::instructions::port::Port::<u16>::new(0x604).write(0x2000);
                }
                x86_64::instructions::interrupts::disable();
                loop {
                    x86_64::instructions::hlt();
                }
            } else {
                runtime::defer_shell_shutdown();
            }
        }
        ShellIntent::Help => {
            state::bump_shell_command_stat(1);
            state::bump_shell_command_total();
            print_shell_help_overview();
        }
        ShellIntent::Clear => {
            state::bump_shell_command_stat(2);
            state::bump_shell_command_total();
            crate::fb_buffer::clear_screen();
        }
        ShellIntent::Status => {
            print_shell_status();
        }
        ShellIntent::Reboot => {
            state::bump_shell_command_stat(4);
            state::bump_shell_command_total();
            if crate::irq_guard::allow_heavy_operation() {
                locale::render_event_auto(kernel_messages::KernelEvent::ShellReboot);
                let mut port = x86_64::instructions::port::Port::new(0x64);
                unsafe {
                    port.write(0xfeu8);
                }
            } else {
                runtime::defer_shell_reboot();
            }
        }
        ShellIntent::LocaleCycle => {
            let new_locale = locale::cycle_locale();
            locale::draw_locale_badge();
            match new_locale {
                kernel_messages::Locale::RuRu => {
                    locale::render_event_auto(kernel_messages::KernelEvent::ShellLocaleRu)
                }
                kernel_messages::Locale::EnUs => {
                    locale::render_event_auto(kernel_messages::KernelEvent::ShellLocaleEn)
                }
                kernel_messages::Locale::ArEg => {
                    locale::render_event_auto(kernel_messages::KernelEvent::ShellLocaleAr)
                }
            }
        }
        ShellIntent::LocaleRu => {
            locale::set_locale(kernel_messages::Locale::RuRu);
            locale::draw_locale_badge();
            locale::render_event_auto(kernel_messages::KernelEvent::ShellLocaleRu);
        }
        ShellIntent::LocaleEn => {
            locale::set_locale(kernel_messages::Locale::EnUs);
            locale::draw_locale_badge();
            locale::render_event_auto(kernel_messages::KernelEvent::ShellLocaleEn);
        }
        ShellIntent::LocaleAr => {
            locale::set_locale(kernel_messages::Locale::ArEg);
            locale::draw_locale_badge();
            locale::render_event_auto(kernel_messages::KernelEvent::ShellLocaleAr);
        }
        ShellIntent::ModeLore => {
            locale::set_mode(kernel_messages::MessageMode::Lore);
            locale::draw_locale_badge();
            locale::render_event_auto(kernel_messages::KernelEvent::ShellModeLore);
        }
        ShellIntent::ModeTech => {
            locale::set_mode(kernel_messages::MessageMode::Technical);
            locale::draw_locale_badge();
            locale::render_event_auto(kernel_messages::KernelEvent::ShellModeTech);
        }
        ShellIntent::Apps => {
            state::bump_shell_command_total();
            if crate::irq_guard::allow_heavy_operation() {
                apps::activity::run_activity_manager();
            } else {
                runtime::defer_shell_apps();
            }
        }
        ShellIntent::WhoAmI => {
            state::bump_shell_command_total();
            let idx = apps::rng::random_range(5) as usize;
            const RESIDENTS: [&str; 5] = [
                "Петрович (Король жижи)",
                "Группа К.А.Ф.И.Д.Р.А. (Анализ аномалий)",
                "Банановый Турист",
                "Dr. Bred",
                "ДедушкаВКрутую",
            ];
            print_whoami_result(RESIDENTS[idx]);
        }
        ShellIntent::Manifest => {
            state::bump_shell_command_total();
            print_manifest_output();
        }
        ShellIntent::Entropy => {
            state::bump_shell_command_total();
            let mut bytes = [0u8; 64];
            for (index, cp) in buffer.iter().enumerate() {
                bytes[index] = (*cp & 0xFF) as u8;
            }
            let entropy = apps::fpu::shannon_entropy(&bytes[..len]);
            print_entropy_result(entropy);
        }
        ShellIntent::Rng => {
            state::bump_shell_command_total();
            let random = apps::rng::random_range(u64::MAX);
            let dice = apps::rng::random_range(6) + 1;
            print_rng_result(random, dice);
        }
        ShellIntent::Voodoo => {
            state::bump_shell_command_total();
            voodoo_engine::demo_cellular_automaton();
        }
        ShellIntent::Unknown => {
            state::bump_shell_command_stat(8);
            state::bump_shell_command_total();
            let first_cp = buffer[0];
            let block = unicode::unicode_block_name(first_cp);
            let script = unicode::unicode_script_name(first_cp);
            let cat = unicode::unicode_category(first_cp);
            locale::render_event_auto(kernel_messages::KernelEvent::ShellUnknownCommand);
            if let Some((name, dist)) = closest_shell_intent(buffer) {
                print_unknown_command_suggestion(name, dist);
            }
            match locale::get_locale() {
                kernel_messages::Locale::ArEg => {
                    locale::print_localized_fmt(0x0E, format_args!("U+{:04X}", first_cp));
                    locale::print_localized_fmt(0x0E, format_args!("  الكتلة:    {}", block));
                    locale::print_localized_fmt(0x0E, format_args!("  الخط:      {}", script));
                    locale::print_localized_fmt(0x0E, format_args!("  الفئة:     {}", cat.name()));
                }
                kernel_messages::Locale::EnUs => {
                    locale::print_localized_fmt(0x0E, format_args!("U+{:04X}", first_cp));
                    locale::print_localized_fmt(0x0E, format_args!("  Block:    {}", block));
                    locale::print_localized_fmt(0x0E, format_args!("  Script:   {}", script));
                    locale::print_localized_fmt(0x0E, format_args!("  Category: {}", cat.name()));
                }
                kernel_messages::Locale::RuRu => {
                    locale::print_localized_fmt(0x0E, format_args!("U+{:04X}", first_cp));
                    locale::print_localized_fmt(0x0E, format_args!("  Блок:     {}", block));
                    locale::print_localized_fmt(0x0E, format_args!("  Скрипт:   {}", script));
                    locale::print_localized_fmt(0x0E, format_args!("  Категория: {}", cat.name()));
                }
            }
        }
    }

    true
}

fn handle_help_command() -> bool {
    let Some((command, len)) = shell_command_line() else {
        return false;
    };
    let Some(line) = shell_command_str(&command, len) else {
        return false;
    };

    let mut parts = line.split_whitespace();
    let Some(cmd) = parts.next() else {
        return false;
    };
    if cmd != "help" && cmd != "?" {
        return false;
    }

    state::bump_shell_command_total();

    match parts.next() {
        Some(topic) => match find_builtin_command(topic) {
            Some(command) => print_command_details(command),
            None => {
                locale::print_localized_fmt(0x0C, format_args!("[HELP] unknown command: {}", topic))
            }
        },
        None => print_shell_help_overview(),
    }

    true
}

fn handle_irq_guard_debug_command() -> bool {
    if shell_buffer_eq_ascii("irqdbg") {
        validator::display_irq_guard_status();
        return true;
    }

    if shell_buffer_eq_ascii("irqguard on") {
        crate::irq_guard::set_guard_enabled(true);
        locale::print_localized_line("[IRQGUARD] ON", 0x0A);
        return true;
    }

    if shell_buffer_eq_ascii("irqguard off") {
        crate::irq_guard::set_guard_enabled(false);
        locale::print_localized_line("[IRQGUARD] OFF", 0x0C);
        return true;
    }

    if shell_buffer_eq_ascii("irqguard reset") {
        crate::irq_guard::reset_counters();
        locale::print_localized_line("[IRQGUARD] counters reset", 0x0B);
        return true;
    }

    false
}

fn handle_log_debug_command() -> bool {
    let Some((command, len)) = shell_command_line() else {
        return false;
    };
    let Some(line) = shell_command_str(&command, len) else {
        return false;
    };

    let mut parts = line.split_whitespace();
    let Some(cmd) = parts.next() else {
        return false;
    };
    if cmd != "log" && cmd != "diag" {
        return false;
    }

    match parts.next() {
        Some("status") | None => {
            locale::print_localized_fmt(
                0x0B,
                format_args!(
                    "[LOG] level={} format={} dedup={} ps2={} apps={} games={} doom={} bite={} mem={} sys={} irqguard={} validator={} tribe={}",
                    crate::serial::min_level().as_str(),
                    crate::serial::format().as_str(),
                    if crate::serial::dedup_enabled() { 1 } else { 0 },
                    if crate::serial::subsys_enabled("PS2") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("APPS") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("GAMES") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("DOOM") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("BITE") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("MEM") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("SYS") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("IRQGUARD") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("VALIDATOR") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("TRIBE") {
                        1
                    } else {
                        0
                    },
                ),
            );
            true
        }
        Some("level") => {
            let Some(level_name) = parts.next() else {
                locale::print_localized_line(
                    "[LOG] usage: log level trace|debug|info|warn|error|fatal",
                    0x0C,
                );
                return true;
            };
            match crate::serial::LogLevel::parse(level_name) {
                Some(level) => {
                    crate::serial::set_min_level(level);
                    locale::print_localized_fmt(
                        0x0A,
                        format_args!("[LOG] level={}", level.as_str()),
                    );
                }
                None => {
                    locale::print_localized_line("[LOG] bad level", 0x0C);
                }
            }
            true
        }
        Some("subsys") => {
            let Some(subsys) = parts.next() else {
                locale::print_localized_line("[LOG] usage: log subsys <name> on|off", 0x0C);
                return true;
            };
            let Some(state_name) = parts.next() else {
                locale::print_localized_line("[LOG] usage: log subsys <name> on|off", 0x0C);
                return true;
            };
            match state_name {
                "on" => {
                    crate::serial::set_subsys_enabled(subsys, true);
                    locale::print_localized_fmt(0x0A, format_args!("[LOG] subsys={} on", subsys));
                }
                "off" => {
                    crate::serial::set_subsys_enabled(subsys, false);
                    locale::print_localized_fmt(0x0A, format_args!("[LOG] subsys={} off", subsys));
                }
                _ => {
                    locale::print_localized_line("[LOG] usage: log subsys <name> on|off", 0x0C);
                }
            }
            true
        }
        Some("format") => {
            let Some(format_name) = parts.next() else {
                locale::print_localized_line(
                    "[LOG] usage: log format canonical|compact|logfmt",
                    0x0C,
                );
                return true;
            };
            match crate::serial::LogFormat::parse(format_name) {
                Some(format) => {
                    crate::serial::set_format(format);
                    locale::print_localized_fmt(
                        0x0A,
                        format_args!("[LOG] format={}", format.as_str()),
                    );
                }
                None => locale::print_localized_line("[LOG] bad format", 0x0C),
            }
            true
        }
        Some("dedup") => {
            let Some(state_name) = parts.next() else {
                locale::print_localized_line("[LOG] usage: log dedup on|off", 0x0C);
                return true;
            };
            match state_name {
                "on" => {
                    crate::serial::set_dedup_enabled(true);
                    locale::print_localized_line("[LOG] dedup=on", 0x0A);
                }
                "off" => {
                    crate::serial::set_dedup_enabled(false);
                    locale::print_localized_line("[LOG] dedup=off", 0x0A);
                }
                _ => locale::print_localized_line("[LOG] usage: log dedup on|off", 0x0C),
            }
            true
        }
        Some("preset") => {
            match parts.next() {
                Some("quiet") => {
                    crate::serial::set_min_level(crate::serial::LogLevel::Info);
                    crate::serial::set_subsys_enabled("PS2", false);
                    crate::serial::set_subsys_enabled("MEM", false);
                    crate::serial::set_format(crate::serial::LogFormat::Compact);
                    crate::serial::set_dedup_enabled(true);
                    locale::print_localized_line("[LOG] preset=quiet", 0x0A);
                }
                Some("debug") => {
                    crate::serial::set_min_level(crate::serial::LogLevel::Debug);
                    crate::serial::set_subsys_enabled("PS2", true);
                    crate::serial::set_subsys_enabled("MEM", true);
                    crate::serial::set_format(crate::serial::LogFormat::Canonical);
                    crate::serial::set_dedup_enabled(true);
                    locale::print_localized_line("[LOG] preset=debug", 0x0A);
                }
                Some("structured") => {
                    crate::serial::set_min_level(crate::serial::LogLevel::Info);
                    crate::serial::set_format(crate::serial::LogFormat::LogFmt);
                    crate::serial::set_dedup_enabled(true);
                    locale::print_localized_line("[LOG] preset=structured", 0x0A);
                }
                _ => locale::print_localized_line(
                    "[LOG] usage: log preset quiet|debug|structured",
                    0x0C,
                ),
            }
            true
        }
        _ => {
            locale::print_localized_line(
                "[LOG] usage: log status | log level ... | log subsys ... | log format ... | log dedup ... | log preset ...",
                0x0C,
            );
            true
        }
    }
}

fn print_nhs_usage() {
    locale::print_localized_line("[NHS] install demo | hello | serial | list", 0x0B);
    locale::print_localized_line("[NHS] install list  - show installed app registry", 0x08);
    locale::print_localized_line("[NHS] install disk  - show raw NSFS disk index", 0x08);
    locale::print_localized_line("[NHS] install view <path>  - view NSFS text file", 0x08);
    locale::print_localized_line("[NHS] nsfs [disk] | nsfs view <path>", 0x08);
    locale::print_localized_line("[NHS] view <path>  - shortcut for NSFS viewer", 0x08);
    locale::print_localized_line("[NHS] listen  - alias for install serial", 0x08);
    locale::print_localized_line("[NHS] uninstall <slot>", 0x0B);
}

fn print_nhs_registry() {
    let installed = apps::installer::registry::installed_count();
    let free = apps::installer::slots::free_count();
    let disk_objects = apps::installer::slots::disk_object_count();
    let disk_used = apps::installer::slots::disk_used_bytes();
    let disk_capacity = apps::installer::slots::disk_capacity_bytes();

    locale::print_localized_fmt(
        0x0B,
        format_args!(
            "[NHS] installed={} free_slots={} disk_objects={} disk={}/{} bytes",
            installed, free, disk_objects, disk_used, disk_capacity,
        ),
    );

    if installed == 0 {
        locale::print_localized_line("[NHS] registry is empty", 0x08);
    } else {
        apps::installer::registry::for_each(|slot, app| {
            let (major, minor, patch) = app.version_tuple();
            locale::print_localized_fmt(
                0x0E,
                format_args!(
                    "[NHS] slot #{}  {}  v{}.{}.{}  {} bytes",
                    slot,
                    app.name_str(),
                    major,
                    minor,
                    patch,
                    app.installed_size,
                ),
            );
        });
    }
}

fn print_nsfs_disk_summary() {
    locale::print_localized_fmt(
        0x0B,
        format_args!(
            "[NSFS] objects={} disk={}/{} bytes",
            apps::installer::slots::disk_object_count(),
            apps::installer::slots::disk_used_bytes(),
            apps::installer::slots::disk_capacity_bytes(),
        ),
    );
}

fn print_nsfs_disk_view() {
    print_nsfs_disk_summary();
    locale::print_localized_line("[NSFS] objects:", 0x0B);
    let mut seen = 0usize;
    apps::installer::slots::for_each_disk_object(|entry| {
        seen += 1;
        locale::print_localized_fmt(
            0x0E,
            format_args!(
                "[NSFS] {:>12}  {:>6} bytes  {}",
                nsfs_kind_name(entry.kind),
                entry.size_bytes,
                entry.path(),
            ),
        );
    });

    if seen == 0 {
        locale::print_localized_line("[NSFS] disk index is empty", 0x08);
    }
}

fn print_nsfs_viewer_usage() {
    locale::print_localized_line("[NSFS] usage: view <path> | nsfs view <path>", 0x08);
}

fn print_nsfs_file(path: &str) {
    match apps::installer::slots::read_path(path) {
        Some(data) => {
            locale::print_localized_fmt(
                0x0B,
                format_args!("[NSFS] {}  ({} bytes)", path, data.len()),
            );
            if let Ok(text) = core::str::from_utf8(data) {
                for line in text.lines() {
                    locale::print_localized_line(line, 0x0F);
                }
            } else {
                locale::print_localized_line("[NSFS] file is binary or invalid UTF-8", 0x08);
            }
        }
        None => locale::print_localized_fmt(0x0C, format_args!("[NSFS] file not found: {}", path)),
    }
}

fn nsfs_kind_name(kind: apps::installer::nsfs::NsfsObjectKind) -> &'static str {
    match kind {
        apps::installer::nsfs::NsfsObjectKind::Empty => "empty",
        apps::installer::nsfs::NsfsObjectKind::KernelAsset => "kernel",
        apps::installer::nsfs::NsfsObjectKind::NhsPackage => "nhs",
        apps::installer::nsfs::NsfsObjectKind::Config => "config",
        apps::installer::nsfs::NsfsObjectKind::Save => "save",
        apps::installer::nsfs::NsfsObjectKind::Log => "log",
        apps::installer::nsfs::NsfsObjectKind::Temp => "temp",
        apps::installer::nsfs::NsfsObjectKind::DirectoryMarker => "dir",
    }
}

fn run_nhs_install_serial() {
    locale::print_localized_line(
        "[NHS] Waiting for .nhs on COM1... (protocol: NHS_SYNC -> NHS_READY -> [size][data])",
        0x0E,
    );
    locale::print_localized_line(
        "[NHS] QEMU must use: -serial tcp:127.0.0.1:4321,server,nowait",
        0x08,
    );
    locale::print_localized_line(
        "[NHS] Host: python tools/send_nhs.py app.nhs --tcp localhost:4321",
        0x08,
    );
    match apps::installer::serial_recv::receive() {
        Ok(data) => run_nhs_install(data, "<serial>"),
        Err(e) => {
            locale::print_localized_fmt(0x0C, format_args!("[NHS] Receive error: {}", e.message()));
        }
    }
}

fn run_nhs_install(package: &[u8], label: &str) {
    let result = apps::installer::installer::install(package);
    crate::fb_buffer::clear_screen();
    locale::draw_locale_badge();

    match result {
        apps::installer::installer::InstallResult::Ok(slot) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("[NHS] Installed {} into slot #{}", label, slot),
            );
        }
        other => {
            locale::print_localized_fmt(
                0x0C,
                format_args!("[NHS] {}: {}", label, other.description()),
            );
        }
    }
}

fn handle_nhs_shell_command() -> bool {
    let Some((command, len)) = shell_command_line() else {
        return false;
    };
    let Some(line) = shell_command_str(&command, len) else {
        return false;
    };

    let mut parts = line.split_whitespace();
    let Some(cmd) = parts.next() else {
        return false;
    };

    match cmd {
        "install" => {
            state::bump_shell_command_total();
            match parts.next() {
                Some("demo") => run_nhs_install(NHS_DEMO_PACKAGE, "demo.nhs"),
                Some("hello") => run_nhs_install(NHS_HELLO_PACKAGE, "hello.nhs"),
                Some("serial") => run_nhs_install_serial(),
                Some("list") => print_nhs_registry(),
                Some("disk") => print_nsfs_disk_view(),
                Some("view") | Some("cat") | Some("read") => match parts.next() {
                    Some(path) => print_nsfs_file(path),
                    None => print_nsfs_viewer_usage(),
                },
                Some("help") | None => print_nhs_usage(),
                Some(other) => {
                    locale::print_localized_fmt(
                        0x0C,
                        format_args!("[NHS] unknown package: {}", other),
                    );
                    print_nhs_usage();
                }
            }
            true
        }
        "nsfs" | "disk" => {
            state::bump_shell_command_total();
            match parts.next() {
                None | Some("disk") | Some("list") => print_nsfs_disk_view(),
                Some("view") | Some("cat") | Some("read") => match parts.next() {
                    Some(path) => print_nsfs_file(path),
                    None => print_nsfs_viewer_usage(),
                },
                Some("help") => print_nhs_usage(),
                Some(other) => {
                    locale::print_localized_fmt(
                        0x0C,
                        format_args!("[NSFS] unknown subcommand: {}", other),
                    );
                    print_nhs_usage();
                }
            }
            true
        }
        "view" | "cat" | "read" => {
            state::bump_shell_command_total();
            match parts.next() {
                Some(path) => print_nsfs_file(path),
                None => print_nsfs_viewer_usage(),
            }
            true
        }
        "listen" => {
            state::bump_shell_command_total();
            run_nhs_install_serial();
            true
        }
        "uninstall" => {
            state::bump_shell_command_total();
            match parts.next().and_then(|slot| slot.parse::<usize>().ok()) {
                Some(slot) => match apps::installer::installer::uninstall(slot) {
                    Ok(_) => {}
                    Err(apps::installer::installer::UninstallError::UserCanceled) => {
                        locale::print_localized_line("[NHS] removal canceled", 0x08);
                    }
                    Err(error) => {
                        locale::print_localized_fmt(
                            0x0C,
                            format_args!("[NHS] slot #{}: {}", slot, error.description()),
                        );
                    }
                },
                None => print_nhs_usage(),
            }
            true
        }
        _ => false,
    }
}

pub(super) fn handle_shell_commands() -> bool {
    let Some((command, len)) = shell_command_line() else {
        return false;
    };
    let Some(line) = shell_command_str(&command, len) else {
        return false;
    };
    let Some(first) = line.split_whitespace().next() else {
        return false;
    };

    match find_builtin_command(first) {
        Some(command) => (command.handler)(),
        None => handle_shell_intent_command(),
    }
}
