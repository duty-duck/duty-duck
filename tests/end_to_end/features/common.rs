use std::time::Duration;

use cucumber::*;
use thirtyfour::prelude::*;
use thirtyfour::stringmatch::Needle;

use crate::TestWorld;

#[derive(Clone)]
pub struct PartialTextNeedle {
    search_pattern: String,
}

impl Needle for PartialTextNeedle {
    fn is_match(&self, haystack: &str) -> bool {
        haystack.contains(&self.search_pattern)
    }
}

#[given(expr = "the {string} field is filled with {string}")]
#[when(expr = "the {string} field is filled with {string}")]
pub async fn fill_field(world: &mut TestWorld, field_name: String, field_value: String) {
    let input = world
        .driver
        .query(By::Css(format!("input[name=\"{field_name}\"]")))
        .first()
        .await
        .expect("cannot find input field");
    input
        .send_keys(field_value)
        .await
        .expect("cannot fill input");
    world.wait_for_delay().await;
}

#[given("the form is submitted")]
#[when("the form is submitted")]
pub async fn submit_form(world: &mut TestWorld) {
    let submit_button = world
        .driver
        .query(By::Css("form button[type=\"submit\"]"))
        .first()
        .await
        .expect("cannot find form submit button");
    submit_button.click().await.unwrap();
    world.wait_for_delay().await;
}

#[then(expr = "the {string} element should show {string}")]
pub async fn assert_page_showing_text(world: &mut TestWorld, element: String, content: String) {
    world
        .driver
        .query(By::Css(element))
        .and_displayed()
        .with_text(PartialTextNeedle {
            search_pattern: content,
        })
        .first()
        .await
        .expect("Cannot find element");
    world.wait_for_delay().await;
}

#[then(expr = "the current page is {string}")]
pub async fn assert_page_url(world: &mut TestWorld, url: String) {
    let current_url = world.driver.current_url().await.unwrap();

    if url.starts_with('/') {
        let current_path = current_url.path();
        assert_eq!(
            current_path, url,
            "expected current path to be {url} but got {current_path}"
        )
    } else {
        let current_url = current_url.to_string();
        assert_eq!(
            current_url, url,
            "expected current path to be {url} but got {current_url}"
        )
    }
    world.wait_for_delay().await;
}

#[given("a user visits their e-mail inbox")]
#[when("a user visits their e-mail inbox")]
pub async fn visit_email_inbox(world: &mut TestWorld) {
    world.driver.goto("http://localhost:1080").await.unwrap();
    world.wait_for_delay().await;
}

#[given("the most recent e-mail is opened")]
#[when("the most recent e-mail is opened")]
pub async fn click_on_most_recent_email(world: &mut TestWorld) {
    world
        .driver
        .find(By::Css(
            "div.sidebar-emails-container > div.sidebar-scrollable-content > ul > li",
        ))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();

    world.wait_for_delay().await;
}
