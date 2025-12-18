//! Relation generation for SeaORM entities
//!
//! This module generates HasOne, HasMany, and BelongsTo relations for SeaORM 2.0.
//!
//! SeaORM 2.0 uses the `DeriveRelation` macro with enum variants to define relations.

use crate::options::seaorm::FieldOptions;
use heck::{ToSnakeCase, ToUpperCamelCase};

/// Represents a generated relation
#[derive(Debug, Clone)]
pub struct GeneratedRelation {
    /// The enum variant name (e.g., "Posts", "Author")
    pub variant_name: String,
    /// The relation type (HasOne, HasMany, BelongsTo)
    pub relation_type: RelationType,
    /// Target entity module path (e.g., "super::post")
    pub target_entity: String,
    /// For BelongsTo: the local foreign key column
    pub from_column: Option<String>,
    /// For BelongsTo: the remote primary key column
    pub to_column: Option<String>,
    /// For many-to-many: the junction table
    pub via_table: Option<String>,
}

/// Type of relation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationType {
    /// One-to-one relationship
    HasOne,
    /// One-to-many relationship
    HasMany,
    /// Foreign key relationship (many-to-one)
    BelongsTo,
}

impl RelationType {
    /// Get the SeaORM attribute name for this relation type
    pub fn attribute_name(&self) -> &'static str {
        match self {
            RelationType::HasOne => "has_one",
            RelationType::HasMany => "has_many",
            RelationType::BelongsTo => "belongs_to",
        }
    }
}

/// Generate a relation from field options
///
/// Returns None if the field doesn't define a relation
pub fn generate_relation(
    field_name: &str,
    field_options: &FieldOptions,
) -> Option<GeneratedRelation> {
    // Check for has_one
    if !field_options.has_one.is_empty() {
        let target = &field_options.has_one;
        return Some(GeneratedRelation {
            variant_name: target.to_upper_camel_case(),
            relation_type: RelationType::HasOne,
            target_entity: format!("super::{}::Entity", target.to_snake_case()),
            from_column: None,
            to_column: None,
            via_table: None,
        });
    }

    // Check for has_many
    if !field_options.has_many.is_empty() {
        let target = &field_options.has_many;

        // Check for many-to-many via junction table
        if !field_options.has_many_via.is_empty() {
            return Some(GeneratedRelation {
                variant_name: target.to_upper_camel_case(),
                relation_type: RelationType::HasMany,
                target_entity: format!("super::{}::Entity", target.to_snake_case()),
                from_column: None,
                to_column: None,
                via_table: Some(field_options.has_many_via.clone()),
            });
        }

        return Some(GeneratedRelation {
            variant_name: target.to_upper_camel_case(),
            relation_type: RelationType::HasMany,
            target_entity: format!("super::{}::Entity", target.to_snake_case()),
            from_column: None,
            to_column: None,
            via_table: None,
        });
    }

    // Check for belongs_to
    if !field_options.belongs_to.is_empty() {
        let target = &field_options.belongs_to;

        // Get from/to columns, with defaults
        let from_column = if field_options.belongs_to_from.is_empty() {
            format!("{}_id", target.to_snake_case())
        } else {
            field_options.belongs_to_from.clone()
        };

        let to_column = if field_options.belongs_to_to.is_empty() {
            "id".to_string()
        } else {
            field_options.belongs_to_to.clone()
        };

        return Some(GeneratedRelation {
            variant_name: target.to_upper_camel_case(),
            relation_type: RelationType::BelongsTo,
            target_entity: format!("super::{}::Entity", target.to_snake_case()),
            from_column: Some(from_column),
            to_column: Some(to_column),
            via_table: None,
        });
    }

    // No relation defined
    let _ = field_name; // Suppress unused warning
    None
}

/// Generate the #[sea_orm(...)] attribute for a relation
pub fn generate_relation_attribute(relation: &GeneratedRelation) -> String {
    match relation.relation_type {
        RelationType::HasOne => {
            format!(
                "has_one = \"{}\"",
                relation.target_entity
            )
        }
        RelationType::HasMany => {
            if let Some(ref via) = relation.via_table {
                format!(
                    "has_many = \"{}\", via = \"{}\"",
                    relation.target_entity, via
                )
            } else {
                format!(
                    "has_many = \"{}\"",
                    relation.target_entity
                )
            }
        }
        RelationType::BelongsTo => {
            let from = relation.from_column.as_deref().unwrap_or("id");
            let to = relation.to_column.as_deref().unwrap_or("id");
            format!(
                "belongs_to = \"{}\", from = \"Column::{}\", to = \"super::{}::Column::{}\"",
                relation.target_entity,
                from.to_upper_camel_case(),
                relation.target_entity.replace("super::", "").replace("::Entity", ""),
                to.to_upper_camel_case()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_has_many_relation() {
        let opts = FieldOptions {
            has_many: "post".to_string(),
            ..Default::default()
        };
        let rel = generate_relation("posts", &opts).unwrap();
        assert_eq!(rel.variant_name, "Post");
        assert_eq!(rel.relation_type, RelationType::HasMany);
        assert_eq!(rel.target_entity, "super::post::Entity");
    }

    #[test]
    fn test_generate_belongs_to_relation() {
        let opts = FieldOptions {
            belongs_to: "user".to_string(),
            belongs_to_from: "user_id".to_string(),
            belongs_to_to: "id".to_string(),
            ..Default::default()
        };
        let rel = generate_relation("user", &opts).unwrap();
        assert_eq!(rel.variant_name, "User");
        assert_eq!(rel.relation_type, RelationType::BelongsTo);
        assert_eq!(rel.from_column, Some("user_id".to_string()));
        assert_eq!(rel.to_column, Some("id".to_string()));
    }

    #[test]
    fn test_no_relation() {
        let opts = FieldOptions::default();
        assert!(generate_relation("field", &opts).is_none());
    }
}
