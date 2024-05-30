use serde::Deserialize;

pub mod auth;
pub mod filters;
pub mod public;
pub mod dashboard;

#[derive(Deserialize)]
pub struct Pagination {
    pub page: u64,
    pub per_page: u64,
}