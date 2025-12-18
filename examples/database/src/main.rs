//! Database Example Demo
//!
//! Demonstrates creating and querying entities with SeaORM 2.0's builder pattern.

use sea_orm::{Database, EntityTrait};
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

    // Create a user using the builder pattern
    let user = user::ActiveModel::builder()
        .set_email("alice@example.com")
        .set_name("Alice")
        .set_created_at(chrono::Utc::now())
        .insert(&db)
        .await?;
    println!("Created user: {} (id={})", user.name, user.id);

    // Create some posts using the builder pattern
    for i in 1..=3 {
        let post = post::ActiveModel::builder()
            .set_title(format!("Post #{}", i))
            .set_content(format!("Content of post #{}", i))
            .set_author_id(user.id)
            .set_created_at(chrono::Utc::now())
            .insert(&db)
            .await?;
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
