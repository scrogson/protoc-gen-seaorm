//! Database Integration Tests
//!
//! Tests the generated SeaORM entities with actual database operations.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, Database,
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

    let user = user::ActiveModel {
        email: Set("test@example.com".to_string()),
        name: Set("Test User".to_string()),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    let result = user.insert(&db).await.unwrap();

    assert_eq!(result.email, "test@example.com");
    assert_eq!(result.name, "Test User");
    assert!(result.id > 0);
}

#[tokio::test]
async fn test_find_user_by_id() {
    let db = setup_db().await;

    // Create a user
    let user = user::ActiveModel {
        email: Set("findme@example.com".to_string()),
        name: Set("Find Me".to_string()),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    let created = user.insert(&db).await.unwrap();

    // Find by ID
    let found = user::Entity::find_by_id(created.id)
        .one(&db)
        .await
        .unwrap();

    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.email, "findme@example.com");
}

#[tokio::test]
async fn test_find_user_by_email() {
    let db = setup_db().await;

    // Create a user
    let user = user::ActiveModel {
        email: Set("unique@example.com".to_string()),
        name: Set("Unique User".to_string()),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    user.insert(&db).await.unwrap();

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
    let user = user::ActiveModel {
        email: Set("update@example.com".to_string()),
        name: Set("Original Name".to_string()),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    let created = user.insert(&db).await.unwrap();

    // Update the user
    let mut active: user::ActiveModel = created.into();
    active.name = Set("Updated Name".to_string());
    let updated = active.update(&db).await.unwrap();

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.email, "update@example.com");
}

#[tokio::test]
async fn test_delete_user() {
    let db = setup_db().await;

    // Create a user
    let user = user::ActiveModel {
        email: Set("delete@example.com".to_string()),
        name: Set("Delete Me".to_string()),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    let created = user.insert(&db).await.unwrap();
    let user_id = created.id;

    // Delete the user
    let active: user::ActiveModel = created.into();
    active.delete(&db).await.unwrap();

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
    let user = user::ActiveModel {
        email: Set("author@example.com".to_string()),
        name: Set("Author".to_string()),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    let author = user.insert(&db).await.unwrap();

    // Create a post for the user
    let post = post::ActiveModel {
        title: Set("My First Post".to_string()),
        content: Set("Hello, world!".to_string()),
        author_id: Set(author.id),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    let created_post = post.insert(&db).await.unwrap();

    assert_eq!(created_post.title, "My First Post");
    assert_eq!(created_post.author_id, author.id);
}

#[tokio::test]
async fn test_find_posts_by_author() {
    let db = setup_db().await;

    // Create a user
    let user = user::ActiveModel {
        email: Set("prolific@example.com".to_string()),
        name: Set("Prolific Writer".to_string()),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    let author = user.insert(&db).await.unwrap();

    // Create multiple posts
    for i in 1..=3 {
        let post = post::ActiveModel {
            title: Set(format!("Post {}", i)),
            content: Set(format!("Content {}", i)),
            author_id: Set(author.id),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        };
        post.insert(&db).await.unwrap();
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
        let user = user::ActiveModel {
            email: Set(format!("user{}@example.com", i)),
            name: Set(format!("User {}", i)),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        };
        user.insert(&db).await.unwrap();
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
        let user = user::ActiveModel {
            email: Set(format!("{}@example.com", name.to_lowercase().replace(' ', ""))),
            name: Set(name.to_string()),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        };
        user.insert(&db).await.unwrap();
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
        let user = user::ActiveModel {
            email: Set(format!("{}@example.com", name.to_lowercase())),
            name: Set(name.to_string()),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        };
        user.insert(&db).await.unwrap();
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
        let user = user::ActiveModel {
            email: Set(format!("page{}@example.com", i)),
            name: Set(format!("Page User {}", i)),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        };
        user.insert(&db).await.unwrap();
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
