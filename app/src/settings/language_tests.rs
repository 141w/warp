use settings::Setting;
use settings_value::SettingsValue;

use super::Language;
use warp_i18n::Language as I18nLanguage;

#[test]
fn language_defaults_to_auto() {
    assert_eq!(*Language::new(None).value(), Language::Auto);
}

#[test]
fn language_toml_path() {
    assert_eq!(Language::toml_path(), Some("general.language"));
}

#[test]
fn language_serializes_to_snake_case() {
    assert_eq!(Language::Chinese.to_file_value(), serde_json::json!("chinese"));
    assert_eq!(Language::Auto.to_file_value(), serde_json::json!("auto"));
}

#[test]
fn language_converts_to_i18n_language() {
    assert_eq!(I18nLanguage::from(Language::Chinese), I18nLanguage::Chinese);
    assert_eq!(I18nLanguage::from(Language::Auto), I18nLanguage::Auto);
    assert_eq!(I18nLanguage::from(Language::English), I18nLanguage::English);
}

