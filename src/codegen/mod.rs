//! Code generation modules for SeaORM entities and storage traits
//!
//! This module contains the code generation logic for creating
//! SeaORM 2.0 entity definitions and storage traits from Protocol Buffer messages.

pub mod column;
pub mod domain;
pub mod entity;
pub mod enum_gen;
pub mod oneof;
pub mod relation;
pub mod service;

use crate::GeneratorError;
use prost_types::compiler::code_generator_response::File;
use prost_types::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorProto, ServiceDescriptorProto,
};

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

/// Generate a Storage trait from a protobuf service definition
///
/// Returns None if the service should be skipped (no seaorm options or generate_storage is false)
pub fn generate_service(
    file: &FileDescriptorProto,
    service: &ServiceDescriptorProto,
) -> Result<Option<File>, GeneratorError> {
    service::generate(file, service)
}

/// Generate a domain type with garde validation from a protobuf message
///
/// Returns None if the message has no input options
pub fn generate_domain(
    file: &FileDescriptorProto,
    message: &DescriptorProto,
) -> Result<Option<File>, GeneratorError> {
    domain::generate(file, message)
}
