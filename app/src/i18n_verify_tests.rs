// Verifies every bindings.* key resolves to a real translation under zh-CN
// (rather than echoing the key back), which is what the UI would show.

#[test]
fn all_binding_keys_resolve_in_chinese() {
    warp_i18n::set_language(warp_i18n::Language::Chinese);

    let en = include_str!("../../crates/warp_i18n/locales/en.yml");
    let doc: serde_yaml::Value = serde_yaml::from_str(en).expect("en.yml parses");
    let bindings = doc["bindings"].as_mapping().expect("bindings mapping");

    let mut missing = Vec::new();
    for (k, _v) in bindings {
        let key = k.as_str().expect("string key");
        let full = format!("bindings.{key}");
        let got = warp_i18n::tr(&full);
        // rust-i18n echoes the key when the lookup fails.
        if got == full {
            missing.push(key.to_string());
        }
    }
    assert!(
        missing.is_empty(),
        "keys that did not resolve ({}): {:?}",
        missing.len(),
        missing
    );
}

#[test]
fn slash_command_prefix_interpolates() {
    warp_i18n::set_language(warp_i18n::Language::Chinese);
    let got = warp_i18n::t!("bindings.slash_command_prefix", name = "/agent");
    assert_eq!(got, "斜杠命令：/agent");

    warp_i18n::set_language(warp_i18n::Language::English);
    let got_en = warp_i18n::t!("bindings.slash_command_prefix", name = "/agent");
    assert_eq!(got_en, "Slash command: /agent");
}

#[test]
fn sampled_command_names_are_chinese() {
    warp_i18n::set_language(warp_i18n::Language::Chinese);
    let cases = [
        ("bindings.workspace:activate_next_tab", "切换到下一个标签页"),
        ("bindings.workspace:open_repository", "打开仓库"),
        (
            "bindings.workspace:show_settings_appearance_page",
            "打开设置：外观",
        ),
        ("bindings.terminal:cancel_command", "取消活动进程"),
        (
            "bindings.pane_group:toggle_maximize_pane",
            "切换最大化活动窗格",
        ),
        (
            "bindings.workspace:toggle_vertical_tabs_panel",
            "切换竖排标签页面板",
        ),
        ("bindings.editor_view:insert_autosuggestion", "接受自动建议"),
        (
            "bindings.file_tree:toggle_hidden_files",
            "切换项目浏览器中的隐藏文件",
        ),
    ];
    for (key, want) in cases {
        assert_eq!(warp_i18n::tr(key), want, "key {key}");
    }
}
