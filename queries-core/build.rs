extern crate glob;

use std::env;
use std::path::PathBuf;
use std::process::Command;

use glob::glob;

fn main() {
    let root_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let proto_dir = root_dir.join("proto");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let proto_files: Vec<PathBuf> = glob(proto_dir.join("*.proto").to_str().unwrap())
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect();

    Command::new("protoc")
        .arg(format!("--rust_out={}", out_dir.to_str().unwrap()))
        .arg(format!("--proto_path={}", proto_dir.to_str().unwrap()))
        .args(&proto_files)
        .status()
        .unwrap_or_else(|e| {
            panic!("failed to execute protoc: {}", e)
        });

    let generated_files: Vec<PathBuf> = glob(out_dir.join("*.rs").to_str().unwrap())
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect();

    Command::new("perl")
        .arg("-pi")
        .arg("-e")
        .arg("s/#!.*//")
        .args(&generated_files)
        .status()
        .unwrap_or_else(|e| {
            panic!("failed to execute perl: {}", e)
        });
}
