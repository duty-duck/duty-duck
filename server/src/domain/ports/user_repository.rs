use crate::domain::entities::user::{CreateUserCommand, CreateUserError, User};

pub trait UserRepository {
    async fn create_user(&self, command: CreateUserCommand) -> Result<User, CreateUserError>;
}
