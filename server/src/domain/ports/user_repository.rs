use uuid::Uuid;

use crate::domain::entities::user::*;

pub trait UserRepository: Clone + Send + Sync + 'static {
    async fn get_user(&self, id: Uuid) -> anyhow::Result<Option<User>>;
    async fn create_user(&self, command: CreateUserCommand) -> Result<User, CreateUserError>;
    async fn update_user(&self, id: Uuid, command: UpdateUserCommand) -> Result<User, UpdateUserError>;
}
