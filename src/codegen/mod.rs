//! Code generation modules for SeaORM entities
//!
//! This module contains the code generation logic for creating
//! SeaORM 2.0 entity definitions from Protocol Buffer messages.

pub mod column;
pub mod entity;
pub mod enum_gen;
pub mod oneof;
pub mod relation;

use crate::GeneratorError;
use prost_types::compiler::code_generator_response::File;
use prost_types::{DescriptorProto, EnumDescriptorProto, FileDescriptorProto};

/// Generate a SeaORM entity from a protobuf message
///
/// Returns None if the message should be skipped (no seaorm options)
pub fn generate_entity(
    file: &FileDescriptorProto,
    message: &DescriptorProto,
) -> Result<Option<File>, GeneratorError> {
    entity::generate(file, message)
}

/// Generate a SeaORM enum from a protobuf enum definition
///
/// Returns None if the enum should be skipped (no seaorm options)
pub fn generate_enum(
    file: &FileDescriptorProto,
    enum_desc: &EnumDescriptorProto,
) -> Result<Option<File>, GeneratorError> {
    enum_gen::generate(file, enum_desc)
}
