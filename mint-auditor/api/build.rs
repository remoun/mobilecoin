// Copyright (c) 2018-2022 The MobileCoin Foundation

use mc_util_build_script::Environment;

fn main() {
    let env = Environment::default();

    let proto_dir = env.dir().join("proto");
    let proto_str = proto_dir.to_str().unwrap();
    // Other crates may depend on this output for their deps.
    cargo_emit::pair!("PROTOS_PATH", "{}", proto_str);

    mc_util_build_grpc::compile_protos_and_generate_mod_rs(&[proto_dir], &["mint_auditor.proto"]);
}
