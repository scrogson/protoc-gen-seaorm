//! Options parsing for SeaORM protobuf extensions
//!
//! This module handles parsing of `(seaorm.message)`, `(seaorm.field)`,
//! `(seaorm.enum_opt)`, `(seaorm.enum_value)`, and `(seaorm.oneof)` options
//! from protobuf descriptors.
//!
//! Custom protobuf extensions are stored in the `uninterpreted_option` field
//! of the various `*Options` messages. We parse these to extract our SeaORM
//! configuration.

use prost_types::{
    DescriptorProto, EnumDescriptorProto, EnumValueDescriptorProto, FieldDescriptorProto,
    OneofDescriptorProto, UninterpretedOption,
};

/// Generated SeaORM option types from `proto/seaorm/options.proto`
///
/// These types represent the custom protobuf extensions used to annotate
/// messages and fields with SeaORM configuration.
#[allow(missing_docs)]
pub mod seaorm {
    include!(concat!(env!("OUT_DIR"), "/seaorm.rs"));
}

/// Extension name for model options
const MODEL_EXTENSION_NAME: &str = "seaorm.model";

/// Extension name for field options
const FIELD_EXTENSION_NAME: &str = "seaorm.field";

/// Extension name for enum options
const ENUM_EXTENSION_NAME: &str = "seaorm.enum_opt";

/// Extension name for enum value options
const ENUM_VALUE_EXTENSION_NAME: &str = "seaorm.enum_value";

/// Extension name for oneof options
const ONEOF_EXTENSION_NAME: &str = "seaorm.oneof";

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

/// Parse SeaORM enum options from an EnumDescriptorProto
pub fn parse_enum_options(enum_desc: &EnumDescriptorProto) -> Option<seaorm::EnumOptions> {
    let opts = enum_desc.options.as_ref()?;
    parse_enum_options_from_uninterpreted(&opts.uninterpreted_option)
}

/// Parse SeaORM enum value options from an EnumValueDescriptorProto
pub fn parse_enum_value_options(
    value: &EnumValueDescriptorProto,
) -> Option<seaorm::EnumValueOptions> {
    let opts = value.options.as_ref()?;
    parse_enum_value_options_from_uninterpreted(&opts.uninterpreted_option)
}

/// Parse SeaORM oneof options from a OneofDescriptorProto
pub fn parse_oneof_options(oneof: &OneofDescriptorProto) -> Option<seaorm::OneofOptions> {
    let opts = oneof.options.as_ref()?;
    parse_oneof_options_from_uninterpreted(&opts.uninterpreted_option)
}

/// Parse MessageOptions from uninterpreted options
fn parse_message_options_from_uninterpreted(
    uninterpreted: &[UninterpretedOption],
) -> Option<seaorm::MessageOptions> {
    let mut result = seaorm::MessageOptions::default();
    let mut found = false;

    for opt in uninterpreted {
        if is_extension_option(opt, MODEL_EXTENSION_NAME) {
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

/// Parse EnumOptions from uninterpreted options
fn parse_enum_options_from_uninterpreted(
    uninterpreted: &[UninterpretedOption],
) -> Option<seaorm::EnumOptions> {
    let mut result = seaorm::EnumOptions::default();
    let mut found = false;

    for opt in uninterpreted {
        if is_extension_option(opt, ENUM_EXTENSION_NAME) {
            found = true;
            apply_enum_option(&mut result, opt);
        }
    }

    if found {
        Some(result)
    } else {
        None
    }
}

/// Parse EnumValueOptions from uninterpreted options
fn parse_enum_value_options_from_uninterpreted(
    uninterpreted: &[UninterpretedOption],
) -> Option<seaorm::EnumValueOptions> {
    let mut result = seaorm::EnumValueOptions::default();
    let mut found = false;

    for opt in uninterpreted {
        if is_extension_option(opt, ENUM_VALUE_EXTENSION_NAME) {
            found = true;
            apply_enum_value_option(&mut result, opt);
        }
    }

    if found {
        Some(result)
    } else {
        None
    }
}

/// Parse OneofOptions from uninterpreted options
fn parse_oneof_options_from_uninterpreted(
    uninterpreted: &[UninterpretedOption],
) -> Option<seaorm::OneofOptions> {
    let mut result = seaorm::OneofOptions::default();
    let mut found = false;

    for opt in uninterpreted {
        if is_extension_option(opt, ONEOF_EXTENSION_NAME) {
            found = true;
            apply_oneof_option(&mut result, opt);
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

/// Apply a single uninterpreted option to EnumOptions
fn apply_enum_option(result: &mut seaorm::EnumOptions, opt: &UninterpretedOption) {
    if let Some(aggregate) = opt.aggregate_value.as_ref() {
        parse_aggregate_into_enum_options(result, aggregate);
    } else if let Some(field_name) = get_subfield_name(opt) {
        match field_name {
            "name" => result.name = parse_string_option(opt),
            "db_type" => result.db_type = parse_string_option(opt),
            "skip" => result.skip = parse_bool_option(opt),
            _ => {}
        }
    }
}

/// Apply a single uninterpreted option to EnumValueOptions
fn apply_enum_value_option(result: &mut seaorm::EnumValueOptions, opt: &UninterpretedOption) {
    if let Some(aggregate) = opt.aggregate_value.as_ref() {
        parse_aggregate_into_enum_value_options(result, aggregate);
    } else if let Some(field_name) = get_subfield_name(opt) {
        match field_name {
            "name" => result.name = parse_string_option(opt),
            "string_value" => result.string_value = parse_string_option(opt),
            "int_value" => result.int_value = parse_int_option(opt),
            _ => {}
        }
    }
}

/// Apply a single uninterpreted option to OneofOptions
fn apply_oneof_option(result: &mut seaorm::OneofOptions, opt: &UninterpretedOption) {
    if let Some(aggregate) = opt.aggregate_value.as_ref() {
        parse_aggregate_into_oneof_options(result, aggregate);
    } else if let Some(field_name) = get_subfield_name(opt) {
        match field_name {
            "strategy" => result.strategy = parse_string_option(opt),
            "column_prefix" => result.column_prefix = parse_string_option(opt),
            "discriminator_column" => result.discriminator_column = parse_string_option(opt),
            _ => {}
        }
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
        "embed" => result.embed = parse_bool_option(opt),
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

/// Parse an integer value from an uninterpreted option
fn parse_int_option(opt: &UninterpretedOption) -> i32 {
    if let Some(v) = opt.positive_int_value {
        return v as i32;
    }
    if let Some(v) = opt.negative_int_value {
        return v as i32;
    }
    0
}

/// Parse an aggregate value (text format) into MessageOptions
///
/// Aggregate values look like: `table_name: "users", skip: true`
/// Or with relations: `table_name: "users", relations: [{name: "posts", ...}]`
fn parse_aggregate_into_message_options(result: &mut seaorm::MessageOptions, aggregate: &str) {
    // First, extract relations if present (they need special handling for nested braces)
    if let Some(rel_start) = aggregate.find("relations:") {
        let after_key = &aggregate[rel_start + "relations:".len()..];
        if let Some(relations) = extract_relations_array(after_key.trim()) {
            result.relations = relations;
        }
    }

    // Parse simple key-value pairs (excluding relations which we handled above)
    for part in split_aggregate_parts_simple(aggregate) {
        let (key, value) = match part.split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };

        // Skip relations - already handled above
        if key == "relations" {
            continue;
        }

        match key {
            "table_name" => result.table_name = parse_quoted_string(value),
            "skip" => result.skip = value == "true",
            "indexes" => {
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
            "embed" => result.embed = value == "true",
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

/// Parse an aggregate value (text format) into EnumOptions
fn parse_aggregate_into_enum_options(result: &mut seaorm::EnumOptions, aggregate: &str) {
    for part in split_aggregate_parts(aggregate) {
        let (key, value) = match part.split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };

        match key {
            "name" => result.name = parse_quoted_string(value),
            "db_type" => result.db_type = parse_quoted_string(value),
            "skip" => result.skip = value == "true",
            _ => {}
        }
    }
}

/// Parse an aggregate value (text format) into EnumValueOptions
fn parse_aggregate_into_enum_value_options(result: &mut seaorm::EnumValueOptions, aggregate: &str) {
    for part in split_aggregate_parts(aggregate) {
        let (key, value) = match part.split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };

        match key {
            "name" => result.name = parse_quoted_string(value),
            "string_value" => result.string_value = parse_quoted_string(value),
            "int_value" => {
                if let Ok(v) = value.parse::<i32>() {
                    result.int_value = v;
                }
            }
            _ => {}
        }
    }
}

/// Parse an aggregate value (text format) into OneofOptions
fn parse_aggregate_into_oneof_options(result: &mut seaorm::OneofOptions, aggregate: &str) {
    for part in split_aggregate_parts(aggregate) {
        let (key, value) = match part.split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };

        match key {
            "strategy" => result.strategy = parse_quoted_string(value),
            "column_prefix" => result.column_prefix = parse_quoted_string(value),
            "discriminator_column" => result.discriminator_column = parse_quoted_string(value),
            _ => {}
        }
    }
}

/// Split aggregate value into simple parts (only top-level commas, not inside braces)
fn split_aggregate_parts_simple(aggregate: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut brace_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;

    for (i, c) in aggregate.char_indices() {
        match c {
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if brace_depth == 0 && bracket_depth == 0 => {
                parts.push(&aggregate[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    if start < aggregate.len() {
        parts.push(&aggregate[start..]);
    }

    parts
}

/// Extract relations array from text format
///
/// Handles both array syntax `[{...}, {...}]` and single object syntax `{...}`
fn extract_relations_array(s: &str) -> Option<Vec<seaorm::RelationDef>> {
    let s = s.trim();
    let mut relations = Vec::new();

    if s.starts_with('[') {
        // Array of relations: [{...}, {...}]
        let content = extract_balanced(s, '[', ']')?;
        for obj_str in extract_objects(content) {
            if let Some(rel) = parse_single_relation_def(obj_str) {
                relations.push(rel);
            }
        }
    } else if s.starts_with('{') {
        // Single relation: {...}
        let content = extract_balanced(s, '{', '}')?;
        if let Some(rel) = parse_single_relation_def(content) {
            relations.push(rel);
        }
    }

    if relations.is_empty() {
        None
    } else {
        Some(relations)
    }
}

/// Extract content between balanced delimiters
fn extract_balanced(s: &str, open: char, close: char) -> Option<&str> {
    let s = s.trim();
    if !s.starts_with(open) {
        return None;
    }

    let mut depth = 0;
    for (i, c) in s.char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return Some(&s[1..i]);
            }
        }
    }
    None
}

/// Extract individual objects from an array content string
fn extract_objects(s: &str) -> Vec<&str> {
    let mut objects = Vec::new();
    let mut start = None;
    let mut depth = 0;

    for (i, c) in s.char_indices() {
        match c {
            '{' => {
                if depth == 0 {
                    start = Some(i + 1);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s_idx) = start {
                        objects.push(&s[s_idx..i]);
                    }
                    start = None;
                }
            }
            _ => {}
        }
    }

    objects
}

/// Parse a single relation definition from object content
fn parse_single_relation_def(s: &str) -> Option<seaorm::RelationDef> {
    let mut rel = seaorm::RelationDef::default();
    let mut has_content = false;

    for part in split_aggregate_parts_simple(s) {
        let (key, value) = match part.split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };

        has_content = true;
        match key {
            "name" => rel.name = parse_quoted_string(value),
            "type" => rel.r#type = parse_relation_type(value),
            "related" | "related_schema" => rel.related = parse_quoted_string(value),
            "foreign_key" => rel.foreign_key = parse_quoted_string(value),
            "references" => rel.references = parse_quoted_string(value),
            "through" => rel.through = parse_quoted_string(value),
            _ => {}
        }
    }

    if has_content {
        Some(rel)
    } else {
        None
    }
}

/// Parse relation type from string
fn parse_relation_type(s: &str) -> i32 {
    let s = s.trim();
    match s {
        "RELATION_TYPE_BELONGS_TO" | "belongs_to" | "BelongsTo" => {
            seaorm::RelationType::BelongsTo as i32
        }
        "RELATION_TYPE_HAS_ONE" | "has_one" | "HasOne" => {
            seaorm::RelationType::HasOne as i32
        }
        "RELATION_TYPE_HAS_MANY" | "has_many" | "HasMany" => {
            seaorm::RelationType::HasMany as i32
        }
        "RELATION_TYPE_MANY_TO_MANY" | "many_to_many" | "ManyToMany" => {
            seaorm::RelationType::ManyToMany as i32
        }
        _ => seaorm::RelationType::Unspecified as i32,
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
