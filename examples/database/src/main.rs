//! Database Example Demo
//!
//! Demonstrates using the generated UsersStorage trait with SeaORM.

use sea_orm::Database;
use seaorm_example::entity::example::prelude::*;
use seaorm_example::entity::example::UsersStorage;
use seaorm_example::SeaOrmUserStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to SQLite in-memory database
    let db = Database::connect("sqlite::memory:").await?;

    // Create tables using schema sync
    println!("Creating database schema...");
    db.get_schema_registry("seaorm_example::entity::*")
        .sync(&db)
        .await?;

    // Create the storage implementation
    let storage = SeaOrmUserStorage::new(db);

    // Create users via the storage trait
    println!("\nCreating users via UserServiceStorage...");
    let alice = storage
        .create_user(CreateUserRequest {
            email: "alice@example.com".into(),
            name: "Alice".into(),
        })
        .await?;
    println!("Created user: {} (id={})", alice.name, alice.id);

    let bob = storage
        .create_user(CreateUserRequest {
            email: "bob@example.com".into(),
            name: "Bob".into(),
        })
        .await?;
    println!("Created user: {} (id={})", bob.name, bob.id);

    // Get a user by ID
    println!("\nGetting user by ID...");
    let user = storage.get_user(GetUserRequest { id: alice.id }).await?;
    println!("Found user: {} <{}>", user.name, user.email);

    // List users with pagination
    println!("\nListing users (page 0, size 10)...");
    let response = storage
        .list_users(ListUsersRequest {
            page: 0,
            page_size: 10,
        })
        .await?;
    println!("Total users: {}", response.total);
    for u in response.users {
        println!("  - {} <{}>", u.name, u.email);
    }

    // Try to get a non-existent user
    println!("\nTrying to get non-existent user (id=999)...");
    match storage.get_user(GetUserRequest { id: 999 }).await {
        Ok(u) => println!("Found user: {}", u.name),
        Err(e) => println!("Error (expected): {}", e),
    }

    Ok(())
}
