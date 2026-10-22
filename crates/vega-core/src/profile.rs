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
    pub game_version: String,
    pub loader: Loader,
    /// Pinned when the profile is created so updates never surprise anyone.
    #[serde(default)]
    pub loader_version: Option<String>,
    /// Unix seconds.
    #[serde(default)]
    pub last_played: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub username: String,
    pub memory_mb: u32,
    pub selected_profile: Option<String>,
    /// `tr` or `en`; follows the system when unset.
    pub language: Option<String>,
    /// Hide the launcher while the game runs so it uses no GPU time.
    pub hide_on_launch: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { username: "Steve".into(), memory_mb: 4096, selected_profile: None, language: None, hide_on_launch: true }
    }
}
