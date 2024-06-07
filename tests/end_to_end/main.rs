use std::{
    env,
    process::{Child, Command},
    time::Duration,
};

use ::entity::*;
use anyhow::anyhow;
use cucumber::World;
use once_cell::sync::OnceCell;
use sea_orm::*;
use thirtyfour::prelude::*;

mod features;

static WEB_DRIVER: OnceCell<WebDriver> = OnceCell::new();

pub const BASE_URL: &str = "http://localhost:3000";
pub const MAIL_DEV_URL: &str = "http://localhost:1080";

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TestWorld {
    driver: WebDriver,
    slow_down_delay: Duration,
}

impl TestWorld {
    pub async fn new() -> anyhow::Result<Self> {
        let driver = WEB_DRIVER.get().unwrap().clone();
        let slow_down_delay_seconds = env::var("SLOW_DOWN_DELAY_SECONDS")
            .unwrap_or_default()
            .parse()
            .unwrap_or_default();
        let slow_down_delay = Duration::from_secs_f64(slow_down_delay_seconds);

        Ok(Self {
            driver,
            slow_down_delay,
        })
    }

    pub async fn wait_for_delay(&self) {
        tokio::time::sleep(self.slow_down_delay).await;
    }
}

struct GeckoDriver {
    child: Child,
}

impl GeckoDriver {
    pub async fn start() -> Self {
        let child = Command::new("geckodriver").spawn().unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        Self { child }
    }
}

impl Drop for GeckoDriver {
    fn drop(&mut self) {
        println!("Killing geckodriver");
        self.child.kill().unwrap();
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load env
    dotenv::dotenv()?;

    // Initialize database
    let db = Database::connect(env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to the database");

    // Clean everything in the database
    // organization::Entity::delete_many().exec(&db).await?;
    user_account::Entity::delete_many().exec(&db).await?;

    // Start gecko driver in the background
    // The process will be stopped automatically on drop
    let _gecko_driver = GeckoDriver::start().await;

    // Start the web driver
    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new("http://localhost:4444", caps).await?;
    WEB_DRIVER
        .set(driver.clone())
        .map_err(|_| anyhow!("Failed to set WEB_DRIVER"))?;

    // Run the tests
    TestWorld::run("tests/end_to_end/features").await;

    // Stop the web driver gracefully
    driver.quit().await.unwrap();

    Ok(())
}
