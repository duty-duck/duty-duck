use std::time::Duration;

use cucumber::*;
use thirtyfour::prelude::*;
use thirtyfour::stringmatch::Needle;

use crate::TestWorld;

#[derive(Clone)]
pub struct PartialTextNeedle { search_pattern: String }

impl Needle for PartialTextNeedle {
    fn is_match(&self, haystack: &str) -> bool {
        haystack.contains(&self.search_pattern)
    }
}

#[when(expr = "the {string} field is filled with {string}")]
async fn fill_field(world: &mut TestWorld, field_name: String, field_value: String) {
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
}

#[when("the form is submitted")]
async fn submit_form(world: &mut TestWorld) {
    let submit_button = world
        .driver
        .query(By::Css("form button[type=\"submit\"]"))
        .first()
        .await
        .expect("cannot find form submit button");
    submit_button.click().await.unwrap();
}

#[then(expr = "the {string} element should show {string}")]
async fn assert_page_showing_text(world: &mut TestWorld, element: String, content: String) {
    world
        .driver
        .query(By::Css(element))
        .and_displayed()
        .with_text(PartialTextNeedle { search_pattern: content })
        .wait(Duration::from_secs(5), Duration::from_millis(100))
        .first()
        .await
        .expect("Cannot find element");
}
