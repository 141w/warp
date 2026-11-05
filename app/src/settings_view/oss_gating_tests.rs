use super::{is_hidden_for_channel, LOGIN_GATED_SECTIONS};
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
    for channel in [Channel::Stable, Channel::Preview, Channel::Dev, Channel::Local] {
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
        assert!(!is_hidden_for_channel(section, Channel::Oss), "{section:?} should stay visible");
    }
}

#[test]
fn every_login_gated_section_is_a_known_variant() {
    // Guards against a typo'd variant silently disabling the whole list.
    assert!(LOGIN_GATED_SECTIONS.len() >= 10);
    assert!(LOGIN_GATED_SECTIONS.contains(&SettingsSection::WarpDrive));
}
