use crate::views::filters;
use askama_axum::*;
use entity::user_account;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
}

#[derive(Template)]
#[template(path = "pricing.html")]
pub struct PricingTemplate {
}
