use settings::{SupportedPlatforms, SyncToCloud};

use crate::terminal::general_settings::{GeneralSettings, GeneralSettingsChangedEvent};

/// Interface language for the UI. `Auto` follows the system locale.
#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Interface language.",
    rename_all = "snake_case"
)]
pub enum Language {
    #[default]
    Auto,
    English,
    Chinese,
}

impl From<Language> for warp_i18n::Language {
    fn from(value: Language) -> Self {
        match value {
            Language::Auto => warp_i18n::Language::Auto,
            Language::English => warp_i18n::Language::English,
            Language::Chinese => warp_i18n::Language::Chinese,
        }
    }
}

settings::macros::implement_setting_for_enum!(
    Language,
    GeneralSettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Never,
    surface: settings::SettingSurfaces::GUI,
    private: false,
    toml_path: "general.language",
    description: "Interface language.",
);
