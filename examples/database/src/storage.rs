//! Storage implementation for UserService
//!
//! Demonstrates implementing the generated storage trait with SeaORM.
//!
//! The storage trait uses validated domain types (`GetUser`, `CreateUser`, `ListUsers`)
//! instead of raw proto request types. Validation happens in the gRPC handler layer
//! via `TryFrom`, so the storage layer receives pre-validated input.

use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder};

use crate::entity::example::prelude::*;
use crate::entity::example::user;
use crate::entity::example::users_storage::{StorageError, UsersStorage};
use crate::entity::example::{CreateUser, GetUser, ListUsers};

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
    async fn get_user(&self, request: GetUser) -> Result<User, StorageError> {
        // Domain type is already validated - id >= 1 guaranteed
        user::Entity::find_by_id(request.id)
            .one(&self.db)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("user with id {}", request.id)))
    }

    async fn create_user(&self, request: CreateUser) -> Result<User, StorageError> {
        // Domain type is already validated:
        // - email is valid email format
        // - name has length between 1 and 100
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

    async fn list_users(&self, request: ListUsers) -> Result<ListUsersResponse, StorageError> {
        // Domain type is already validated:
        // - page >= 0
        // - page_size between 1 and 100
        let page = request.page as u64;
        let page_size = request.page_size as u64;

        let paginator = user::Entity::find()
            .order_by_asc(user::Column::Id)
            .paginate(&self.db, page_size);

        let total = paginator.num_items().await? as i32;
        let users = paginator.fetch_page(page).await?;

        Ok(ListUsersResponse { users, total })
    }
}
