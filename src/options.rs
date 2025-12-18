//! Options parsing for SeaORM protobuf extensions
//!
//! This module handles parsing of `(seaorm.message)` and `(seaorm.field)` options
//! from protobuf descriptors.
//!
//! Custom protobuf extensions are stored in the `uninterpreted_option` field
//! of the various `*Options` messages. We parse these to extract our SeaORM
//! configuration.

use prost_types::{DescriptorProto, FieldDescriptorProto, UninterpretedOption};

/// Generated SeaORM option types from `proto/seaorm/options.proto`
///
/// These types represent the custom protobuf extensions used to annotate
/// messages and fields with SeaORM configuration.
#[allow(missing_docs)]
pub mod seaorm {
    include!(concat!(env!("OUT_DIR"), "/seaorm.rs"));
}

/// Extension name for message options
const MESSAGE_EXTENSION_NAME: &str = "seaorm.message";

/// Extension name for field options
const FIELD_EXTENSION_NAME: &str = "seaorm.field";

/// Parse SeaORM message options from a DescriptorProto
pub fn parse_message_options(desc: &DescriptorProto) -> Option<seaorm::MessageOptions> {
    let opts = desc.options.as_ref()?;
    parse_message_options_from_uninterpreted(&opts.uninterpreted_option)
}

/// Parse SeaORM field options from a FieldDescriptorProto
pub fn parse_field_options(field: &FieldDescriptorProto) -> Option<seaorm::FieldOptions> {
    let opts = field.options.as_ref()?;
    parse_field_options_from_uninterpreted(&opts.uninterpreted_option)
}

/// Parse MessageOptions from uninterpreted options
fn parse_message_options_from_uninterpreted(
    uninterpreted: &[UninterpretedOption],
) -> Option<seaorm::MessageOptions> {
    let mut result = seaorm::MessageOptions::default();
    let mut found = false;

    for opt in uninterpreted {
        if is_extension_option(opt, MESSAGE_EXTENSION_NAME) {
            found = true;
            apply_message_option(&mut result, opt);
        }
    }

    if found {
        Some(result)
    } else {
        None
    }
}

/// Parse FieldOptions from uninterpreted options
fn parse_field_options_from_uninterpreted(
    uninterpreted: &[UninterpretedOption],
) -> Option<seaorm::FieldOptions> {
    let mut result = seaorm::FieldOptions::default();
    let mut found = false;

    for opt in uninterpreted {
        if is_extension_option(opt, FIELD_EXTENSION_NAME) {
            found = true;
            apply_field_option(&mut result, opt);
        }
    }

    if found {
        Some(result)
    } else {
        None
    }
}

/// Check if an uninterpreted option matches our extension name
fn is_extension_option(opt: &UninterpretedOption, extension_name: &str) -> bool {
    // The name parts form a path like: (seaorm.message).table_name
    // or just (seaorm.message) for aggregate values
    if opt.name.is_empty() {
        return false;
    }

    // First name part should be the extension name in parentheses (is_extension=true)
    let first = &opt.name[0];
    if !first.is_extension {
        return false;
    }

    first.name_part == extension_name
}

/// Get the sub-field name from an uninterpreted option (e.g., "table_name" from "(seaorm.message).table_name")
fn get_subfield_name(opt: &UninterpretedOption) -> Option<&str> {
    if opt.name.len() >= 2 {
        Some(opt.name[1].name_part.as_str())
    } else {
        None
    }
}

/// Apply a single uninterpreted option to MessageOptions
fn apply_message_option(result: &mut seaorm::MessageOptions, opt: &UninterpretedOption) {
    // Check if this is an aggregate value (full message) or individual field
    if let Some(aggregate) = opt.aggregate_value.as_ref() {
        // Parse aggregate value like: table_name: "users", skip: true
        parse_aggregate_into_message_options(result, aggregate);
    } else if let Some(field_name) = get_subfield_name(opt) {
        // Individual field setting like (seaorm.message).table_name = "users"
        match field_name {
            "table_name" => {
                if let Some(ref s) = opt.string_value {
                    result.table_name = String::from_utf8_lossy(s).to_string();
                }
            }
            "skip" => {
                if let Some(v) = opt.identifier_value.as_ref() {
                    result.skip = v == "true";
                }
            }
            _ => {}
        }
    }
}

/// Apply a single uninterpreted option to FieldOptions
fn apply_field_option(result: &mut seaorm::FieldOptions, opt: &UninterpretedOption) {
    // Check if this is an aggregate value (full message) or individual field
    if let Some(aggregate) = opt.aggregate_value.as_ref() {
        // Parse aggregate value like: primary_key: true, auto_increment: true
        parse_aggregate_into_field_options(result, aggregate);
    } else if let Some(field_name) = get_subfield_name(opt) {
        // Individual field setting like (seaorm.field).primary_key = true
        apply_single_field_option(result, field_name, opt);
    }
}

/// Apply a single field option by name
fn apply_single_field_option(
    result: &mut seaorm::FieldOptions,
    field_name: &str,
    opt: &UninterpretedOption,
) {
    match field_name {
        "primary_key" => result.primary_key = parse_bool_option(opt),
        "auto_increment" => result.auto_increment = parse_bool_option(opt),
        "unique" => result.unique = parse_bool_option(opt),
        "nullable" => result.nullable = parse_bool_option(opt),
        "column_name" => result.column_name = parse_string_option(opt),
        "column_type" => result.column_type = parse_string_option(opt),
        "default_value" => result.default_value = parse_string_option(opt),
        "has_one" => result.has_one = parse_string_option(opt),
        "has_many" => result.has_many = parse_string_option(opt),
        "belongs_to" => result.belongs_to = parse_string_option(opt),
        "belongs_to_from" => result.belongs_to_from = parse_string_option(opt),
        "belongs_to_to" => result.belongs_to_to = parse_string_option(opt),
        "has_many_via" => result.has_many_via = parse_string_option(opt),
        _ => {}
    }
}

/// Parse a boolean value from an uninterpreted option
fn parse_bool_option(opt: &UninterpretedOption) -> bool {
    if let Some(ref v) = opt.identifier_value {
        return v == "true";
    }
    if let Some(v) = opt.positive_int_value {
        return v != 0;
    }
    false
}

/// Parse a string value from an uninterpreted option
fn parse_string_option(opt: &UninterpretedOption) -> String {
    if let Some(ref s) = opt.string_value {
        return String::from_utf8_lossy(s).to_string();
    }
    if let Some(ref s) = opt.identifier_value {
        return s.clone();
    }
    String::new()
}

/// Parse an aggregate value (text format) into MessageOptions
///
/// Aggregate values look like: `table_name: "users", skip: true`
fn parse_aggregate_into_message_options(result: &mut seaorm::MessageOptions, aggregate: &str) {
    for part in split_aggregate_parts(aggregate) {
        let (key, value) = match part.split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };

        match key {
            "table_name" => result.table_name = parse_quoted_string(value),
            "skip" => result.skip = value == "true",
            "indexes" => {
                // indexes is repeated, would need more complex parsing
                // For now, handle single value
                result.indexes.push(parse_quoted_string(value));
            }
            _ => {}
        }
    }
}

/// Parse an aggregate value (text format) into FieldOptions
///
/// Aggregate values look like: `primary_key: true, auto_increment: true`
fn parse_aggregate_into_field_options(result: &mut seaorm::FieldOptions, aggregate: &str) {
    for part in split_aggregate_parts(aggregate) {
        let (key, value) = match part.split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };

        match key {
            "primary_key" => result.primary_key = value == "true",
            "auto_increment" => result.auto_increment = value == "true",
            "unique" => result.unique = value == "true",
            "nullable" => result.nullable = value == "true",
            "column_name" => result.column_name = parse_quoted_string(value),
            "column_type" => result.column_type = parse_quoted_string(value),
            "default_value" => result.default_value = parse_quoted_string(value),
            "has_one" => result.has_one = parse_quoted_string(value),
            "has_many" => result.has_many = parse_quoted_string(value),
            "belongs_to" => result.belongs_to = parse_quoted_string(value),
            "belongs_to_from" => result.belongs_to_from = parse_quoted_string(value),
            "belongs_to_to" => result.belongs_to_to = parse_quoted_string(value),
            "has_many_via" => result.has_many_via = parse_quoted_string(value),
            _ => {}
        }
    }
}

/// Split aggregate value into parts, respecting nested braces
fn split_aggregate_parts(aggregate: &str) -> Vec<&str> {
    // Simple split by comma for now - could be enhanced for nested structures
    aggregate.split(',').collect()
}

/// Parse a quoted string value, removing quotes
fn parse_quoted_string(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quoted_string() {
        assert_eq!(parse_quoted_string("\"hello\""), "hello");
        assert_eq!(parse_quoted_string("'world'"), "world");
        assert_eq!(parse_quoted_string("unquoted"), "unquoted");
    }

    #[test]
    fn test_split_aggregate_parts() {
        let parts = split_aggregate_parts("key1: value1, key2: value2");
        assert_eq!(parts.len(), 2);
    }
}
