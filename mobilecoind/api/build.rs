// Copyright (c) 2018-2022 The MobileCoin Foundation

use mc_util_build_script::Environment;
use std::path::Path;

fn main() {
    let env = Environment::default();

    let proto_dir = env.dir().join("proto");
    // Other crates may depend on this output for their deps.
    cargo_emit::pair!("PROTOS_PATH", "{:?}", proto_dir);

    let api_proto_path = env
        .depvar("MC_API_PROTOS_PATH")
        .expect("Could not read mc_api's protos path");
    let consensus_api_proto_path = env
        .depvar("MC_CONSENSUS_API_PROTOS_PATH")
        .expect("Could not read mc_consensus_api's protos path");

    let mut all_proto_dirs = api_proto_path.split(':').collect::<Vec<&Path>>();
    all_proto_dirs.extend(consensus_api_proto_path.split(':'));
    all_proto_dirs.push(&proto_dir);

    mc_util_build_grpc::compile_protos_and_generate_mod_rs(
        all_proto_dirs.as_slice(),
        &["mobilecoind_api.proto"],
    );
}
