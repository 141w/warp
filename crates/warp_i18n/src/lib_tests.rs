use crate::{Language, set_language, setting_description, t};

fn collect_leaf_keys(value: &serde_yaml::Value, prefix: String, out: &mut Vec<String>) {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (k, v) in map {
                let key = k.as_str().expect("yaml keys are strings");
                let path = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{prefix}.{key}")
                };
                collect_leaf_keys(v, path, out);
            }
        }
        serde_yaml::Value::Null => {}
        _ => out.push(prefix),
    }
}

#[test]
fn locales_have_symmetric_keys() {
    let en: serde_yaml::Value =
        serde_yaml::from_str(include_str!("../locales/en.yml")).expect("en.yml parses");
    let zh: serde_yaml::Value =
        serde_yaml::from_str(include_str!("../locales/zh-CN.yml")).expect("zh-CN.yml parses");

    let mut en_keys = Vec::new();
    collect_leaf_keys(&en, String::new(), &mut en_keys);
    let mut zh_keys = Vec::new();
    collect_leaf_keys(&zh, String::new(), &mut zh_keys);

    for key in zh_keys {
        assert!(
            en_keys.contains(&key),
            "zh-CN key `{key}` missing in en.yml"
        );
    }
}

#[test]
fn missing_translation_falls_back_to_english() {
    // `common.probe_fallback` exists only in en.yml, so the Chinese lookup
    // misses and the configured "en" fallback answers.
    set_language(Language::Chinese);
    assert_eq!(t!("common.probe_fallback"), "Fallback works");
}

#[test]
fn translated_key_follows_active_locale() {
    set_language(Language::Chinese);
    assert_eq!(t!("common.cancel"), "取消");
    set_language(Language::English);
    assert_eq!(t!("common.cancel"), "Cancel");
}

#[test]
fn unknown_key_echoes_the_key_itself() {
    // rust-i18n returns the key verbatim when no locale has it, which is what
    // `setting_description` relies on to detect a miss.
    assert_eq!(t!("no.such.key.at.all"), "no.such.key.at.all");
}

#[test]
fn setting_description_falls_back_to_literal() {
    assert_eq!(
        setting_description("FakeSetting", "English literal"),
        "English literal"
    );
}

#[test]
fn language_locale_mapping() {
    assert_eq!(Language::Chinese.locale_str(), "zh-CN");
    assert_eq!(Language::English.locale_str(), "en");
    assert_eq!(Language::Auto.locale_str(), "en");
}
