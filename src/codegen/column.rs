//! Column attribute generation for SeaORM entities
//!
//! This module generates the #[sea_orm(...)] attributes for entity fields.

use crate::options::seaorm::FieldOptions;
use crate::types::MappedType;

/// Generate SeaORM column attributes for a field
pub struct ColumnAttributes {
    /// The #[sea_orm(...)] attribute contents
    pub attributes: Vec<String>,
    /// The Rust type for this column
    pub rust_type: String,
}

/// Generate column attributes from field options and mapped type
pub fn generate_attributes(
    _field_options: Option<&FieldOptions>,
    mapped_type: &MappedType,
    is_nullable: bool,
) -> ColumnAttributes {
    let attributes = Vec::new();
    let mut rust_type = mapped_type.rust_type.clone();

    // TODO: Add attribute generation based on field_options
    // - primary_key
    // - auto_increment
    // - unique
    // - column_name
    // - column_type

    if is_nullable && !rust_type.starts_with("Option<") {
        rust_type = format!("Option<{}>", rust_type);
    }

    ColumnAttributes {
        attributes,
        rust_type,
    }
}
