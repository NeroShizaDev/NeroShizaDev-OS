use crate::apps::activity::AppKind;

use super::header::{CAT_GAME, CAT_SCIENCE, CAT_SYSTEM, CAT_TOOL};

#[derive(Clone, Copy)]
pub struct BuiltinAppSpec {
    pub slug: &'static str,
    pub name: &'static str,
    pub author: &'static str,
    pub version: [u8; 3],
    pub category: u8,
    pub kind: AppKind,
    pub summary: &'static str,
}

pub fn find_builtin_by_kind(kind: AppKind) -> Option<&'static BuiltinAppSpec> {
    BUILTIN_APPS.iter().find(|spec| spec.kind == kind)
}

pub const BUILTIN_APPS: &[BuiltinAppSpec] = &[
    BuiltinAppSpec {
        slug: "games",
        name: "Games",
        author: "NeroShizaDev Core",
        version: [1, 0, 0],
        category: CAT_GAME,
        kind: AppKind::Games,
        summary: "Games hub with arcade launcher",
    },
    BuiltinAppSpec {
        slug: "doom",
        name: "Doom",
        author: "NeroShizaDev Core",
        version: [1, 0, 0],
        category: CAT_GAME,
        kind: AppKind::Doom,
        summary: "Standalone Doom runtime",
    },
    BuiltinAppSpec {
        slug: "tribe",
        name: "Tribe",
        author: "NeroShizaDev Core",
        version: [1, 0, 0],
        category: CAT_GAME,
        kind: AppKind::Tribe,
        summary: "Standalone Tribe world",
    },
    BuiltinAppSpec {
        slug: "jackal",
        name: "Jackal",
        author: "NeroShizaDev Core",
        version: [1, 0, 0],
        category: CAT_TOOL,
        kind: AppKind::Jackal,
        summary: "Jackal analyzer and archive shell",
    },
    BuiltinAppSpec {
        slug: "menger",
        name: "Menger",
        author: "NeroShizaDev Core",
        version: [1, 0, 0],
        category: CAT_SCIENCE,
        kind: AppKind::Menger,
        summary: "3D Menger sponge renderer",
    },
    BuiltinAppSpec {
        slug: "calculator",
        name: "Calculator",
        author: "NeroShizaDev Core",
        version: [1, 0, 0],
        category: CAT_SCIENCE,
        kind: AppKind::Calculator,
        summary: "Engineering calculator",
    },
    BuiltinAppSpec {
        slug: "fpu",
        name: "FPU",
        author: "NeroShizaDev Core",
        version: [1, 0, 0],
        category: CAT_SCIENCE,
        kind: AppKind::Fpu,
        summary: "x87 math and entropy demo",
    },
    BuiltinAppSpec {
        slug: "voodoo",
        name: "Voodoo",
        author: "NeroShizaDev Core",
        version: [1, 0, 0],
        category: CAT_SCIENCE,
        kind: AppKind::Voodoo,
        summary: "Bayesian oracle and automata",
    },
    BuiltinAppSpec {
        slug: "rng",
        name: "RNG",
        author: "NeroShizaDev Core",
        version: [1, 0, 0],
        category: CAT_SCIENCE,
        kind: AppKind::Rng,
        summary: "Hardware random and dice",
    },
    BuiltinAppSpec {
        slug: "chronos",
        name: "Chronos",
        author: "NeroShizaDev Core",
        version: [1, 0, 0],
        category: CAT_SYSTEM,
        kind: AppKind::Chronos,
        summary: "HEX clock and Psychotown time",
    },
    BuiltinAppSpec {
        slug: "rtc",
        name: "RTC",
        author: "NeroShizaDev Core",
        version: [1, 0, 0],
        category: CAT_SYSTEM,
        kind: AppKind::Rtc,
        summary: "CMOS and hardware RTC view",
    },
    BuiltinAppSpec {
        slug: "beeper",
        name: "Beeper",
        author: "NeroShizaDev Core",
        version: [1, 0, 0],
        category: CAT_SYSTEM,
        kind: AppKind::Beeper,
        summary: "PC speaker and hexatonic demo",
    },
    BuiltinAppSpec {
        slug: "language",
        name: "Language",
        author: "NeroShizaDev Core",
        version: [1, 0, 0],
        category: CAT_SYSTEM,
        kind: AppKind::Locale,
        summary: "Locale switcher",
    },
];
