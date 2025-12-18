//! Build script for protoc-gen-seaorm
//!
//! This compiles proto/seaorm/options.proto to generate the Rust types
//! for parsing SeaORM protobuf extensions. It also generates a file
//! descriptor set for use with prost-reflect to parse extension fields.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    println!("cargo:rerun-if-changed=proto/seaorm/options.proto");

    // Compile options.proto to Rust types
    prost_build::Config::new()
        .out_dir(&out_dir)
        .compile_protos(&["proto/seaorm/options.proto"], &["proto/"])?;

    // Generate a FileDescriptorSet for prost-reflect
    let fds_path = out_dir.join("file_descriptor_set.bin");

    // Find the protobuf include path
    let protobuf_include = Command::new("brew")
        .args(["--prefix", "protobuf"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| format!("{}/include", s.trim()))
        .unwrap_or_else(|| "/usr/local/include".to_string());

    let status = Command::new("protoc")
        .args([
            "--descriptor_set_out",
            fds_path.to_str().unwrap(),
            "--include_imports",
            "--include_source_info",
            "-Iproto",
            &format!("-I{}", protobuf_include),
            "seaorm/options.proto",
            "google/protobuf/compiler/plugin.proto",
        ])
        .status()?;

    if !status.success() {
        return Err("protoc failed to generate file descriptor set".into());
    }

    Ok(())
}
