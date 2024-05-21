use std::sync::Arc;

use axum::extract::State;

pub struct AppState {}

pub type ExtractState = State<Arc<AppState>>;

impl AppState {
    pub fn new() -> Self {
        Self {}
    }
}
