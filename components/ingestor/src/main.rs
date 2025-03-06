use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv();

    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_ansi(true)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");

    Ok(())
}
