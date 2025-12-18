//! protoc-gen-seaorm library
//!
//! This crate provides the code generation logic for converting Protocol Buffer
//! definitions into SeaORM 2.0 entity models.

#![deny(warnings)]
#![deny(missing_docs)]

pub mod codegen;
pub mod generator;
pub mod options;
pub mod types;

use prost_types::compiler::{CodeGeneratorRequest, CodeGeneratorResponse};
use thiserror::Error;

/// Errors that can occur during code generation
#[derive(Error, Debug)]
pub enum GeneratorError {
    /// Failed to parse protobuf options/extensions
    #[error("Failed to parse options: {0}")]
    OptionsParseError(String),

    /// Encountered an unknown or unsupported field type
    #[error("Unknown field type: {0}")]
    UnknownFieldType(String),

    /// Invalid plugin configuration or parameters
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// General code generation failure
    #[error("Code generation failed: {0}")]
    CodeGenError(String),

    /// Failed to decode protobuf message
    #[error("Decode error: {0}")]
    DecodeError(String),
}

/// Generate SeaORM entities from a protobuf CodeGeneratorRequest
///
/// This is the main entry point for the code generator.
pub fn generate(request: CodeGeneratorRequest) -> Result<CodeGeneratorResponse, GeneratorError> {
    generator::generate(request)
}

/// Generate SeaORM entities from raw protobuf bytes
///
/// This entry point preserves extension data by using prost-reflect for decoding.
pub fn generate_from_bytes(bytes: &[u8]) -> Result<CodeGeneratorResponse, GeneratorError> {
    generator::generate_from_bytes(bytes)
}
