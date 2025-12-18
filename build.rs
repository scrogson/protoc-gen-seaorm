//! Build script for protoc-gen-seaorm
//!
//! This compiles proto/seaorm/options.proto to generate the Rust types
//! for parsing SeaORM protobuf extensions.

use std::io::Result;

fn main() -> Result<()> {
    // Compile the seaorm options proto
    prost_build::Config::new()
        .compile_protos(&["proto/seaorm/options.proto"], &["proto"])?;

    // Re-run if the proto file changes
    println!("cargo:rerun-if-changed=proto/seaorm/options.proto");

    Ok(())
}
