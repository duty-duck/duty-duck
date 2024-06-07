use serde::Deserialize;

pub mod auth;
pub mod dashboard;
pub mod filters;
pub mod public;

#[derive(Deserialize)]
pub struct Pagination {
    pub page: u64,
    pub per_page: u64,
}
