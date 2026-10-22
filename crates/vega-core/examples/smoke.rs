//! End-to-end smoke test against the real Mojang, Fabric and Modrinth APIs.
//!
//! ```sh
//! cargo run -p vega-core --example smoke -- /tmp/vega-smoke 1.21.1 sodium 60
//! ```
//! Creates a Fabric profile, installs a mod with its dependencies, downloads
//! everything and runs the game for the given number of seconds.

use std::{sync::Arc, time::Duration};

use tokio::io::{AsyncBufReadExt, BufReader};
use vega_core::{Launcher, Loader, Paths, ProgressEvent, Settings};

#[tokio::main]
async fn main() -> vega_core::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = args.first().map_or("/tmp/vega-smoke", String::as_str);
    let game = args.get(1).map_or("1.21.1", String::as_str);
    let mod_slug = args.get(2).map_or("sodium", String::as_str);
    let seconds: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

    let launcher = Launcher::new(Paths::new(root))?;
    launcher.save_settings(Settings { username: "VegaSmoke".into(), memory_mb: 2048, ..Settings::default() }).await?;
    let profile = launcher.create_profile(&format!("Smoke {game}"), game, Loader::Fabric).await?;
    println!("profile: {} (fabric {:?})", profile.id, profile.loader_version);

    let mods = launcher.install_mod(&profile.id, mod_slug).await?;
    for m in &mods {
        println!("mod: {} {} dependency={}", m.title, m.version_number, m.dependency);
    }

    let started = std::time::Instant::now();
    let progress = Arc::new(|e: ProgressEvent| {
        if e.done == e.total {
            println!("  {:?}: {}/{}", e.stage, e.done, e.total);
        }
    });
    let plan = launcher.prepare(&profile.id, progress).await?;
    println!("prepared in {:.1}s, java = {}", started.elapsed().as_secs_f32(), plan.java.display());

    if seconds == 0 {
        println!("{} {}", plan.java.display(), plan.args.join(" "));
        return Ok(());
    }
    let mut child = plan.command().spawn()?;
    let stdout = child.stdout.take().unwrap();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("[game] {line}");
        }
    });
    match tokio::time::timeout(Duration::from_secs(seconds), child.wait()).await {
        Ok(status) => println!("game exited early: {:?}", status?),
        Err(_) => {
            println!("game still running after {seconds}s, stopping it");
            child.kill().await?;
        }
    }
    Ok(())
}
