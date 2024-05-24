use cucumber::*;
use thirtyfour::prelude::*;

use crate::{TestWorld, BASE_URL};

use super::common::{assert_page_showing_text, assert_page_url};

#[given("a user visits the sign up page")]
async fn visit_sign_up_page(world: &mut TestWorld) {
    world
        .driver
        .goto(format!("{}/auth/signup", BASE_URL))
        .await
        .unwrap();
    world.wait_for_delay().await;
}

/// A steps that verifies the current browser page is the signup confirmation page,
/// and the registration was successful
#[given("a user successfully signed up")]
async fn signed_up_successfully(world: &mut TestWorld) {
    assert_page_url(world, "/auth/signup".to_string()).await;
    assert_page_showing_text(
        world,
        "#auth-page-container".to_string(),
        "A confirmation e-mail has been sent to your inbox.".to_string(),
    )
    .await;

    world.wait_for_delay().await;
}

#[when("a confirmation link is clicked")]
async fn click_on_confirmation_link(world: &mut TestWorld) {
    // Wait for iframe to be available
    world.driver.query(By::Tag("iframe")).first().await.unwrap();

    // Swith to iframe
    world.driver.enter_frame(0).await.unwrap();

    // Get the link destination
    let link_dest = world
        .driver
        .query(By::Id("confirmation-link"))
        .first()
        .await
        .unwrap()
        .attr("href")
        .await
        .unwrap()
        .unwrap();

    // Switch back to the default frame
    world.driver.enter_default_frame().await.unwrap();

    // Visit the confirmation page
    world.driver.goto(link_dest).await.unwrap();

    world.wait_for_delay().await;
}

#[then("the registration is confirmed")]
async fn the_registration_is_confirmed(world: &mut TestWorld) {
    assert_page_showing_text(world, "div.card".to_string(), "You've made it!".to_string()).await;
}
