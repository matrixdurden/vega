use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    Vanilla,
    Fabric,
}

/// One playable setup: a Minecraft version, a loader and its own game
/// directory (mods, worlds, options).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    /// A built-in icon key, or a `data:image/...` URL the player uploaded.
    #[serde(default = "default_icon")]
    pub icon: String,
    pub game_version: String,
    pub loader: Loader,
    /// Pinned when the profile is created so updates never surprise anyone.
    #[serde(default)]
    pub loader_version: Option<String>,
    /// The ready-made client this profile was created from, if any.
    #[serde(default)]
    pub client: Option<String>,
    /// Overrides the global memory setting when set.
    #[serde(default)]
    pub memory_mb: Option<u32>,
    /// Unix seconds.
    #[serde(default)]
    pub last_played: Option<u64>,
}

fn default_icon() -> String {
    "grass".into()
}

/// What the player fills in when creating or editing a profile.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDraft {
    pub name: String,
    pub icon: String,
    pub game_version: String,
    pub loader: Loader,
    #[serde(default)]
    pub memory_mb: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub memory_mb: u32,
    pub selected_profile: Option<String>,
    /// `tr` or `en`; follows the system when unset.
    pub language: Option<String>,
    /// Hide the launcher while the game runs so it uses no GPU time.
    pub hide_on_launch: bool,
    /// The offline name from before accounts existed; read once to migrate.
    #[serde(rename = "username", skip_serializing)]
    pub legacy_username: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self { memory_mb: 4096, selected_profile: None, language: None, hide_on_launch: true, legacy_username: None }
    }
}
