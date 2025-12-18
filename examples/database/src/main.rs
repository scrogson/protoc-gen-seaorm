//! Database Example Demo
//!
//! Demonstrates creating and querying entities with SeaORM 2.0.

use sea_orm::{ActiveModelTrait, Database, EntityTrait, Set};
use seaorm_example::entity::example::{post, user};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to SQLite in-memory database
    let db = Database::connect("sqlite::memory:").await?;

    // Create tables using schema sync
    println!("Creating database schema...");
    db.get_schema_registry("seaorm_example::entity::*")
        .sync(&db)
        .await?;

    // Create a user
    let user = user::ActiveModel {
        email: Set("alice@example.com".to_string()),
        name: Set("Alice".to_string()),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    let user = user.insert(&db).await?;
    println!("Created user: {} (id={})", user.name, user.id);

    // Create some posts
    for i in 1..=3 {
        let post = post::ActiveModel {
            title: Set(format!("Post #{}", i)),
            content: Set(format!("Content of post #{}", i)),
            author_id: Set(user.id),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        };
        let post = post.insert(&db).await?;
        println!("Created post: {} (id={})", post.title, post.id);
    }

    // Query all users
    let users = user::Entity::find().all(&db).await?;
    println!("\nAll users:");
    for u in users {
        println!("  - {} <{}>", u.name, u.email);
    }

    // Query all posts
    let posts = post::Entity::find().all(&db).await?;
    println!("\nAll posts:");
    for p in posts {
        println!("  - {} (author_id={})", p.title, p.author_id);
    }

    Ok(())
}
