extern crate glob;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use glob::glob;

fn main() {
    let root_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let proto_dir = root_dir.join("protos");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let generated_dir = out_dir.join("protos");

    let _ = fs::create_dir(&generated_dir);

    let proto_files: Vec<PathBuf> = glob(proto_dir.join("*.proto").to_str().unwrap())
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect();

    let protoc_status = Command::new("protoc")
        .arg(format!("--rust_out={}", generated_dir.to_str().unwrap()))
        .arg(format!("--proto_path={}", proto_dir.to_str().unwrap()))
        .args(&proto_files)
        .status()
        .unwrap_or_else(|e| panic!("failed to execute protoc: {}", e));

    if !protoc_status.success() {
        panic!("An error occured with protoc: {}", protoc_status)
    }

    let generated_files: Vec<PathBuf> = glob(generated_dir.join("*.rs").to_str().unwrap())
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect();

    let perl_status = Command::new("perl")
        .arg("-pi")
        .arg("-e")
        .arg("s/#!.*//")
        .args(&generated_files)
        .status()
        .unwrap_or_else(|e| panic!("Failed to execute perl: {}", e));

    if !perl_status.success() {
        panic!("An error occured with perl: {}", perl_status)
    }
}
