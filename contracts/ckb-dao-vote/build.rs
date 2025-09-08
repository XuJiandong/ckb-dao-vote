use std::fs;
use std::process::Command;

fn compile(schema: &str) {
    let out_dir = std::path::PathBuf::from("./src/molecules");

    // Ensure the output directory exists
    if let Err(err) = fs::create_dir_all(&out_dir) {
        panic!("Failed to create output directory {:?}: {}", out_dir, err);
    }

    let mut compiler = molecule_codegen::Compiler::new();
    let result = compiler
        .input_schema_file(schema)
        .generate_code(molecule_codegen::Language::RustLazyReader)
        .output_dir(out_dir)
        .run();

    if let Err(err) = result {
        panic!("Failed to compile schema {}: {}", schema, err);
    }
}

fn main() {
    println!("cargo:rerun-if-changed=./molecules/vote.mol");
    println!("cargo:rerun-if-changed=./molecules/ckb.mol");
    compile("./molecules/vote.mol");
    compile("./molecules/ckb.mol");

    let output = Command::new("cargo")
        .arg("fmt")
        .arg("--")
        .arg("src/molecules/vote.rs")
        .arg("src/molecules/ckb.rs")
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        panic!("Command failed: {}", error);
    }
}
