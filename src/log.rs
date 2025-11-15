use anyhow::Result;
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{
    Layer, filter::Targets, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::{cli::LogLevel, time::LocalDateTime};

pub fn init_logger(log_level: &LogLevel) -> Result<()> {
    let level: Level = log_level.to_tracing_level();

    // Filter: disable everything except `nifa`
    let filter = Targets::new()
        .with_default(LevelFilter::OFF)
        .with_target("nifa", level);

    // Build subscriber layer
    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_timer(LocalDateTime)
        .with_filter(filter);

    // Compose registry + formatting layer
    tracing_subscriber::registry().with(fmt_layer).init();

    Ok(())
}
