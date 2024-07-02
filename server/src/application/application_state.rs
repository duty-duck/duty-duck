use axum::extract::State;

use crate::infrastructure::adapters::{
    organization_repository_adapter::OrganizationRepositoryAdapter,
    user_repository_adapter::UserRepositoryAdapter,
};

pub type ExtractAppState = State<ApplicationState>;

#[derive(Clone)]
pub struct ApplicationState {
    pub adapters: Adapters,
}

#[derive(Clone)]
pub struct Adapters {
    pub user_repository: UserRepositoryAdapter,
    pub organization_repository: OrganizationRepositoryAdapter,
}
