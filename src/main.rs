//! protoc-gen-seaorm - A protoc plugin for generating SeaORM 2.0 entities
//!
//! This binary reads a CodeGeneratorRequest from stdin and writes a
//! CodeGeneratorResponse to stdout, following the protoc plugin protocol.

use prost::Message;
use prost_types::compiler::CodeGeneratorResponse;
use std::io::{self, Read, Write};

fn main() {
    if let Err(e) = run() {
        eprintln!("protoc-gen-seaorm: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Read raw bytes from stdin
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;

    // Generate code using the bytes-based entry point
    // This preserves extension data by using prost-reflect for initial decoding
    let response =
        protoc_gen_seaorm::generate_from_bytes(&buf).unwrap_or_else(|e| CodeGeneratorResponse {
            error: Some(e.to_string()),
            ..Default::default()
        });

    // Debug: print what we generated
    if std::env::var("SEAORM_DEBUG").is_ok() {
        eprintln!(
            "[protoc-gen-seaorm] Generated {} files",
            response.file.len()
        );
        for f in &response.file {
            eprintln!(
                "[protoc-gen-seaorm]   - {}",
                f.name.as_deref().unwrap_or("<unnamed>")
            );
        }
        if let Some(ref err) = response.error {
            eprintln!("[protoc-gen-seaorm] Error: {}", err);
        }
    }

    // Write CodeGeneratorResponse to stdout
    let mut out = Vec::new();
    response.encode(&mut out)?;
    io::stdout().write_all(&out)?;

    Ok(())
}
