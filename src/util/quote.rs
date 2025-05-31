use poise::serenity_prelude::{ActivityData, OnlineStatus};
use rand::Rng;

const QUOTES: &[&str] = &[
    "I LOVE TRANSGENDER WOEMN",
    "im sexy and i know it",
    "thinking github came before git is like thinking pornhub came before porn",
    "translation lookaside buff a cock up my ass",
    "mov eax 0x80000000 mov ebx [eax] int 0x80 🥺🥺🥺🥺🥺🥺🥺🥺",
    "“love is in the air!” WRONG cannibalism 🥩🥩🥩🥩🥩🥩🥩🥩🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍖🍖🍖🍖🍖🍖🍖🍖🍖🍖🔪🔪🔪🔪🔪🔪🔪🔪🔪🔪🔪🔪🔪🥩🥩🥩🥩🥩🥩🥩🥩🥩🥩🥩🥩🥩🥩🥩🥩🍴🍴🍴🍴🍴🍴🍴🍖🍖🍖🍖🍖🍖🍖🍖🍖🍴🍴🍴🍴🍴🍖🍖🍴🍴🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🍽️🔪🔪🔪🔪🔪🔪🍴🔪🔪🔪🍴🍴🍴🍴🍴",
    "`((void (__fastcall *)(_OWORD *, _QWORD, __int64))memset)(_mm_loadh_ps((const double *)&v3[-254]), 0LL, 42681LL);`",
    "ghidra is backdoored by the NSA",
    "a monad is just a monoid in the category of endofunctors",
    "nix was created by homosexuals, for homosexuals",
    "schizophrenic pond dweller : the Frog is coming",
    "GITPULLO COMMITO MERGE CONFLICTO 🗣️ 🗣️",
    "🥺i wannq fuck my computer",
    "looks like the guys doing the testing got their CFLAGS wrong. I reckon they forgot omit-frame-pointer.",
    "-g -fsanitize=undefined,address -fno-omit-frame-pointer",
    "segfault yourself",
    "cat /dev/random",
];

pub fn get_random_quote() -> &'static str {
    let mut rng = rand::rng();
    let index = rng.random_range(0..QUOTES.len());
    QUOTES[index]
}

pub fn get_random_status() -> OnlineStatus {
    let mut rng = rand::rng();
    let statuses = [
        OnlineStatus::Online,
        OnlineStatus::Idle,
        OnlineStatus::DoNotDisturb,
    ];

    let index = rng.random_range(0..statuses.len());
    statuses[index]
}

pub fn get_random_activity() -> ActivityData {
    let mut rng = rand::rng();
    let quote = get_random_quote();

    let activity_types = [
        ActivityData::playing(quote),
        ActivityData::listening(quote),
        ActivityData::watching(quote),
        ActivityData::competing(quote),
    ];

    let index = rng.random_range(0..activity_types.len());
    activity_types[index].clone()
}

pub fn get_random_interval_minutes() -> u64 {
    let mut rng = rand::rng();
    rng.random_range(3..=5)
}
