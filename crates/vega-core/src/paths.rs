use std::path::{Path, PathBuf};

/// On-disk layout. Libraries, assets and Java runtimes are shared between
/// profiles, so ten profiles on the same version cost one download.
///
/// ```text
/// <root>/
///   assets/  libraries/  natives/  runtimes/  versions/
///   instances/<profile-id>/     game directory (mods, saves, options.txt)
///   profiles.json  settings.json
/// ```
#[derive(Debug, Clone)]
pub struct Paths {
    root: PathBuf,
}

impl Paths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// `~/.local/share/vega`, `%APPDATA%\vega` or `~/Library/Application Support/vega`.
    pub fn platform_default() -> Self {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        Self::new(base.join("vega"))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn assets(&self) -> PathBuf {
        self.root.join("assets")
    }

    pub fn libraries(&self) -> PathBuf {
        self.root.join("libraries")
    }

    pub fn runtimes(&self) -> PathBuf {
        self.root.join("runtimes")
    }

    pub fn versions(&self) -> PathBuf {
        self.root.join("versions")
    }

    pub fn natives(&self, version: &str) -> PathBuf {
        self.root.join("natives").join(version)
    }

    pub fn version_json(&self, id: &str) -> PathBuf {
        self.versions().join(id).join(format!("{id}.json"))
    }

    pub fn version_jar(&self, id: &str) -> PathBuf {
        self.versions().join(id).join(format!("{id}.jar"))
    }

    pub fn instance(&self, profile_id: &str) -> PathBuf {
        self.root.join("instances").join(profile_id)
    }

    pub fn profiles_file(&self) -> PathBuf {
        self.root.join("profiles.json")
    }

    pub fn settings_file(&self) -> PathBuf {
        self.root.join("settings.json")
    }
}
