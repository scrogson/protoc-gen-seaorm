//! protoc-gen-seaorm - A protoc plugin for generating SeaORM 2.0 entities
//!
//! This binary reads a CodeGeneratorRequest from stdin and writes a
//! CodeGeneratorResponse to stdout, following the protoc plugin protocol.

use prost::Message;
use prost_types::compiler::{CodeGeneratorRequest, CodeGeneratorResponse};
use std::io::{self, Read, Write};

fn main() {
    if let Err(e) = run() {
        eprintln!("protoc-gen-seaorm: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Read CodeGeneratorRequest from stdin
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;

    let request = CodeGeneratorRequest::decode(&buf[..])?;

    // Generate code
    let response = protoc_gen_seaorm::generate(request).unwrap_or_else(|e| CodeGeneratorResponse {
        error: Some(e.to_string()),
        ..Default::default()
    });

    // Write CodeGeneratorResponse to stdout
    let mut out = Vec::new();
    response.encode(&mut out)?;
    io::stdout().write_all(&out)?;

    Ok(())
}
