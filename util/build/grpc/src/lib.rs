// Copyright (c) 2018-2022 The MobileCoin Foundation

#![doc = include_str!("../README.md")]

use mc_util_build_script::Environment;
use std::{fs, path::Path};

/// Compile protobuf files into Rust code, and generate a mod.rs that references
/// all the generated modules.
pub fn compile_protos_and_generate_mod_rs(
    proto_dirs: &[impl AsRef<Path>],
    proto_files: &[impl AsRef<Path>],
) {
    // If the proto files change, we need to re-run.
    for path in proto_dirs {
        mc_util_build_script::rerun_if_path_changed(path);
    }

    // Output directory for generated code.
    let env = Environment::default();
    let output_destination = env.out_dir().join("protos-auto-gen");

    // Delete old code and create output directory.
    let _ = fs::remove_dir_all(&output_destination);
    fs::create_dir_all(&output_destination).expect("failed creating output destination");

    // Generate code.
    tonic_build::configure()
        .out_dir(output_destination)
        .include_file("mod.rs")
        .compile(proto_files, proto_dirs)
        .expect("Failed to compile gRPC definitions!");
}
