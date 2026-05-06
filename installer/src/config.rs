//! Locale, timezone, and keyboard layout configuration.

use anyhow::Result;

pub struct LocaleConfig {
    pub locale: String,
    pub timezone: String,
    pub keyboard: String,
}

pub async fn apply(cfg: &LocaleConfig) -> Result<()> {
    // In a real implementation:
    //   - Write /etc/locale.gen and run locale-gen
    //   - Write /etc/timezone and run dpkg-reconfigure tzdata
    //   - Write /etc/default/keyboard
    tracing::info!(
        "Locale config: locale={}, tz={}, kb={}",
        cfg.locale, cfg.timezone, cfg.keyboard
    );
    Ok(())
}
