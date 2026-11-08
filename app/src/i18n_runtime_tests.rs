//! Verifies that the keys the UI actually references resolve at runtime,
//! not just that the strings exist somewhere in the binary.

#[test]
fn menu_and_chip_keys_resolve_under_chinese() {
    warp_i18n::set_language(warp_i18n::Language::Chinese);

    // These are the keys that previously rendered raw because of a
    // namespace-prefix mistake in the locale file.
    let cases = [
        ("menu.split_pane_right", "向右拆分窗格"),
        ("menu.copy", "复制"),
        ("menu.paste", "粘贴"),
        ("menu.close_tab", "关闭标签页"),
        ("menu.pin_tab", "固定标签页"),
        ("menu.whats_new", "新功能"),
        ("menu.keyboard_shortcuts", "键盘快捷键"),
        ("menu.log_out", "退出登录"),
        ("menu.rename_pane", "重命名窗格"),
        ("menu.move_tab_up", "标签页上移"),
        ("chips.date", "日期"),
        ("chips.git_branch", "Git 分支"),
        ("chips.time_12h", "时间（12 小时制）"),
        ("panel.sign_in_warp_drive", "登录以使用 Warp Drive"),
        ("panel.feature_unavailable", "功能不可用"),
    ];

    let mut broken = Vec::new();
    for (key, want) in cases {
        let got = warp_i18n::tr(key);
        if got != want {
            broken.push(format!("{key}: got {got:?}, want {want:?}"));
        }
    }
    assert!(broken.is_empty(), "keys that did not resolve: {broken:?}");
}

#[test]
fn menu_and_chip_keys_resolve_under_english() {
    warp_i18n::set_language(warp_i18n::Language::English);

    let cases = [
        ("menu.split_pane_right", "Split pane right"),
        ("menu.copy", "Copy"),
        ("menu.close_tab", "Close tab"),
        ("menu.whats_new", "What's new"),
        ("chips.date", "Date"),
        ("panel.sign_in_warp_drive", "Sign in to access Warp Drive"),
    ];

    let mut broken = Vec::new();
    for (key, want) in cases {
        let got = warp_i18n::tr(key);
        if got != want {
            broken.push(format!("{key}: got {got:?}, want {want:?}"));
        }
    }
    assert!(broken.is_empty(), "keys that did not resolve: {broken:?}");
}

#[test]
fn copy_chip_interpolation_uses_translated_title() {
    warp_i18n::set_language(warp_i18n::Language::Chinese);
    let title = warp_i18n::tr("chips.time_12h");
    let got = warp_i18n::t!("menu.copy_chip", name = title);
    assert_eq!(got, "复制时间（12 小时制）");
}
