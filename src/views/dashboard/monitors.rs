use askama_axum::Template;
use entity::{http_monitor, user_account};
use serde::Deserialize;

use crate::session::CSRFToken;

#[derive(Deserialize)]
pub struct CreateMonitorForm {
    pub url: String,
    #[serde(deserialize_with = "serde_aux::prelude::deserialize_number_from_string")]
    pub interval_seconds: u64,
}

#[derive(Template)]
#[template(path = "dashboard/monitors/index.html")]
pub struct MonitorsIndex {
    pub user: user_account::Model,
    pub monitors: Vec<http_monitor::Model>,
}

#[derive(Template)]
#[template(path = "dashboard/monitors/new.html")]
pub struct NewMonitorForm<'e> {
    pub user: user_account::Model,
    pub form: CreateMonitorForm,
    pub csrf_token: CSRFToken,
    pub error: Option<&'e str>
}
