//! Oneof code generation for SeaORM
//!
//! Handles protobuf oneof fields in entity generation with multiple strategies:
//! - `flatten`: Each variant becomes a nullable column (default)
//! - `json`: Store as JSON with discriminator
//! - `tagged`: Store type tag + value columns

use crate::options::{parse_oneof_options, seaorm::OneofOptions};
use crate::types::map_proto_type;
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use prost_types::{DescriptorProto, FieldDescriptorProto};
use quote::{format_ident, quote};

/// Strategy for handling oneof fields in the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OneofStrategy {
    /// Each variant becomes a nullable column (default)
    #[default]
    Flatten,
    /// Store as JSON with discriminator
    Json,
    /// Store type tag + value columns
    Tagged,
}

impl OneofStrategy {
    /// Parse strategy from string option
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OneofStrategy::Json,
            "tagged" => OneofStrategy::Tagged,
            _ => OneofStrategy::Flatten,
        }
    }
}

/// Information about a oneof and its variants
#[derive(Debug)]
pub struct OneofInfo {
    /// Name of the oneof
    pub name: String,
    /// Strategy for handling the oneof
    pub strategy: OneofStrategy,
    /// Column prefix for flattened fields
    pub column_prefix: String,
    /// Discriminator column name for tagged strategy
    pub discriminator_column: String,
    /// Fields that belong to this oneof
    pub fields: Vec<OneofField>,
}

/// A field within a oneof
#[derive(Debug)]
pub struct OneofField {
    /// Field name
    pub name: String,
    /// Field proto type
    pub proto_type: i32,
    /// Field type name (for messages/enums)
    pub type_name: Option<String>,
}

/// Extract oneof information from a message descriptor
pub fn extract_oneofs(message: &DescriptorProto) -> Vec<OneofInfo> {
    let mut oneofs = Vec::new();

    for (idx, oneof_desc) in message.oneof_decl.iter().enumerate() {
        let oneof_name = oneof_desc
            .name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        // Skip synthetic oneofs (proto3 optional fields create oneofs starting with '_')
        if oneof_name.starts_with('_') {
            continue;
        }

        // Parse options
        let options = parse_oneof_options(oneof_desc);
        let (strategy, column_prefix, discriminator_column) = extract_oneof_settings(&options);

        // Find all fields belonging to this oneof
        let fields: Vec<OneofField> = message
            .field
            .iter()
            .filter(|f| f.oneof_index == Some(idx as i32))
            .map(|f| OneofField {
                name: f.name.clone().unwrap_or_default(),
                proto_type: f.r#type.unwrap_or(0),
                type_name: f.type_name.clone(),
            })
            .collect();

        oneofs.push(OneofInfo {
            name: oneof_name.to_string(),
            strategy,
            column_prefix,
            discriminator_column,
            fields,
        });
    }

    oneofs
}

/// Extract oneof settings from options
fn extract_oneof_settings(options: &Option<OneofOptions>) -> (OneofStrategy, String, String) {
    match options {
        Some(opts) => (
            OneofStrategy::from_str(&opts.strategy),
            opts.column_prefix.clone(),
            opts.discriminator_column.clone(),
        ),
        None => (OneofStrategy::Flatten, String::new(), String::new()),
    }
}

/// Check if a field belongs to a oneof
pub fn is_oneof_field(field: &FieldDescriptorProto, message: &DescriptorProto) -> bool {
    if let Some(idx) = field.oneof_index {
        // Check if this is a real oneof (not a proto3 optional)
        // Proto3 optional fields also use oneof_index but are synthetic
        if (idx as usize) < message.oneof_decl.len() {
            let oneof = &message.oneof_decl[idx as usize];
            // Synthetic oneofs for proto3 optional have names starting with _
            if let Some(ref name) = oneof.name {
                return !name.starts_with('_');
            }
        }
    }
    false
}

/// Generate fields for a flatten strategy oneof
pub fn generate_flatten_fields(
    oneof: &OneofInfo,
    message: &DescriptorProto,
) -> Vec<TokenStream> {
    let mut fields = Vec::new();

    for oneof_field in &oneof.fields {
        // Find the full field descriptor
        let field_desc = message
            .field
            .iter()
            .find(|f| f.name.as_ref() == Some(&oneof_field.name));

        if let Some(field) = field_desc {
            let field_name = &oneof_field.name;
            let column_name = if oneof.column_prefix.is_empty() {
                field_name.to_snake_case()
            } else {
                format!("{}_{}", oneof.column_prefix, field_name.to_snake_case())
            };

            let field_ident = format_ident!("{}", field_name.to_snake_case());
            let mapped = map_proto_type(field.r#type(), field.type_name.as_deref());
            let rust_type: syn::Type = syn::parse_str(&mapped.rust_type)
                .unwrap_or_else(|_| syn::parse_quote!(String));

            // All oneof fields are nullable since only one can be set
            let column_attr = quote! {
                #[sea_orm(column_name = #column_name, nullable)]
            };

            fields.push(quote! {
                #column_attr
                pub #field_ident: Option<#rust_type>
            });
        }
    }

    fields
}

/// Generate fields for a JSON strategy oneof
pub fn generate_json_fields(oneof: &OneofInfo) -> Vec<TokenStream> {
    let field_name = format_ident!("{}", oneof.name.to_snake_case());
    let column_name = oneof.name.to_snake_case();

    // For JSON strategy, we store the entire oneof as a JSON column
    // The actual Rust type would be an enum, but for simplicity we use Json<serde_json::Value>
    vec![quote! {
        #[sea_orm(column_name = #column_name, column_type = "Json")]
        pub #field_name: Option<sea_orm::prelude::Json>
    }]
}

/// Generate fields for a tagged strategy oneof
pub fn generate_tagged_fields(oneof: &OneofInfo) -> Vec<TokenStream> {
    let base_name = oneof.name.to_snake_case();

    // Discriminator column name
    let disc_col = if oneof.discriminator_column.is_empty() {
        format!("{}_type", base_name)
    } else {
        oneof.discriminator_column.clone()
    };
    let disc_ident = format_ident!("{}", disc_col.to_snake_case());

    // Value column name
    let value_col = format!("{}_value", base_name);
    let value_ident = format_ident!("{}", value_col);

    vec![
        quote! {
            #[sea_orm(column_name = #disc_col)]
            pub #disc_ident: Option<String>
        },
        quote! {
            #[sea_orm(column_name = #value_col, column_type = "Text")]
            pub #value_ident: Option<String>
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oneof_strategy_from_str() {
        assert_eq!(OneofStrategy::from_str("flatten"), OneofStrategy::Flatten);
        assert_eq!(OneofStrategy::from_str("json"), OneofStrategy::Json);
        assert_eq!(OneofStrategy::from_str("tagged"), OneofStrategy::Tagged);
        assert_eq!(OneofStrategy::from_str("unknown"), OneofStrategy::Flatten);
        assert_eq!(OneofStrategy::from_str("JSON"), OneofStrategy::Json);
    }

    #[test]
    fn test_is_oneof_field_empty() {
        let field = FieldDescriptorProto {
            oneof_index: None,
            ..Default::default()
        };
        let message = DescriptorProto::default();
        assert!(!is_oneof_field(&field, &message));
    }
}
