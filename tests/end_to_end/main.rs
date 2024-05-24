use std::{process::{Child, Command}, time::Duration};

use anyhow::anyhow;
use cucumber::World;
use once_cell::sync::OnceCell;
use thirtyfour::prelude::*;

mod features;

static WEB_DRIVER: OnceCell<WebDriver> = OnceCell::new();

pub const BASE_URL: &str = "http://localhost:3000";
pub const MAIL_DEV_URL: &str = "http://localhost:1080";

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TestWorld {
    driver: WebDriver,
}

impl TestWorld {
    pub async fn new() -> anyhow::Result<Self> {
        let driver = WEB_DRIVER.get().unwrap().clone();

        Ok(Self { driver })
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
    // Start gecko driver in the background
    // The process will be stopped automatically on drop
    let _gecko_driver = GeckoDriver::start().await;

    // Start the web driver
    let mut caps = DesiredCapabilities::firefox();
    caps.add_arg("--width=1600")?;
    caps.add_arg("--height=900")?;
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
