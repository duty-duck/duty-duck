use askama_axum::Template;
use entity::user_account;


#[derive(Template)]
#[template(path = "dashboard/index.html")]
pub struct DashboardHome {
    pub user: user_account::Model
}
