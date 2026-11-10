use super::{LOGIN_GATED_SECTIONS, is_hidden_for_channel, is_visible_for_channel};
use crate::settings_view::SettingsSection;
use warp_core::channel::Channel;

#[test]
fn oss_hides_login_gated_sections() {
    for section in LOGIN_GATED_SECTIONS {
        assert!(
            is_hidden_for_channel(*section, Channel::Oss),
            "{section:?} should be hidden in OSS"
        );
    }
}

#[test]
fn release_channels_keep_every_section() {
    for channel in [
        Channel::Stable,
        Channel::Preview,
        Channel::Dev,
        Channel::Local,
    ] {
        for section in LOGIN_GATED_SECTIONS {
            assert!(
                !is_hidden_for_channel(*section, channel),
                "{section:?} must stay visible on {channel}"
            );
        }
    }
}

#[test]
fn oss_keeps_core_sections() {
    for section in [
        SettingsSection::Appearance,
        SettingsSection::Features,
        SettingsSection::Keybindings,
        SettingsSection::Privacy,
        SettingsSection::About,
        SettingsSection::Scripting,
        SettingsSection::SharedBlocks,
        SettingsSection::CodeIndexing,
        SettingsSection::EditorAndCodeReview,
    ] {
        assert!(
            !is_hidden_for_channel(section, Channel::Oss),
            "{section:?} should stay visible"
        );
    }
}

#[test]
fn local_pi_opt_in_keeps_warp_agent_page_in_oss() {
    assert!(
        is_visible_for_channel(SettingsSection::WarpAgent, Channel::Oss, true),
        "the pi model override lives on this page, so it must stay reachable"
    );
}

#[test]
fn local_pi_opt_in_does_not_unhide_other_gated_sections() {
    for section in LOGIN_GATED_SECTIONS
        .iter()
        .copied()
        .filter(|s| *s != SettingsSection::WarpAgent)
    {
        assert!(
            !is_visible_for_channel(section, Channel::Oss, true),
            "{section:?} must stay hidden even with the opt-in on"
        );
    }
}

#[test]
fn without_the_opt_in_the_page_stays_hidden_in_oss() {
    assert!(!is_visible_for_channel(
        SettingsSection::WarpAgent,
        Channel::Oss,
        false
    ));
}

#[test]
fn release_channels_ignore_the_opt_in() {
    for channel in [
        Channel::Stable,
        Channel::Preview,
        Channel::Dev,
        Channel::Local,
    ] {
        assert!(is_visible_for_channel(
            SettingsSection::WarpAgent,
            channel,
            false
        ));
    }
}

#[test]
fn every_login_gated_section_is_a_known_variant() {
    // Guards against a typo'd variant silently disabling the whole list.
    assert!(LOGIN_GATED_SECTIONS.len() >= 10);
    assert!(LOGIN_GATED_SECTIONS.contains(&SettingsSection::WarpDrive));
}
