use std::env;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());

    let registry = tracing_subscriber::registry().with(env_filter);

    match log_format.as_str() {
        "json" => registry
            .with(fmt::layer().json())
            .init(),
        _ => registry
            .with(fmt::layer().pretty())
            .init(),
    }
}
