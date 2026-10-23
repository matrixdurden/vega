//! Vega launcher core.
//!
//! Everything that talks to Mojang, Microsoft, Fabric and Modrinth and turns
//! a profile into a running Minecraft process lives here. There is no GUI
//! code in this crate, so it builds and tests anywhere Rust does.

mod error;
mod launcher;
mod net;
mod paths;
mod rules;
mod store;

pub mod account;
pub mod fabric;
pub mod java;
pub mod launch;
pub mod localmod;
pub mod modrinth;
pub mod mojang;
pub mod mrpack;
pub mod msa;
pub mod profile;

pub use account::AccountSummary;
pub use error::{Error, Result};
pub use launch::LaunchPlan;
pub use launcher::{GameVersion, Launcher, ModResult};
pub use modrinth::{InstalledMod, SearchHit};
pub use msa::DeviceCode;
pub use net::{Progress, ProgressEvent, Stage, no_progress};
pub use paths::Paths;
pub use profile::{Loader, Profile, ProfileDraft, Settings};
