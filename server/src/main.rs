use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod application;
mod domain;
mod infrastructure;
mod shared;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_ansi(true)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");
    application::start_application().await?;
    Ok(())
}
