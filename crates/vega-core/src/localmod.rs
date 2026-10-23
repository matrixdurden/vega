//! Mods and clients added from a `.jar` file instead of Modrinth.

use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::Path,
};

use base64::Engine;
use serde::Deserialize;

use crate::{Error, Result, modrinth::InstalledMod};

/// Icons bigger than this stay out of vega-mods.json.
const MAX_ICON_BYTES: u64 = 96 * 1024;

#[derive(Deserialize)]
struct FabricModJson {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    icon: Option<IconField>,
}

/// `"icon": "path.png"` or `"icon": { "16": "a.png", "128": "b.png" }`.
#[derive(Deserialize)]
#[serde(untagged)]
enum IconField {
    One(String),
    Sizes(HashMap<String, String>),
}

impl IconField {
    fn largest(&self) -> Option<&str> {
        match self {
            IconField::One(path) => Some(path),
            IconField::Sizes(sizes) => {
                sizes.iter().max_by_key(|(size, _)| size.parse::<u32>().unwrap_or(0)).map(|(_, path)| path.as_str())
            }
        }
    }
}

/// The bare file name of an uploaded jar, refusing anything that is not one.
pub fn jar_name(filename: &str) -> Result<String> {
    let name = Path::new(filename.trim()).file_name().and_then(|n| n.to_str()).unwrap_or_default();
    if name.is_empty() || !name.to_ascii_lowercase().ends_with(".jar") {
        return Err(Error::other(format!("{filename} is not a .jar file")));
    }
    Ok(name.to_owned())
}

/// Reads name, version and icon from the jar's fabric.mod.json, falling
/// back to the file name for jars that have none.
pub fn describe(filename: &str, bytes: &[u8]) -> Result<InstalledMod> {
    let filename = jar_name(filename)?;
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| Error::other(format!("{filename} is not a valid jar")))?;
    let meta: Option<FabricModJson> =
        archive.by_name("fabric.mod.json").ok().and_then(|entry| serde_json::from_reader(entry).ok());

    let icon_url = meta.as_ref().and_then(|meta| meta.icon.as_ref()).and_then(|icon| icon.largest()).and_then(|path| {
        let mut entry = archive.by_name(path.trim_start_matches('/')).ok()?;
        if entry.size() > MAX_ICON_BYTES {
            return None;
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data).ok()?;
        let mime = if path.ends_with(".jpg") || path.ends_with(".jpeg") { "image/jpeg" } else { "image/png" };
        Some(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(data)))
    });

    let stem = filename.trim_end_matches(".jar").trim_end_matches(".JAR").to_owned();
    let meta = meta.unwrap_or(FabricModJson { id: None, name: None, version: None, icon: None });
    Ok(InstalledMod {
        project_id: format!("file:{}", meta.id.unwrap_or_else(|| stem.clone())),
        version_id: String::new(),
        version_number: meta.version.unwrap_or_default(),
        title: meta.name.unwrap_or(stem),
        icon_url,
        filename,
        dependency: false,
        enabled: true,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn jar(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, data) in files {
            writer.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn reads_fabric_metadata() {
        let bytes = jar(&[
            ("fabric.mod.json", br#"{"id":"meteor-client","name":"Meteor Client","version":"1.0","icon":{"16":"s.png","128":"big.png"}}"#),
            ("big.png", b"\x89PNG"),
        ]);
        let m = describe("meteor-client-26.3.jar", &bytes).unwrap();
        assert_eq!(
            (m.project_id.as_str(), m.title.as_str(), m.version_number.as_str()),
            ("file:meteor-client", "Meteor Client", "1.0")
        );
        assert!(m.icon_url.unwrap().starts_with("data:image/png;base64,"));
    }

    #[test]
    fn falls_back_to_file_name() {
        let m = describe("some-client.jar", &jar(&[("a.class", b"x")])).unwrap();
        assert_eq!((m.title.as_str(), m.project_id.as_str()), ("some-client", "file:some-client"));
    }

    #[test]
    fn rejects_non_jars() {
        assert!(describe("evil.exe", &jar(&[])).is_err());
        assert!(describe("broken.jar", b"not a zip").is_err());
        assert_eq!(jar_name("../../x/mod.jar").unwrap(), "mod.jar");
    }
}
