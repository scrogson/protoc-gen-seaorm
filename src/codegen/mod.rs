//! Code generation modules for SeaORM entities
//!
//! This module contains the code generation logic for creating
//! SeaORM 2.0 entity definitions from Protocol Buffer messages.

pub mod column;
pub mod entity;
pub mod relation;

use crate::GeneratorError;
use prost_types::compiler::code_generator_response::File;
use prost_types::{DescriptorProto, FileDescriptorProto};

/// Generate a SeaORM entity from a protobuf message
///
/// Returns None if the message should be skipped (no seaorm options)
pub fn generate_entity(
    file: &FileDescriptorProto,
    message: &DescriptorProto,
) -> Result<Option<File>, GeneratorError> {
    entity::generate(file, message)
}
