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

#[cfg(test)]
#[path = "oss_gating_tests.rs"]
mod tests;
