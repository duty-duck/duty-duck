use askama_axum::Template;
use entity::user_account;

pub mod monitors;

#[derive(Template)]
#[template(path = "dashboard/index.html")]
pub struct DashboardHome {
    pub user: user_account::Model,
}
