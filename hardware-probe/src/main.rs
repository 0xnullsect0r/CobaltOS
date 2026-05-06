mod dmi;
mod keyboard;
mod audio;
mod display;
mod power;
mod service;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("cobalt-hardware-probe v{}", env!("CARGO_PKG_VERSION"));

    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--fix-audio".to_string()) {
        return audio::fix_audio().await;
    }

    // Normal first-boot probe
    let board = dmi::detect_board().await?;
    info!("Detected board: {}", board.name);

    keyboard::apply_remapping(&board).await?;
    audio::apply_ucm_profile(&board).await?;
    display::apply_scaling(&board).await?;
    power::apply_power_profile(&board).await?;

    info!("Hardware probe complete for board: {}", board.name);
    Ok(())
}
