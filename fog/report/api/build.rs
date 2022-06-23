// Copyright (c) 2018-2022 The MobileCoin Foundation

use mc_util_build_script::Environment;
use std::path::Path;

fn main() {
    let env = Environment::default();

    let proto_dir = env.dir().join("proto");
    // Other crates depend on this output for their deps.
    cargo_emit::pair!("PROTOS_PATH", "{:?}", proto_dir);

    let attest_proto_path = env
        .depvar("MC_ATTEST_API_PROTOS_PATH")
        .expect("Could not read attest api's protos path");
    let mut all_proto_dirs = attest_proto_path.split(':').collect::<Vec<&Path>>();
    all_proto_dirs.push(&proto_dir);

    let api_proto_path = env
        .depvar("MC_API_PROTOS_PATH")
        .expect("Could not read api's protos path");
    all_proto_dirs.extend(api_proto_path.split(':'));

    let consensus_api_proto_path = env
        .depvar("MC_CONSENSUS_API_PROTOS_PATH")
        .expect("Could not read consensus api's protos path");
    all_proto_dirs.extend(consensus_api_proto_path.split(':'));

    mc_util_build_grpc::compile_protos_and_generate_mod_rs(
        all_proto_dirs.as_slice(),
        &["report.proto"],
    );
}
