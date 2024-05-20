use std::sync::Arc;

use axum::extract::State;
use tera::Tera;

pub struct AppState {
    pub templates: Tera,
}

pub type ExtractState = State<Arc<AppState>>;

impl AppState {
    pub fn new() -> Self {
        Self {
            templates: Tera::new("templates/**/*.tera").unwrap(),
        }
    }
}
