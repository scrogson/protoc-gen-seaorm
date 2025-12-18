//! Relation generation for SeaORM entities
//!
//! This module generates HasOne, HasMany, and BelongsTo relations for SeaORM 2.0.
//!
//! SeaORM 2.0 uses the `DeriveRelation` macro with enum variants to define relations.

use crate::options::seaorm::{FieldOptions, RelationDef, RelationType};
use heck::{ToSnakeCase, ToUpperCamelCase};

/// Represents a generated relation
#[derive(Debug, Clone)]
pub struct GeneratedRelation {
    /// The enum variant name (e.g., "Posts", "Author")
    pub variant_name: String,
    /// The relation type (HasOne, HasMany, BelongsTo)
    pub relation_type: SeaOrmRelationType,
    /// Target entity module path (e.g., "super::post")
    pub target_entity: String,
    /// For BelongsTo: the local foreign key column
    pub from_column: Option<String>,
    /// For BelongsTo: the remote primary key column
    pub to_column: Option<String>,
    /// For many-to-many: the junction table
    pub via_table: Option<String>,
}

/// Type of relation for SeaORM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeaOrmRelationType {
    /// One-to-one relationship
    HasOne,
    /// One-to-many relationship
    HasMany,
    /// Foreign key relationship (many-to-one)
    BelongsTo,
    /// Many-to-many relationship (via junction table)
    ManyToMany,
}

impl SeaOrmRelationType {
    /// Get the SeaORM attribute name for this relation type
    pub fn attribute_name(&self) -> &'static str {
        match self {
            SeaOrmRelationType::HasOne => "has_one",
            SeaOrmRelationType::HasMany => "has_many",
            SeaOrmRelationType::BelongsTo => "belongs_to",
            SeaOrmRelationType::ManyToMany => "many_to_many",
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
            relation_type: SeaOrmRelationType::HasOne,
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
                relation_type: SeaOrmRelationType::HasMany,
                target_entity: format!("super::{}::Entity", target.to_snake_case()),
                from_column: None,
                to_column: None,
                via_table: Some(field_options.has_many_via.clone()),
            });
        }

        return Some(GeneratedRelation {
            variant_name: target.to_upper_camel_case(),
            relation_type: SeaOrmRelationType::HasMany,
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
            relation_type: SeaOrmRelationType::BelongsTo,
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

/// Generate a relation from a message-level RelationDef
///
/// This supports the cleaner message-level relation syntax
pub fn generate_relation_from_def(rel_def: &RelationDef) -> Option<GeneratedRelation> {
    if rel_def.name.is_empty() || rel_def.related.is_empty() {
        return None;
    }

    let rel_type = RelationType::try_from(rel_def.r#type).unwrap_or(RelationType::Unspecified);

    let relation_type = match rel_type {
        RelationType::BelongsTo => SeaOrmRelationType::BelongsTo,
        RelationType::HasOne => SeaOrmRelationType::HasOne,
        RelationType::HasMany => SeaOrmRelationType::HasMany,
        RelationType::ManyToMany => SeaOrmRelationType::ManyToMany,
        RelationType::Unspecified => return None,
    };

    let target_entity = format!("super::{}::Entity", rel_def.related.to_snake_case());

    // Determine from/to columns based on relation type
    let (from_column, to_column) = match relation_type {
        SeaOrmRelationType::BelongsTo => {
            let from = if rel_def.foreign_key.is_empty() {
                format!("{}_id", rel_def.related.to_snake_case())
            } else {
                rel_def.foreign_key.clone()
            };
            let to = if rel_def.references.is_empty() {
                "id".to_string()
            } else {
                rel_def.references.clone()
            };
            (Some(from), Some(to))
        }
        SeaOrmRelationType::HasOne | SeaOrmRelationType::HasMany => {
            // For has_one/has_many, foreign_key is on the related table
            let fk = if !rel_def.foreign_key.is_empty() {
                Some(rel_def.foreign_key.clone())
            } else {
                None
            };
            let refs = if !rel_def.references.is_empty() {
                Some(rel_def.references.clone())
            } else {
                None
            };
            (fk, refs)
        }
        SeaOrmRelationType::ManyToMany => {
            // For many-to-many, we use the junction table (through)
            // foreign_key and references can optionally specify the join columns
            let fk = if !rel_def.foreign_key.is_empty() {
                Some(rel_def.foreign_key.clone())
            } else {
                None
            };
            let refs = if !rel_def.references.is_empty() {
                Some(rel_def.references.clone())
            } else {
                None
            };
            (fk, refs)
        }
    };

    let via_table = if !rel_def.through.is_empty() {
        Some(rel_def.through.clone())
    } else {
        None
    };

    Some(GeneratedRelation {
        variant_name: rel_def.name.to_upper_camel_case(),
        relation_type,
        target_entity,
        from_column,
        to_column,
        via_table,
    })
}

/// Find the reverse relation name for a self-referential relation
///
/// Given a list of relations and a self-referential relation, find its reverse pair.
/// For example, if we have `parent` (belongs_to) and `replies` (has_many) both pointing
/// to the same entity, they are a reverse pair.
fn find_self_ref_reverse(
    relations: &[RelationDef],
    current_rel: &RelationDef,
    current_entity: &str,
) -> Option<String> {
    let current_type =
        RelationType::try_from(current_rel.r#type).unwrap_or(RelationType::Unspecified);
    let is_self_ref = current_rel.related.to_snake_case() == current_entity.to_snake_case();

    if !is_self_ref {
        return None;
    }

    // Find a relation that is:
    // 1. Self-referential (same entity)
    // 2. Different name from current
    // 3. Complementary type (belongs_to <-> has_many, or has_one <-> has_many)
    // 4. Uses the same foreign key (if specified)
    for rel in relations {
        if rel.name == current_rel.name {
            continue; // Skip self
        }

        let rel_type = RelationType::try_from(rel.r#type).unwrap_or(RelationType::Unspecified);
        let rel_is_self_ref = rel.related.to_snake_case() == current_entity.to_snake_case();

        if !rel_is_self_ref {
            continue;
        }

        // Check if they are complementary types
        let is_complementary = match (current_type, rel_type) {
            (RelationType::BelongsTo, RelationType::HasMany)
            | (RelationType::BelongsTo, RelationType::HasOne)
            | (RelationType::HasMany, RelationType::BelongsTo)
            | (RelationType::HasOne, RelationType::BelongsTo) => true,
            _ => false,
        };

        if is_complementary {
            // Check if they use the same foreign key
            let same_fk = if !current_rel.foreign_key.is_empty() && !rel.foreign_key.is_empty() {
                current_rel.foreign_key == rel.foreign_key
            } else {
                // If FK not specified, assume they match
                true
            };

            if same_fk {
                return Some(rel.name.to_upper_camel_case());
            }
        }
    }

    None
}

/// Generate all relation fields for a message, properly handling self-referential pairs
pub fn generate_relation_fields(
    relations: &[RelationDef],
    current_entity: &str,
) -> Vec<proc_macro2::TokenStream> {
    relations
        .iter()
        .filter_map(|rel| {
            let reverse = find_self_ref_reverse(relations, rel, current_entity);
            generate_relation_field_with_reverse(rel, current_entity, reverse.as_deref())
        })
        .collect()
}

/// Generate a relation field for the SeaORM 2.0 dense format
///
/// This generates a field like:
/// ```ignore
/// #[sea_orm(has_many)]
/// pub posts: HasMany<super::post::Entity>,
/// ```
///
/// For self-referential relations, adds `self_ref`, `relation_enum`, and `relation_reverse` attributes.
pub fn generate_relation_field(
    rel_def: &RelationDef,
    current_entity: &str,
) -> Option<proc_macro2::TokenStream> {
    generate_relation_field_with_reverse(rel_def, current_entity, None)
}

/// Generate a relation field with optional relation_reverse for self-referential relations
fn generate_relation_field_with_reverse(
    rel_def: &RelationDef,
    current_entity: &str,
    relation_reverse: Option<&str>,
) -> Option<proc_macro2::TokenStream> {
    use quote::{format_ident, quote};

    if rel_def.name.is_empty() || rel_def.related.is_empty() {
        return None;
    }

    let rel_type = RelationType::try_from(rel_def.r#type).unwrap_or(RelationType::Unspecified);

    let field_name = format_ident!("{}", rel_def.name.to_snake_case());
    let relation_enum_name = rel_def.name.to_upper_camel_case();

    // Check if this is a self-referential relation
    let is_self_ref = rel_def.related.to_snake_case() == current_entity.to_snake_case();

    // For self-ref, use Entity directly; otherwise use super::module::Entity
    let target_entity: syn::Type = if is_self_ref {
        syn::parse_quote!(Entity)
    } else {
        syn::parse_str(&format!("super::{}::Entity", rel_def.related.to_snake_case()))
            .unwrap_or_else(|_| syn::parse_quote!(Entity))
    };

    match rel_type {
        RelationType::HasOne => {
            if is_self_ref {
                if let Some(reverse) = relation_reverse {
                    Some(quote! {
                        #[sea_orm(has_one, self_ref, relation_enum = #relation_enum_name, relation_reverse = #reverse)]
                        pub #field_name: HasOne<#target_entity>
                    })
                } else {
                    Some(quote! {
                        #[sea_orm(has_one, self_ref, relation_enum = #relation_enum_name)]
                        pub #field_name: HasOne<#target_entity>
                    })
                }
            } else {
                Some(quote! {
                    #[sea_orm(has_one)]
                    pub #field_name: HasOne<#target_entity>
                })
            }
        }
        RelationType::HasMany => {
            if !rel_def.through.is_empty() {
                // Many-to-many via junction table
                // Convert to snake_case module name (treat as message name)
                let via_module = rel_def.through.to_snake_case();
                if is_self_ref {
                    if let Some(reverse) = relation_reverse {
                        Some(quote! {
                            #[sea_orm(has_many, self_ref, relation_enum = #relation_enum_name, relation_reverse = #reverse, via = #via_module)]
                            pub #field_name: HasMany<#target_entity>
                        })
                    } else {
                        Some(quote! {
                            #[sea_orm(has_many, self_ref, relation_enum = #relation_enum_name, via = #via_module)]
                            pub #field_name: HasMany<#target_entity>
                        })
                    }
                } else {
                    Some(quote! {
                        #[sea_orm(has_many, via = #via_module)]
                        pub #field_name: HasMany<#target_entity>
                    })
                }
            } else if is_self_ref {
                if let Some(reverse) = relation_reverse {
                    Some(quote! {
                        #[sea_orm(has_many, self_ref, relation_enum = #relation_enum_name, relation_reverse = #reverse)]
                        pub #field_name: HasMany<#target_entity>
                    })
                } else {
                    Some(quote! {
                        #[sea_orm(has_many, self_ref, relation_enum = #relation_enum_name)]
                        pub #field_name: HasMany<#target_entity>
                    })
                }
            } else {
                Some(quote! {
                    #[sea_orm(has_many)]
                    pub #field_name: HasMany<#target_entity>
                })
            }
        }
        RelationType::BelongsTo => {
            let from_col = if rel_def.foreign_key.is_empty() {
                format!("{}_id", rel_def.related.to_snake_case())
            } else {
                rel_def.foreign_key.clone()
            };
            let to_col = if rel_def.references.is_empty() {
                "id".to_string()
            } else {
                rel_def.references.clone()
            };

            // belongs_to uses HasOne type in SeaORM 2.0 dense format
            if is_self_ref {
                if let Some(reverse) = relation_reverse {
                    Some(quote! {
                        #[sea_orm(self_ref, relation_enum = #relation_enum_name, relation_reverse = #reverse, from = #from_col, to = #to_col)]
                        pub #field_name: HasOne<#target_entity>
                    })
                } else {
                    Some(quote! {
                        #[sea_orm(belongs_to, self_ref, relation_enum = #relation_enum_name, from = #from_col, to = #to_col)]
                        pub #field_name: HasOne<#target_entity>
                    })
                }
            } else {
                Some(quote! {
                    #[sea_orm(belongs_to, from = #from_col, to = #to_col)]
                    pub #field_name: HasOne<#target_entity>
                })
            }
        }
        RelationType::ManyToMany => {
            if !rel_def.through.is_empty() {
                // Convert to snake_case module name (treat as message name)
                let via_module = rel_def.through.to_snake_case();
                if is_self_ref {
                    Some(quote! {
                        #[sea_orm(has_many, self_ref, relation_enum = #relation_enum_name, via = #via_module)]
                        pub #field_name: HasMany<#target_entity>
                    })
                } else {
                    Some(quote! {
                        #[sea_orm(has_many, via = #via_module)]
                        pub #field_name: HasMany<#target_entity>
                    })
                }
            } else if is_self_ref {
                Some(quote! {
                    #[sea_orm(has_many, self_ref, relation_enum = #relation_enum_name)]
                    pub #field_name: HasMany<#target_entity>
                })
            } else {
                Some(quote! {
                    #[sea_orm(has_many)]
                    pub #field_name: HasMany<#target_entity>
                })
            }
        }
        RelationType::Unspecified => None,
    }
}

/// Generate the #[sea_orm(...)] attribute for a relation
pub fn generate_relation_attribute(relation: &GeneratedRelation) -> String {
    match relation.relation_type {
        SeaOrmRelationType::HasOne => {
            format!(
                "has_one = \"{}\"",
                relation.target_entity
            )
        }
        SeaOrmRelationType::HasMany => {
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
        SeaOrmRelationType::BelongsTo => {
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
        SeaOrmRelationType::ManyToMany => {
            // Many-to-many in SeaORM requires a Linked trait implementation
            // We generate the relation through the junction table
            if let Some(ref via) = relation.via_table {
                format!(
                    "many_to_many = \"{}\", via = \"super::{}::Entity\"",
                    relation.target_entity,
                    via.to_snake_case()
                )
            } else {
                // Without a junction table, fall back to has_many (user needs to specify via)
                format!(
                    "has_many = \"{}\"",
                    relation.target_entity
                )
            }
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
        assert_eq!(rel.relation_type, SeaOrmRelationType::HasMany);
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
        assert_eq!(rel.relation_type, SeaOrmRelationType::BelongsTo);
        assert_eq!(rel.from_column, Some("user_id".to_string()));
        assert_eq!(rel.to_column, Some("id".to_string()));
    }

    #[test]
    fn test_no_relation() {
        let opts = FieldOptions::default();
        assert!(generate_relation("field", &opts).is_none());
    }
}
