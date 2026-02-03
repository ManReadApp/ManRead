use std::{str::FromStr as _, time::SystemTime};

use fern::colors::{Color, ColoredLevelConfig};
use log::{info, LevelFilter};

use super::env::Config;

pub fn init_logger(log_mode: &str) -> Result<(), log::SetLoggerError> {
    let colors = ColoredLevelConfig::default()
        .trace(Color::Cyan)
        .debug(Color::Blue)
        .info(Color::Green);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::from_str(log_mode).unwrap_or(LevelFilter::Warn))
        .level_for("selectors", LevelFilter::Off) //remove logger for scraping
        .level_for("html5ever", LevelFilter::Off)
        .level_for("hyper", LevelFilter::Off)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

pub fn log_url(config: &Config) {
    #[cfg(feature = "log-ip")]
    if let Ok(ip) = local_ip_address::local_ip() {
        info!("Starting server at http://{}:{}/", &ip, config.port);
    }
    info!("Starting server at http://0.0.0.0:{}/", config.port);
}
