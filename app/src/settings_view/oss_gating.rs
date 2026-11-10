//! OSS-channel gating for settings sections.
//!
//! The open-source build ships without Warp account services, so sections
//! whose content requires a logged-in user (agents, cloud drive, billing,
//! teams, referrals, the account page itself) are hidden instead of showing
//! dead-end login prompts. Release channels keep every section.

use warp_core::channel::Channel;

use super::SettingsSection;

/// Sections that require a Warp account and are therefore hidden in the OSS
/// channel. Kept exhaustive over the login-gated surface so a newly added
/// section is a conscious decision rather than a silent leak.
pub(crate) const LOGIN_GATED_SECTIONS: &[SettingsSection] = &[
    SettingsSection::Account,
    SettingsSection::BillingAndUsage,
    SettingsSection::Teams,
    SettingsSection::Referrals,
    SettingsSection::WarpDrive,
    // Agents umbrella subpages.
    SettingsSection::WarpAgent,
    SettingsSection::AgentProfiles,
    SettingsSection::AgentMCPServers,
    SettingsSection::Knowledge,
    SettingsSection::ThirdPartyCLIAgents,
];

/// Whether `section` is hidden when building for `channel`.
pub(crate) fn is_hidden_for_channel(section: SettingsSection, channel: Channel) -> bool {
    channel == Channel::Oss && LOGIN_GATED_SECTIONS.contains(&section)
}

/// Whether `section` is hidden in the channel this process is running as.
pub(crate) fn is_hidden(section: SettingsSection) -> bool {
    is_hidden_for_channel(section, warp_core::channel::ChannelState::channel())
}

/// Whether `section` should be shown, given the channel and whether the
/// account-free local pi opt-in is on.
///
/// The opt-in keeps the Warp Agent page reachable in the OSS channel even though
/// the page is login-gated: it is where the pi model override lives, and a user
/// running without an account has no other way to reach it. Everything else that
/// is login-gated stays hidden, so the exemption is scoped to this one section.
pub(crate) fn is_visible_for_channel(
    section: SettingsSection,
    channel: Channel,
    local_pi_only_enabled: bool,
) -> bool {
    if local_pi_only_enabled && section == SettingsSection::WarpAgent {
        return true;
    }
    !is_hidden_for_channel(section, channel)
}

#[cfg(test)]
#[path = "oss_gating_tests.rs"]
mod tests;
