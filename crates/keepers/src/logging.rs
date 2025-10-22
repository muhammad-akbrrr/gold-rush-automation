use crate::config::RuntimeConfig;
use std::io;
use tracing::Level;
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing(cfg: &RuntimeConfig) {
    let level = match cfg.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let filter = EnvFilter::from_default_env()
        .add_directive(level.into())
        .add_directive("hyper=warn".parse().unwrap_or_else(|_| level.into()));

    let make_writer = io::stdout;

    if cfg.log_format.to_lowercase() == "pretty" {
        let layer = fmt::layer()
            .with_writer(make_writer)
            .with_ansi(cfg.log_color)
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_level(true);

        Registry::default().with(filter).with(layer).init();
    } else {
        let layer = fmt::layer()
            .with_writer(make_writer)
            .with_ansi(false)
            .with_target(true)
            .with_level(true)
            .json()
            .flatten_event(true);

        Registry::default().with(filter).with(layer).init();
    }
}
