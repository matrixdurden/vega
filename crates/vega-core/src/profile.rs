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
    /// Key of one of the icons the UI offers.
    #[serde(default = "default_icon")]
    pub icon: String,
    pub game_version: String,
    pub loader: Loader,
    /// Pinned when the profile is created so updates never surprise anyone.
    #[serde(default)]
    pub loader_version: Option<String>,
    /// Overrides the global memory setting when set.
    #[serde(default)]
    pub memory_mb: Option<u32>,
    /// Unix seconds.
    #[serde(default)]
    pub last_played: Option<u64>,
}

fn default_icon() -> String {
    "cube".into()
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

/// A save in a profile's `saves` folder.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct World {
    /// Folder name, which is also what the game shows unless renamed in-game.
    pub name: String,
    /// Unix seconds of the last save.
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
