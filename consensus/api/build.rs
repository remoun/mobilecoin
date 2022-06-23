// Copyright (c) 2018-2022 The MobileCoin Foundation

use mc_util_build_script::Environment;

fn main() {
    let env = Environment::default();

    let proto_dir = env.dir().join("proto");
    let proto_str = proto_dir.to_str().unwrap();
    // Other crates depend on this output for their deps.
    cargo_emit::pair!("PROTOS_PATH", "{}", proto_str);
    let mut all_proto_dirs = vec![proto_str];

    let api_proto_path = env
        .depvar("MC_API_PROTOS_PATH")
        .expect("Could not read mc_api's protos path");
    all_proto_dirs.extend(api_proto_path.split(':'));

    let attest_proto_path = env
        .depvar("MC_ATTEST_API_PROTOS_PATH")
        .expect("Could not read mc_attest_api's protos path");
    all_proto_dirs.extend(attest_proto_path.split(':'));

    mc_util_build_grpc::compile_protos_and_generate_mod_rs(
        all_proto_dirs.as_slice(),
        &[
            "consensus_client.proto",
            "consensus_common.proto",
            "consensus_config.proto",
            "consensus_peer.proto",
        ],
    );
}
