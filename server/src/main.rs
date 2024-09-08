#[macro_use]
extern crate rust_i18n;

use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod application;
mod domain;
mod infrastructure;
mod shared;

// Initialize i18n
rust_i18n::i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv();

    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_ansi(true)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");
    application::start_application().await?;
    Ok(())
}
