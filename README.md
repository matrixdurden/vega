<p align="center">
  <img src=".github/assets/banner.svg" alt="Vega, a fast, simple and open source Minecraft launcher" width="100%">
</p>

<p align="center">
  <a href="https://github.com/matrixdurden/vega/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/matrixdurden/vega/ci.yml?branch=main&style=flat-square&label=CI&color=7f77dd" alt="CI"></a>
  <a href="https://github.com/matrixdurden/vega/releases"><img src="https://img.shields.io/github/v/release/matrixdurden/vega?include_prereleases&style=flat-square&color=7f77dd&label=release" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-7f77dd?style=flat-square" alt="License: GPL-3.0"></a>
  <img src="https://img.shields.io/badge/Tauri-2-24c8db?style=flat-square&logo=tauri&logoColor=white" alt="Tauri 2">
  <img src="https://img.shields.io/badge/Rust-2024-f74c00?style=flat-square&logo=rust&logoColor=white" alt="Rust 2024">
  <img src="https://img.shields.io/badge/Svelte-5-ff3e00?style=flat-square&logo=svelte&logoColor=white" alt="Svelte 5">
</p>

<p align="center">
  <b>English</b> · <a href="README.tr.md">Türkçe</a>
</p>

---

Vega is a Minecraft launcher that gets out of your way. Pick a version, add a few mods from Modrinth, press **Play**. It downloads only what is missing, checks every file, brings its own Java, and stays tiny because the heavy lifting happens in Rust.

<table>
  <tr>
    <td colspan="2"><img src=".github/assets/screenshot-home.jpg" alt="A Vega profile with its mods"></td>
  </tr>
  <tr>
    <td><img src=".github/assets/screenshot-mods.jpg" alt="Adding mods from Modrinth"></td>
    <td><img src=".github/assets/screenshot-profile.jpg" alt="Creating a profile with an icon"></td>
  </tr>
</table>

## Features

- **Profiles front and centre.** Every profile sits in the sidebar with its own icon, Minecraft version, mods and worlds.
- **Fabric in one click.** Profiles pin a Fabric loader version, so nothing changes under you.
- **Mods you can switch off.** Turn a mod off without deleting it, or remove it for good.
- **Modrinth built in.** Search, install and remove mods. Required dependencies such as Fabric API come along automatically.
- **Java handled for you.** Vega installs the exact Java runtime each Minecraft version expects, from Mojang's own builds.
- **Fast, verified downloads.** 32 parallel downloads, SHA-1 checks wherever the source publishes a hash, automatic retries, atomic writes.
- **Nothing downloaded twice.** Libraries, assets and Java are shared between profiles. Ten profiles on 1.21 cost one download.
- **Out of your way while you play.** The launcher hides itself while the game runs and pauses its animations.
- **Light.** A native Rust core and a Svelte 5 UI on the system webview: no bundled browser.
- **Turkish and English.**

## Download

Installers for Windows, macOS and Linux will be published on the [releases page](https://github.com/matrixdurden/vega/releases) starting with v0.1.0.

> [!NOTE]
> Vega currently uses **offline accounts**. Microsoft sign-in is next on the roadmap and is waiting for Mojang's API approval. Please buy the game: offline mode is meant for development and for people who already own Minecraft.

## Roadmap

- [x] Vanilla and Fabric profiles with icons, per-profile memory and worlds
- [x] Modrinth search and installs with dependencies, mods on/off
- [x] Automatic Java runtimes
- [x] Game console and crash notice
- [ ] Microsoft account sign-in
- [ ] NeoForge and Quilt
- [ ] Modrinth modpacks (`.mrpack`)
- [ ] Crash analyser that explains what went wrong
- [ ] Profile snapshots and one-click rollback
- [ ] Auto-updates

## Building from source

You need [Rust](https://rustup.rs) (stable), [Node.js](https://nodejs.org) 24 and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform. On Debian or Ubuntu:

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

Then:

```bash
git clone https://github.com/matrixdurden/vega.git
cd vega
npm install
npm run tauri dev      # run the app with hot reload
npm run tauri build    # build an installer
```

Working on the UI only? `npm run dev` opens it in a browser with a mock backend, no Rust needed.

The launcher core has no GUI dependencies, so it builds and tests on its own:

```bash
cargo test -p vega-core
# Real end-to-end run: Fabric 1.21.1 + Sodium, then the game for 60 seconds
cargo run -p vega-core --example smoke -- /tmp/vega-smoke 1.21.1 sodium 60
```

## How it is put together

```text
crates/vega-core/   Launcher core in Rust, no GUI code
  mojang.rs         Version manifest, libraries, assets, rules
  fabric.rs         Fabric loader profiles
  java.rs           Java runtimes from Mojang's runtime manifest
  modrinth.rs       Search and installs with dependency resolution
  launch.rs         Builds the java command line
  net.rs            Parallel, verified, retrying downloads
src-tauri/          Tauri shell: commands, events, the game process
src/                Svelte 5 interface
```

Game data lives in `%APPDATA%\vega` on Windows, `~/Library/Application Support/vega` on macOS and `~/.local/share/vega` on Linux.

## Contributing

Issues and pull requests are welcome. Before opening a pull request, please run:

```bash
cargo fmt --all && cargo clippy -p vega-core --all-targets -- -D warnings && cargo test -p vega-core
npm run check
```

## License

Vega is free software under the [GNU General Public License v3.0 or later](LICENSE).

Vega is not an official Minecraft product and is not approved by or associated with Mojang or Microsoft.
