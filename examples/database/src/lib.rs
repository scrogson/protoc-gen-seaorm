//! SeaORM Database Example
//!
//! This example demonstrates using generated SeaORM entities from protobuf
//! definitions with schema-sync for automatic table creation.
//!
//! ## Generating Entities
//!
//! ```bash
//! # Build the protoc-gen-seaorm plugin first (from root)
//! cargo build --release
//!
//! # Generate entities from proto (from this directory)
//! buf generate
//! ```

/// Generated entity modules from protobuf definitions
pub mod entity {
    /// Example entities (User, Post) and storage traits
    pub mod example {
        pub mod post;
        pub mod prelude;
        pub mod user;
        pub mod users_storage;

        pub use post::Entity as Post;
        pub use user::Entity as User;
        pub use users_storage::{StorageError, UsersStorage};
    }

    pub use example::*;
}

pub use entity::*;

/// Storage implementation for the UserService trait
pub mod storage;
pub use storage::SeaOrmUserStorage;
