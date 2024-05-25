use askama_axum::*;
use entity::user_account;
use crate::views::filters;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub current_user: Option<user_account::Model>,
}

#[derive(Template)]
#[template(path = "pricing.html")]
pub struct PricingTemplate {
    pub current_user: Option<user_account::Model>,
}
