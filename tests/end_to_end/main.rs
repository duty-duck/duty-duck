
#[derive(Debug, Default, cucumber::World)]
pub struct World {}

#[tokio::main]
async fn main() {
    <World as cucumber::World>::run("tests/end_to_end/features").await;
}