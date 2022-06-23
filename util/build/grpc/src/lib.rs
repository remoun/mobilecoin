// Copyright (c) 2018-2022 The MobileCoin Foundation

#![doc = include_str!("../README.md")]

use mc_util_build_script::rerun_if_path_changed;
use protobuf_codegen::{Codegen, Customize};
use std::path::{Path, PathBuf};

const GEN_DIR: &str = "protos-auto-gen";

/// Compile protobuf files into Rust code, and generate a mod.rs that references
/// all the generated modules.
/// Accepts any slice of paths, e.g. `&[&str]`, `Vec<&Path>`, `Vec<PathBuf>`
pub fn compile_protos_and_generate_mod_rs(
    proto_dirs: &[impl AsRef<Path>],
    proto_files: &[impl AsRef<Path>],
) {
    let proto_dirs: Vec<&Path> = proto_dirs.into_iter().map(AsRef::as_ref).collect();
    let proto_files: Vec<PathBuf> = proto_files
        .iter()
        .map(|p| make_absolute(p.as_ref(), &proto_dirs))
        .collect();

    // If the proto files change, we need to re-run.
    proto_dirs.iter().for_each(rerun_if_path_changed);

    // Generate code.
    Codegen::new()
        .includes(proto_dirs)
        .inputs(&proto_files)
        .pure()
        .cargo_out_dir(GEN_DIR)
        .customize(
            Customize::default()
                .gen_mod_rs(true)
                .generate_accessors(true)
                .generate_getter(true),
        )
        .run_from_script();
}

fn make_absolute(path: &Path, dirs: &[&Path]) -> PathBuf {
    for dir in dirs {
        let abs_path = dir.join(path);
        if abs_path.exists() {
            return abs_path;
        }
    }
    panic!("Could not find {:?} in {:?}", path, dirs);
}
