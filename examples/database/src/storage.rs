//! Storage implementation for UserService
//!
//! Demonstrates implementing the generated storage trait with SeaORM.

use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder};

use crate::entity::example::prelude::*;
use crate::entity::example::user;
use crate::entity::example::users_storage::{StorageError, UsersStorage};

/// SeaORM-backed implementation of UsersStorage
pub struct SeaOrmUserStorage {
    db: DatabaseConnection,
}

impl SeaOrmUserStorage {
    /// Create a new storage instance with the given database connection
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl UsersStorage for SeaOrmUserStorage {
    async fn get_user(&self, request: GetUserRequest) -> Result<User, StorageError> {
        user::Entity::find_by_id(request.id)
            .one(&self.db)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("user with id {}", request.id)))
    }

    async fn create_user(&self, request: CreateUserRequest) -> Result<User, StorageError> {
        if request.email.is_empty() {
            return Err(StorageError::InvalidArgument(
                "email cannot be empty".into(),
            ));
        }
        if request.name.is_empty() {
            return Err(StorageError::InvalidArgument("name cannot be empty".into()));
        }

        let now = chrono::Utc::now();
        let user = user::ActiveModel::builder()
            .set_email(&request.email)
            .set_name(&request.name)
            .set_created_at(now)
            .set_updated_at(now)
            .insert(&self.db)
            .await?;

        Ok(user.into())
    }

    async fn list_users(
        &self,
        request: ListUsersRequest,
    ) -> Result<ListUsersResponse, StorageError> {
        let page = request.page.max(0) as u64;
        let page_size = request.page_size.clamp(1, 100) as u64;

        let paginator = user::Entity::find()
            .order_by_asc(user::Column::Id)
            .paginate(&self.db, page_size);

        let total = paginator.num_items().await? as i32;
        let users = paginator.fetch_page(page).await?;

        Ok(ListUsersResponse { users, total })
    }
}
