use cucumber::given;

use crate::{TestWorld, BASE_URL};

#[given("a user visits the sign up page")]
async fn visit_sign_up_page(world: &mut TestWorld) {
    world.driver.goto(format!("{}/auth/signup", BASE_URL)).await.unwrap();
}
