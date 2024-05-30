use askama_axum::Template;
use entity::user_account;

#[derive(Template)]
#[template(path = "dashboard/index.html")]
pub struct DashboardHome {
    pub user: user_account::Model
}

#[derive(Template)]
#[template(path = "dashboard/monitors/index.html")]
pub struct MonitorsIndex {
    pub user: user_account::Model
}
