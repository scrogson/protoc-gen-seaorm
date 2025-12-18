//! Database Integration Tests
//!
//! Tests the generated SeaORM entities with actual database operations.

use sea_orm::{
    ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use seaorm_example::entity::example::{post, user};

/// Set up an in-memory SQLite database with schema
async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();

    // Use schema sync to create tables from entity definitions
    db.get_schema_registry("seaorm_example::entity::*")
        .sync(&db)
        .await
        .unwrap();

    db
}

// =============================================================================
// Basic CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_user() {
    let db = setup_db().await;

    let result = user::ActiveModel::builder()
        .set_email("test@example.com")
        .set_name("Test User")
        .set_created_at(chrono::Utc::now())
        .insert(&db)
        .await
        .unwrap();

    assert_eq!(result.email, "test@example.com");
    assert_eq!(result.name, "Test User");
    assert!(result.id > 0);
}

#[tokio::test]
async fn test_find_user_by_id() {
    let db = setup_db().await;

    // Create a user
    let created = user::ActiveModel::builder()
        .set_email("findme@example.com")
        .set_name("Find Me")
        .set_created_at(chrono::Utc::now())
        .insert(&db)
        .await
        .unwrap();

    // Find by ID
    let found = user::Entity::find_by_id(created.id).one(&db).await.unwrap();

    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.email, "findme@example.com");
}

#[tokio::test]
async fn test_find_user_by_email() {
    let db = setup_db().await;

    // Create a user
    user::ActiveModel::builder()
        .set_email("unique@example.com")
        .set_name("Unique User")
        .set_created_at(chrono::Utc::now())
        .insert(&db)
        .await
        .unwrap();

    // Find by email
    let found = user::Entity::find()
        .filter(user::Column::Email.eq("unique@example.com"))
        .one(&db)
        .await
        .unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Unique User");
}

#[tokio::test]
async fn test_update_user() {
    let db = setup_db().await;

    // Create a user
    let created = user::ActiveModel::builder()
        .set_email("update@example.com")
        .set_name("Original Name")
        .set_created_at(chrono::Utc::now())
        .insert(&db)
        .await
        .unwrap();

    // Update the user - build a new ActiveModel with the ID set
    let updated = user::ActiveModel::builder()
        .set_id(created.id)
        .set_name("Updated Name")
        .update(&db)
        .await
        .unwrap();

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.email, "update@example.com");
}

#[tokio::test]
async fn test_delete_user() {
    let db = setup_db().await;

    // Create a user
    let created = user::ActiveModel::builder()
        .set_email("delete@example.com")
        .set_name("Delete Me")
        .set_created_at(chrono::Utc::now())
        .insert(&db)
        .await
        .unwrap();
    let user_id = created.id;

    // Delete the user by ID
    user::Entity::delete_by_id(user_id).exec(&db).await.unwrap();

    // Verify deleted
    let found = user::Entity::find_by_id(user_id).one(&db).await.unwrap();
    assert!(found.is_none());
}

// =============================================================================
// Relation Tests
// =============================================================================

#[tokio::test]
async fn test_create_post_for_user() {
    let db = setup_db().await;

    // Create a user
    let author = user::ActiveModel::builder()
        .set_email("author@example.com")
        .set_name("Author")
        .set_created_at(chrono::Utc::now())
        .insert(&db)
        .await
        .unwrap();

    // Create a post for the user
    let created_post = post::ActiveModel::builder()
        .set_title("My First Post")
        .set_content("Hello, world!")
        .set_author_id(author.id)
        .set_created_at(chrono::Utc::now())
        .insert(&db)
        .await
        .unwrap();

    assert_eq!(created_post.title, "My First Post");
    assert_eq!(created_post.author_id, author.id);
}

#[tokio::test]
async fn test_find_posts_by_author() {
    let db = setup_db().await;

    // Create a user
    let author = user::ActiveModel::builder()
        .set_email("prolific@example.com")
        .set_name("Prolific Writer")
        .set_created_at(chrono::Utc::now())
        .insert(&db)
        .await
        .unwrap();

    // Create multiple posts
    for i in 1..=3 {
        post::ActiveModel::builder()
            .set_title(format!("Post {}", i))
            .set_content(format!("Content {}", i))
            .set_author_id(author.id)
            .set_created_at(chrono::Utc::now())
            .insert(&db)
            .await
            .unwrap();
    }

    // Find all posts by this author
    let posts = post::Entity::find()
        .filter(post::Column::AuthorId.eq(author.id))
        .all(&db)
        .await
        .unwrap();

    assert_eq!(posts.len(), 3);
}

// =============================================================================
// Query Tests
// =============================================================================

#[tokio::test]
async fn test_find_all_users() {
    let db = setup_db().await;

    // Create multiple users
    for i in 1..=5 {
        user::ActiveModel::builder()
            .set_email(format!("user{}@example.com", i))
            .set_name(format!("User {}", i))
            .set_created_at(chrono::Utc::now())
            .insert(&db)
            .await
            .unwrap();
    }

    // Find all
    let users = user::Entity::find().all(&db).await.unwrap();
    assert_eq!(users.len(), 5);
}

#[tokio::test]
async fn test_filter_users_by_name() {
    let db = setup_db().await;

    // Create users with different names
    for name in ["Alice", "Bob", "Alice Jr", "Charlie"] {
        user::ActiveModel::builder()
            .set_email(format!(
                "{}@example.com",
                name.to_lowercase().replace(' ', "")
            ))
            .set_name(name)
            .set_created_at(chrono::Utc::now())
            .insert(&db)
            .await
            .unwrap();
    }

    // Filter by name containing "Alice"
    let alices = user::Entity::find()
        .filter(user::Column::Name.contains("Alice"))
        .all(&db)
        .await
        .unwrap();

    assert_eq!(alices.len(), 2);
}

#[tokio::test]
async fn test_order_users_by_name() {
    let db = setup_db().await;

    // Create users in random order
    for name in ["Charlie", "Alice", "Bob"] {
        user::ActiveModel::builder()
            .set_email(format!("{}@example.com", name.to_lowercase()))
            .set_name(name)
            .set_created_at(chrono::Utc::now())
            .insert(&db)
            .await
            .unwrap();
    }

    // Order by name ascending
    let users = user::Entity::find()
        .order_by_asc(user::Column::Name)
        .all(&db)
        .await
        .unwrap();

    assert_eq!(users.len(), 3);
    assert_eq!(users[0].name, "Alice");
    assert_eq!(users[1].name, "Bob");
    assert_eq!(users[2].name, "Charlie");
}

#[tokio::test]
async fn test_paginate_users() {
    let db = setup_db().await;

    // Create 10 users
    for i in 1..=10 {
        user::ActiveModel::builder()
            .set_email(format!("page{}@example.com", i))
            .set_name(format!("Page User {}", i))
            .set_created_at(chrono::Utc::now())
            .insert(&db)
            .await
            .unwrap();
    }

    // Get first page (5 items)
    let page1 = user::Entity::find()
        .order_by_asc(user::Column::Id)
        .limit(5)
        .all(&db)
        .await
        .unwrap();

    assert_eq!(page1.len(), 5);

    // Get second page (5 items, offset 5)
    let page2 = user::Entity::find()
        .order_by_asc(user::Column::Id)
        .offset(5)
        .limit(5)
        .all(&db)
        .await
        .unwrap();

    assert_eq!(page2.len(), 5);

    // Verify no overlap
    let page1_ids: Vec<_> = page1.iter().map(|u| u.id).collect();
    let page2_ids: Vec<_> = page2.iter().map(|u| u.id).collect();
    for id in &page2_ids {
        assert!(!page1_ids.contains(id));
    }
}
