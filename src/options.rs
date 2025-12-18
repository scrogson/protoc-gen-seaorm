//! Options parsing for SeaORM protobuf extensions
//!
//! This module handles parsing of `(seaorm.model)`, `(seaorm.field)`,
//! `(seaorm.enum_opt)`, `(seaorm.enum_value)`, and `(seaorm.oneof)` options
//! from protobuf descriptors.
//!
//! Custom protobuf extensions are stored as extension fields in the options
//! messages. We use prost-reflect to decode these extensions from the raw
//! protobuf bytes.

use once_cell::sync::Lazy;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, Value};
use prost_types::{
    DescriptorProto, EnumDescriptorProto, EnumValueDescriptorProto, FieldDescriptorProto,
    OneofDescriptorProto, UninterpretedOption,
};
use std::collections::HashMap;
use std::sync::RwLock;

/// Generated SeaORM option types from `proto/seaorm/options.proto`
///
/// These types represent the custom protobuf extensions used to annotate
/// messages and fields with SeaORM configuration.
#[allow(missing_docs)]
pub mod seaorm {
    include!(concat!(env!("OUT_DIR"), "/seaorm.rs"));
}

/// File descriptor set bytes generated at build time by protoc
static FILE_DESCRIPTOR_SET_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/file_descriptor_set.bin"));

/// Extension name for model options (used for uninterpreted_option fallback)
const MODEL_EXTENSION_NAME: &str = "seaorm.model";

/// Extension name for field options
const FIELD_EXTENSION_NAME: &str = "seaorm.field";

/// Extension name for enum options
const ENUM_EXTENSION_NAME: &str = "seaorm.enum_opt";

/// Extension name for enum value options
const ENUM_VALUE_EXTENSION_NAME: &str = "seaorm.enum_value";

/// Extension name for oneof options
const ONEOF_EXTENSION_NAME: &str = "seaorm.oneof";

/// Lazily initialized descriptor pool with our extension definitions
static DESCRIPTOR_POOL: Lazy<DescriptorPool> = Lazy::new(|| {
    DescriptorPool::decode(FILE_DESCRIPTOR_SET_BYTES).expect("Failed to decode file descriptor set")
});

/// Global cache of pre-parsed options from raw bytes
static OPTIONS_CACHE: Lazy<RwLock<OptionsCache>> =
    Lazy::new(|| RwLock::new(OptionsCache::default()));

/// Cache structure holding pre-parsed options
#[derive(Default)]
struct OptionsCache {
    /// Message options: (file_name, message_name) -> MessageOptions
    message_options: HashMap<(String, String), seaorm::MessageOptions>,
    /// Field options: (file_name, message_name, field_number) -> FieldOptions
    field_options: HashMap<(String, String, i32), seaorm::FieldOptions>,
    /// Enum options: (file_name, enum_name) -> EnumOptions
    enum_options: HashMap<(String, String), seaorm::EnumOptions>,
    /// Enum value options: (file_name, enum_name, value_number) -> EnumValueOptions
    enum_value_options: HashMap<(String, String, i32), seaorm::EnumValueOptions>,
    /// Oneof options: (file_name, message_name, oneof_index) -> OneofOptions
    oneof_options: HashMap<(String, String, i32), seaorm::OneofOptions>,
}

/// Pre-process raw CodeGeneratorRequest bytes to extract options using prost-reflect
///
/// This must be called before `generate()` to populate the options cache with
/// extension data that would otherwise be lost when prost decodes the request.
pub fn preprocess_request_bytes(bytes: &[u8]) -> Result<(), String> {
    // Get the CodeGeneratorRequest descriptor
    let request_desc = DESCRIPTOR_POOL
        .get_message_by_name("google.protobuf.compiler.CodeGeneratorRequest")
        .ok_or("CodeGeneratorRequest not found in descriptor pool")?;

    // Decode the request as a DynamicMessage
    let request = DynamicMessage::decode(request_desc, bytes)
        .map_err(|e| format!("Failed to decode CodeGeneratorRequest: {}", e))?;

    let mut cache = OPTIONS_CACHE
        .write()
        .map_err(|e| format!("Lock error: {}", e))?;

    // Get proto_file field
    if let Some(cow) = request.get_field_by_name("proto_file") {
        if let Value::List(files) = cow.as_ref() {
            for file_value in files.iter() {
                if let Some(file_msg) = file_value.as_message() {
                    extract_options_from_file(&mut cache, file_msg)?;
                }
            }
        }
    }

    Ok(())
}

/// Extract options from a FileDescriptorProto DynamicMessage
fn extract_options_from_file(
    cache: &mut OptionsCache,
    file: &DynamicMessage,
) -> Result<(), String> {
    let file_name = file
        .get_field_by_name("name")
        .and_then(|v| v.as_ref().as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    // Extract message options
    if let Some(cow) = file.get_field_by_name("message_type") {
        if let Value::List(messages) = cow.as_ref() {
            for msg_value in messages.iter() {
                if let Some(msg) = msg_value.as_message() {
                    extract_message_options(cache, &file_name, msg, "")?;
                }
            }
        }
    }

    // Extract enum options
    if let Some(cow) = file.get_field_by_name("enum_type") {
        if let Value::List(enums) = cow.as_ref() {
            for enum_value in enums.iter() {
                if let Some(enum_msg) = enum_value.as_message() {
                    extract_enum_options(cache, &file_name, enum_msg)?;
                }
            }
        }
    }

    Ok(())
}

/// Extract options from a DescriptorProto DynamicMessage
fn extract_message_options(
    cache: &mut OptionsCache,
    file_name: &str,
    msg: &DynamicMessage,
    parent_prefix: &str,
) -> Result<(), String> {
    let msg_name = msg
        .get_field_by_name("name")
        .and_then(|v| v.as_ref().as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let full_name = if parent_prefix.is_empty() {
        msg_name.clone()
    } else {
        format!("{}.{}", parent_prefix, msg_name)
    };

    // Extract message-level options (seaorm.model)
    if let Some(cow) = msg.get_field_by_name("options") {
        if let Some(opts_msg) = cow.as_ref().as_message() {
            // Get the seaorm.model extension
            if let Some(ext_field) = DESCRIPTOR_POOL.get_extension_by_name("seaorm.model") {
                if opts_msg.has_extension(&ext_field) {
                    let ext_value = opts_msg.get_extension(&ext_field);
                    if let Some(model_opts) = convert_to_message_options(&ext_value) {
                        cache
                            .message_options
                            .insert((file_name.to_string(), full_name.clone()), model_opts);
                    }
                }
            }
        }
    }

    // Extract field-level options (seaorm.field)
    if let Some(cow) = msg.get_field_by_name("field") {
        if let Value::List(fields) = cow.as_ref() {
            for field_value in fields.iter() {
                if let Some(field_msg) = field_value.as_message() {
                    let field_number = field_msg
                        .get_field_by_name("number")
                        .and_then(|v| {
                            if let Value::I32(n) = v.as_ref() {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0);

                    if let Some(opts_cow) = field_msg.get_field_by_name("options") {
                        if let Some(opts_msg) = opts_cow.as_ref().as_message() {
                            if let Some(ext_field) =
                                DESCRIPTOR_POOL.get_extension_by_name("seaorm.field")
                            {
                                if opts_msg.has_extension(&ext_field) {
                                    let ext_value = opts_msg.get_extension(&ext_field);
                                    if let Some(field_opts) = convert_to_field_options(&ext_value) {
                                        cache.field_options.insert(
                                            (
                                                file_name.to_string(),
                                                full_name.clone(),
                                                field_number,
                                            ),
                                            field_opts,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Extract oneof-level options (seaorm.oneof)
    if let Some(cow) = msg.get_field_by_name("oneof_decl") {
        if let Value::List(oneofs) = cow.as_ref() {
            for (idx, oneof_value) in oneofs.iter().enumerate() {
                if let Some(oneof_msg) = oneof_value.as_message() {
                    if let Some(opts_cow) = oneof_msg.get_field_by_name("options") {
                        if let Some(opts_msg) = opts_cow.as_ref().as_message() {
                            if let Some(ext_field) =
                                DESCRIPTOR_POOL.get_extension_by_name("seaorm.oneof")
                            {
                                if opts_msg.has_extension(&ext_field) {
                                    let ext_value = opts_msg.get_extension(&ext_field);
                                    if let Some(oneof_opts) = convert_to_oneof_options(&ext_value) {
                                        cache.oneof_options.insert(
                                            (file_name.to_string(), full_name.clone(), idx as i32),
                                            oneof_opts,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Process nested messages
    if let Some(cow) = msg.get_field_by_name("nested_type") {
        if let Value::List(nested) = cow.as_ref() {
            for nested_value in nested.iter() {
                if let Some(nested_msg) = nested_value.as_message() {
                    extract_message_options(cache, file_name, nested_msg, &full_name)?;
                }
            }
        }
    }

    // Process nested enums
    if let Some(cow) = msg.get_field_by_name("enum_type") {
        if let Value::List(enums) = cow.as_ref() {
            for enum_value in enums.iter() {
                if let Some(enum_msg) = enum_value.as_message() {
                    extract_enum_options_nested(cache, file_name, enum_msg, &full_name)?;
                }
            }
        }
    }

    Ok(())
}

/// Extract options from an EnumDescriptorProto DynamicMessage
fn extract_enum_options(
    cache: &mut OptionsCache,
    file_name: &str,
    enum_msg: &DynamicMessage,
) -> Result<(), String> {
    extract_enum_options_nested(cache, file_name, enum_msg, "")
}

/// Extract options from an EnumDescriptorProto with optional parent prefix
fn extract_enum_options_nested(
    cache: &mut OptionsCache,
    file_name: &str,
    enum_msg: &DynamicMessage,
    parent_prefix: &str,
) -> Result<(), String> {
    let enum_name = enum_msg
        .get_field_by_name("name")
        .and_then(|v| v.as_ref().as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let full_name = if parent_prefix.is_empty() {
        enum_name.clone()
    } else {
        format!("{}.{}", parent_prefix, enum_name)
    };

    // Extract enum-level options (seaorm.enum_opt)
    if let Some(cow) = enum_msg.get_field_by_name("options") {
        if let Some(opts_msg) = cow.as_ref().as_message() {
            if let Some(ext_field) = DESCRIPTOR_POOL.get_extension_by_name("seaorm.enum_opt") {
                if opts_msg.has_extension(&ext_field) {
                    let ext_value = opts_msg.get_extension(&ext_field);
                    if let Some(enum_opts) = convert_to_enum_options(&ext_value) {
                        cache
                            .enum_options
                            .insert((file_name.to_string(), full_name.clone()), enum_opts);
                    }
                }
            }
        }
    }

    // Extract enum value options (seaorm.enum_value)
    if let Some(cow) = enum_msg.get_field_by_name("value") {
        if let Value::List(values) = cow.as_ref() {
            for value_val in values.iter() {
                if let Some(value_msg) = value_val.as_message() {
                    let value_number = value_msg
                        .get_field_by_name("number")
                        .and_then(|v| {
                            if let Value::I32(n) = v.as_ref() {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0);

                    if let Some(opts_cow) = value_msg.get_field_by_name("options") {
                        if let Some(opts_msg) = opts_cow.as_ref().as_message() {
                            if let Some(ext_field) =
                                DESCRIPTOR_POOL.get_extension_by_name("seaorm.enum_value")
                            {
                                if opts_msg.has_extension(&ext_field) {
                                    let ext_value = opts_msg.get_extension(&ext_field);
                                    if let Some(value_opts) =
                                        convert_to_enum_value_options(&ext_value)
                                    {
                                        cache.enum_value_options.insert(
                                            (
                                                file_name.to_string(),
                                                full_name.clone(),
                                                value_number,
                                            ),
                                            value_opts,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Look up cached message options for a given file and message name
pub fn get_cached_message_options(
    file_name: &str,
    msg_name: &str,
) -> Option<seaorm::MessageOptions> {
    OPTIONS_CACHE.read().ok().and_then(|cache| {
        cache
            .message_options
            .get(&(file_name.to_string(), msg_name.to_string()))
            .cloned()
    })
}

/// Look up cached field options for a given file, message name, and field number
pub fn get_cached_field_options(
    file_name: &str,
    msg_name: &str,
    field_number: i32,
) -> Option<seaorm::FieldOptions> {
    OPTIONS_CACHE.read().ok().and_then(|cache| {
        cache
            .field_options
            .get(&(file_name.to_string(), msg_name.to_string(), field_number))
            .cloned()
    })
}

/// Look up cached enum options for a given file and enum name
pub fn get_cached_enum_options(file_name: &str, enum_name: &str) -> Option<seaorm::EnumOptions> {
    OPTIONS_CACHE.read().ok().and_then(|cache| {
        cache
            .enum_options
            .get(&(file_name.to_string(), enum_name.to_string()))
            .cloned()
    })
}

/// Look up cached oneof options for a given file, message name, and oneof index
pub fn get_cached_oneof_options(
    file_name: &str,
    msg_name: &str,
    oneof_index: i32,
) -> Option<seaorm::OneofOptions> {
    OPTIONS_CACHE.read().ok().and_then(|cache| {
        cache
            .oneof_options
            .get(&(file_name.to_string(), msg_name.to_string(), oneof_index))
            .cloned()
    })
}

/// Parse SeaORM message options from a DescriptorProto
pub fn parse_message_options(desc: &DescriptorProto) -> Option<seaorm::MessageOptions> {
    let opts = desc.options.as_ref()?;

    // First try to parse from extension fields using prost-reflect
    if let Some(result) = parse_message_options_from_extension(opts) {
        return Some(result);
    }

    // Fallback to uninterpreted_option for backwards compatibility
    parse_message_options_from_uninterpreted(&opts.uninterpreted_option)
}

/// Parse SeaORM field options from a FieldDescriptorProto
pub fn parse_field_options(field: &FieldDescriptorProto) -> Option<seaorm::FieldOptions> {
    let opts = field.options.as_ref()?;

    // First try to parse from extension fields using prost-reflect
    if let Some(result) = parse_field_options_from_extension(opts) {
        return Some(result);
    }

    // Fallback to uninterpreted_option
    parse_field_options_from_uninterpreted(&opts.uninterpreted_option)
}

/// Parse SeaORM enum options from an EnumDescriptorProto
pub fn parse_enum_options(enum_desc: &EnumDescriptorProto) -> Option<seaorm::EnumOptions> {
    let opts = enum_desc.options.as_ref()?;

    // First try to parse from extension fields using prost-reflect
    if let Some(result) = parse_enum_options_from_extension(opts) {
        return Some(result);
    }

    // Fallback to uninterpreted_option
    parse_enum_options_from_uninterpreted(&opts.uninterpreted_option)
}

/// Parse SeaORM enum value options from an EnumValueDescriptorProto
pub fn parse_enum_value_options(
    value: &EnumValueDescriptorProto,
) -> Option<seaorm::EnumValueOptions> {
    let opts = value.options.as_ref()?;

    // First try to parse from extension fields using prost-reflect
    if let Some(result) = parse_enum_value_options_from_extension(opts) {
        return Some(result);
    }

    // Fallback to uninterpreted_option
    parse_enum_value_options_from_uninterpreted(&opts.uninterpreted_option)
}

/// Parse SeaORM oneof options from a OneofDescriptorProto
pub fn parse_oneof_options(oneof: &OneofDescriptorProto) -> Option<seaorm::OneofOptions> {
    let opts = oneof.options.as_ref()?;

    // First try to parse from extension fields using prost-reflect
    if let Some(result) = parse_oneof_options_from_extension(opts) {
        return Some(result);
    }

    // Fallback to uninterpreted_option
    parse_oneof_options_from_uninterpreted(&opts.uninterpreted_option)
}

// =============================================================================
// Extension parsing using prost-reflect
// =============================================================================

/// Parse MessageOptions from extension fields using prost-reflect
fn parse_message_options_from_extension(
    opts: &prost_types::MessageOptions,
) -> Option<seaorm::MessageOptions> {
    // Re-encode the options to bytes so we can decode with prost-reflect
    let mut buf = Vec::new();
    opts.encode(&mut buf).ok()?;

    if buf.is_empty() {
        return None;
    }

    // Get the MessageOptions descriptor from the pool
    let message_options_desc =
        DESCRIPTOR_POOL.get_message_by_name("google.protobuf.MessageOptions")?;

    // Decode the bytes as a DynamicMessage
    let dynamic_msg = DynamicMessage::decode(message_options_desc, &buf[..]).ok()?;

    // Try to get the extension field
    let ext_field = DESCRIPTOR_POOL.get_extension_by_name("seaorm.model")?;

    if !dynamic_msg.has_extension(&ext_field) {
        return None;
    }

    let ext_value = dynamic_msg.get_extension(&ext_field);

    // Convert the extension value to our MessageOptions type
    convert_to_message_options(&ext_value)
}

/// Parse FieldOptions from extension fields using prost-reflect
fn parse_field_options_from_extension(
    opts: &prost_types::FieldOptions,
) -> Option<seaorm::FieldOptions> {
    let mut buf = Vec::new();
    opts.encode(&mut buf).ok()?;

    if buf.is_empty() {
        return None;
    }

    let field_options_desc = DESCRIPTOR_POOL.get_message_by_name("google.protobuf.FieldOptions")?;

    let dynamic_msg = DynamicMessage::decode(field_options_desc, &buf[..]).ok()?;

    let ext_field = DESCRIPTOR_POOL.get_extension_by_name("seaorm.field")?;

    if !dynamic_msg.has_extension(&ext_field) {
        return None;
    }

    let ext_value = dynamic_msg.get_extension(&ext_field);

    convert_to_field_options(&ext_value)
}

/// Parse EnumOptions from extension fields using prost-reflect
fn parse_enum_options_from_extension(
    opts: &prost_types::EnumOptions,
) -> Option<seaorm::EnumOptions> {
    let mut buf = Vec::new();
    opts.encode(&mut buf).ok()?;

    if buf.is_empty() {
        return None;
    }

    let enum_options_desc = DESCRIPTOR_POOL.get_message_by_name("google.protobuf.EnumOptions")?;

    let dynamic_msg = DynamicMessage::decode(enum_options_desc, &buf[..]).ok()?;

    let ext_field = DESCRIPTOR_POOL.get_extension_by_name("seaorm.enum_opt")?;

    if !dynamic_msg.has_extension(&ext_field) {
        return None;
    }

    let ext_value = dynamic_msg.get_extension(&ext_field);

    convert_to_enum_options(&ext_value)
}

/// Parse EnumValueOptions from extension fields using prost-reflect
fn parse_enum_value_options_from_extension(
    opts: &prost_types::EnumValueOptions,
) -> Option<seaorm::EnumValueOptions> {
    let mut buf = Vec::new();
    opts.encode(&mut buf).ok()?;

    if buf.is_empty() {
        return None;
    }

    let enum_value_options_desc =
        DESCRIPTOR_POOL.get_message_by_name("google.protobuf.EnumValueOptions")?;

    let dynamic_msg = DynamicMessage::decode(enum_value_options_desc, &buf[..]).ok()?;

    let ext_field = DESCRIPTOR_POOL.get_extension_by_name("seaorm.enum_value")?;

    if !dynamic_msg.has_extension(&ext_field) {
        return None;
    }

    let ext_value = dynamic_msg.get_extension(&ext_field);

    convert_to_enum_value_options(&ext_value)
}

/// Parse OneofOptions from extension fields using prost-reflect
fn parse_oneof_options_from_extension(
    opts: &prost_types::OneofOptions,
) -> Option<seaorm::OneofOptions> {
    let mut buf = Vec::new();
    opts.encode(&mut buf).ok()?;

    if buf.is_empty() {
        return None;
    }

    let oneof_options_desc = DESCRIPTOR_POOL.get_message_by_name("google.protobuf.OneofOptions")?;

    let dynamic_msg = DynamicMessage::decode(oneof_options_desc, &buf[..]).ok()?;

    let ext_field = DESCRIPTOR_POOL.get_extension_by_name("seaorm.oneof")?;

    if !dynamic_msg.has_extension(&ext_field) {
        return None;
    }

    let ext_value = dynamic_msg.get_extension(&ext_field);

    convert_to_oneof_options(&ext_value)
}

// =============================================================================
// Value conversion helpers
// =============================================================================

/// Convert a prost-reflect Value to our MessageOptions type
fn convert_to_message_options(value: &Value) -> Option<seaorm::MessageOptions> {
    let msg = value.as_message()?;
    let mut result = seaorm::MessageOptions::default();

    if let Some(cow) = msg.get_field_by_name("table_name") {
        if let Value::String(s) = cow.as_ref() {
            result.table_name = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("skip") {
        if let Value::Bool(b) = cow.as_ref() {
            result.skip = *b;
        }
    }

    if let Some(cow) = msg.get_field_by_name("indexes") {
        if let Value::List(list) = cow.as_ref() {
            for item in list.iter() {
                if let Value::String(s) = item {
                    result.indexes.push(s.clone());
                }
            }
        }
    }

    if let Some(cow) = msg.get_field_by_name("relations") {
        if let Value::List(list) = cow.as_ref() {
            for item in list.iter() {
                if let Some(rel) = convert_to_relation_def(item) {
                    result.relations.push(rel);
                }
            }
        }
    }

    Some(result)
}

/// Convert a prost-reflect Value to a RelationDef
fn convert_to_relation_def(value: &Value) -> Option<seaorm::RelationDef> {
    let msg = value.as_message()?;
    let mut result = seaorm::RelationDef::default();

    if let Some(cow) = msg.get_field_by_name("name") {
        if let Value::String(s) = cow.as_ref() {
            result.name = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("type") {
        if let Value::EnumNumber(n) = cow.as_ref() {
            result.r#type = *n;
        }
    }

    if let Some(cow) = msg.get_field_by_name("related") {
        if let Value::String(s) = cow.as_ref() {
            result.related = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("foreign_key") {
        if let Value::String(s) = cow.as_ref() {
            result.foreign_key = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("references") {
        if let Value::String(s) = cow.as_ref() {
            result.references = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("through") {
        if let Value::String(s) = cow.as_ref() {
            result.through = s.clone();
        }
    }

    Some(result)
}

/// Convert a prost-reflect Value to our FieldOptions type
fn convert_to_field_options(value: &Value) -> Option<seaorm::FieldOptions> {
    let msg = value.as_message()?;
    let mut result = seaorm::FieldOptions::default();

    if let Some(cow) = msg.get_field_by_name("primary_key") {
        if let Value::Bool(b) = cow.as_ref() {
            result.primary_key = *b;
        }
    }

    if let Some(cow) = msg.get_field_by_name("auto_increment") {
        if let Value::Bool(b) = cow.as_ref() {
            result.auto_increment = *b;
        }
    }

    if let Some(cow) = msg.get_field_by_name("unique") {
        if let Value::Bool(b) = cow.as_ref() {
            result.unique = *b;
        }
    }

    if let Some(cow) = msg.get_field_by_name("nullable") {
        if let Value::Bool(b) = cow.as_ref() {
            result.nullable = *b;
        }
    }

    if let Some(cow) = msg.get_field_by_name("column_name") {
        if let Value::String(s) = cow.as_ref() {
            result.column_name = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("column_type") {
        if let Value::String(s) = cow.as_ref() {
            result.column_type = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("default_value") {
        if let Value::String(s) = cow.as_ref() {
            result.default_value = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("embed") {
        if let Value::Bool(b) = cow.as_ref() {
            result.embed = *b;
        }
    }

    if let Some(cow) = msg.get_field_by_name("has_one") {
        if let Value::String(s) = cow.as_ref() {
            result.has_one = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("has_many") {
        if let Value::String(s) = cow.as_ref() {
            result.has_many = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("belongs_to") {
        if let Value::String(s) = cow.as_ref() {
            result.belongs_to = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("belongs_to_from") {
        if let Value::String(s) = cow.as_ref() {
            result.belongs_to_from = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("belongs_to_to") {
        if let Value::String(s) = cow.as_ref() {
            result.belongs_to_to = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("has_many_via") {
        if let Value::String(s) = cow.as_ref() {
            result.has_many_via = s.clone();
        }
    }

    Some(result)
}

/// Convert a prost-reflect Value to our EnumOptions type
fn convert_to_enum_options(value: &Value) -> Option<seaorm::EnumOptions> {
    let msg = value.as_message()?;
    let mut result = seaorm::EnumOptions::default();

    if let Some(cow) = msg.get_field_by_name("name") {
        if let Value::String(s) = cow.as_ref() {
            result.name = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("db_type") {
        if let Value::String(s) = cow.as_ref() {
            result.db_type = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("skip") {
        if let Value::Bool(b) = cow.as_ref() {
            result.skip = *b;
        }
    }

    Some(result)
}

/// Convert a prost-reflect Value to our EnumValueOptions type
fn convert_to_enum_value_options(value: &Value) -> Option<seaorm::EnumValueOptions> {
    let msg = value.as_message()?;
    let mut result = seaorm::EnumValueOptions::default();

    if let Some(cow) = msg.get_field_by_name("name") {
        if let Value::String(s) = cow.as_ref() {
            result.name = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("string_value") {
        if let Value::String(s) = cow.as_ref() {
            result.string_value = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("int_value") {
        if let Value::I32(n) = cow.as_ref() {
            result.int_value = *n;
        }
    }

    Some(result)
}

/// Convert a prost-reflect Value to our OneofOptions type
fn convert_to_oneof_options(value: &Value) -> Option<seaorm::OneofOptions> {
    let msg = value.as_message()?;
    let mut result = seaorm::OneofOptions::default();

    if let Some(cow) = msg.get_field_by_name("strategy") {
        if let Value::String(s) = cow.as_ref() {
            result.strategy = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("column_prefix") {
        if let Value::String(s) = cow.as_ref() {
            result.column_prefix = s.clone();
        }
    }

    if let Some(cow) = msg.get_field_by_name("discriminator_column") {
        if let Value::String(s) = cow.as_ref() {
            result.discriminator_column = s.clone();
        }
    }

    Some(result)
}

// =============================================================================
// Fallback: Uninterpreted option parsing (for older protoc versions)
// =============================================================================

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
    // The name parts form a path like: (seaorm.model).table_name
    // or just (seaorm.model) for aggregate values
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

/// Get the sub-field name from an uninterpreted option (e.g., "table_name" from "(seaorm.model).table_name")
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
        // Individual field setting like (seaorm.model).table_name = "users"
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
        "RELATION_TYPE_HAS_ONE" | "has_one" | "HasOne" => seaorm::RelationType::HasOne as i32,
        "RELATION_TYPE_HAS_MANY" | "has_many" | "HasMany" => seaorm::RelationType::HasMany as i32,
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
