use std::{env::current_dir, fs};

fn main() {
    println!("cargo:rerun-if-changed=src/bridge");
    let files: Vec<_> = glob::glob("src/bridge/*.rs")
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect();

    let attrs = ["window::bridge", "bridge"];
    let output = current_dir()
        .expect("Failed to get current directory")
        .join("../../vue/app/src/generate");
    if !output.exists() {
        fs::create_dir_all(&output).expect("Failed to create output directory");
    }
    let output = output.join("bridge.ts");
    println!("{output:?}");
    window_generate::generate_ts(&files, &attrs, output).expect("Failed to generate TS bindings");
}
