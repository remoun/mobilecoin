// Copyright (c) 2018-2022 The MobileCoin Foundation

use mc_util_build_script::Environment;

fn main() {
    let env = Environment::default();

    let proto_dir = env.dir().join("proto");
    let proto_str = proto_dir.to_str().unwrap();
    // Other crates depend on this output for their deps.
    cargo_emit::pair!("PROTOS_PATH", "{}", proto_str);
    let mut all_proto_dirs = vec![proto_str];

    let attest_proto_path = env
        .depvar("MC_ATTEST_API_PROTOS_PATH")
        .expect("Could not read attest api's protos path");
    all_proto_dirs.extend(attest_proto_path.split(':'));

    let api_proto_path = env
        .depvar("MC_API_PROTOS_PATH")
        .expect("Could not read api's protos path");
    all_proto_dirs.extend(api_proto_path.split(':'));

    let mc_fog_report_api_proto_path = env
        .depvar("MC_FOG_REPORT_API_PROTOS_PATH")
        .expect("Could not read mc-fog-report-api's protos path");
    all_proto_dirs.extend(mc_fog_report_api_proto_path.split(':'));

    let consensus_api_proto_path = env
        .depvar("MC_CONSENSUS_API_PROTOS_PATH")
        .expect("Could not read consensus api's protos path");
    all_proto_dirs.extend(consensus_api_proto_path.split(':'));

    mc_util_build_grpc::compile_protos_and_generate_mod_rs(
        all_proto_dirs.as_slice(),
        &[
            "fog_common.proto",
            "ingest.proto",
            "ingest_common.proto",
            "ingest_peer.proto",
            "kex_rng.proto",
            "ledger.proto",
            "view.proto",
        ],
    );
}
