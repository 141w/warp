//! Internationalization layer for Warp UI strings.
//!
//! Static keys are translated at compile time via the re-exported `t!` macro;
//! runtime language switching goes through `set_language`.

pub use rust_i18n::i18n;

// Embed the locale files shipped next to this crate. `fallback` makes missing
// zh-CN keys resolve to English instead of rendering the key itself.
i18n!("locales", fallback = "en");

pub use rust_i18n::t;

/// UI language selection. `Auto` follows the system locale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    Auto,
    English,
    Chinese,
}

impl Language {
    /// Locale identifier handed to rust-i18n. `Auto` is resolved against the
    /// system locale by `set_language`, so its own value is only a placeholder.
    pub fn locale_str(&self) -> &'static str {
        match self {
            Language::Auto | Language::English => "en",
            Language::Chinese => "zh-CN",
        }
    }
}

/// Switches the active UI language, resolving `Auto` against the system locale.
pub fn set_language(language: Language) {
    let resolved = match language {
        Language::Auto => detect_system_language(),
        other => other,
    };
    rust_i18n::set_locale(resolved.locale_str());
}

/// Maps the OS locale to a supported `Language`; anything not Chinese is English.
pub fn detect_system_language() -> Language {
    match sys_locale::get_locale() {
        Some(locale) if locale.to_lowercase().starts_with("zh") => Language::Chinese,
        _ => Language::English,
    }
}

/// Translates a macro-defined setting description via its derived key
/// `settings.desc.<SettingName>`, falling back to the English literal baked
/// into the settings macro when no translation exists.
pub fn setting_description(setting_name: &str, fallback: &str) -> String {
    let key = format!("settings.desc.{setting_name}");
    let translated = t!(key.as_str()).into_owned();
    if translated == key {
        fallback.to_string()
    } else {
        translated
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;
