// Copyright (c) 2018-2022 The MobileCoin Foundation

//! Bake the compile-time target features into the enclave.

use cargo_emit::rerun_if_env_changed;
use mc_util_build_script::Environment;
use std::{env::var, fs};

const HEADER: &str =
    "// Copyright (c) 2018-2022 The MobileCoin Foundation\n\n// Auto-generated file\n\n";

// These public keys are associated with the private key used in the tests for
// consensus/enclave/impl. These are the hex-encoded public spend and view key
// bytes.
const DEFAULT_FEE_SPEND_PUB: &str =
    "26b507c63124a2f5e940b4fb89e4b2bb0a2078ed0c8e551ad59268b9646ec241";
const DEFAULT_FEE_VIEW_PUB: &str =
    "5222a1e9ae32d21c23114a5ce6bb39e0cb56aea350d4619d43b1207061b10346";

fn main() {
    let env = Environment::default();

    let mut target_features: Vec<String> = env
        .target_features()
        .iter()
        .map(ToOwned::to_owned)
        .collect();
    target_features.sort();

    let mut features_rs = HEADER.to_string() + "const TARGET_FEATURES: &[&str] = &[\n";
    for feature in target_features {
        features_rs.push_str("    \"");
        features_rs.push_str(&feature);
        features_rs.push_str("\",\n");
    }
    features_rs.push_str("];\n\n");

    let features_out = env.out_dir().join("target_features.rs");
    // Only write if the contents would change.
    if fs::read_to_string(&features_out).unwrap_or_default() != features_rs {
        fs::write(features_out, features_rs).expect("Could not write target feature array");
    }

    rerun_if_env_changed!("FEE_SPEND_PUBLIC_KEY");
    rerun_if_env_changed!("FEE_VIEW_PUBLIC_KEY");

    let mut fee_spend_public_key = [0u8; 32];
    let mut fee_view_public_key = [0u8; 32];

    // Check for env var and override
    fee_spend_public_key.copy_from_slice(
        &hex::decode(
            var("FEE_SPEND_PUBLIC_KEY").unwrap_or_else(|_| DEFAULT_FEE_SPEND_PUB.to_string()),
        )
        .expect("Failed parsing public spend key."),
    );
    fee_view_public_key.copy_from_slice(
        &hex::decode(
            var("FEE_VIEW_PUBLIC_KEY").unwrap_or_else(|_| DEFAULT_FEE_VIEW_PUB.to_string()),
        )
        .expect("Failed parsing public view key."),
    );

    let mut constants_rs = HEADER.to_string();
    constants_rs.push_str(&format!(
        "pub const FEE_SPEND_PUBLIC_KEY: [u8; 32] = {:?};\n\n",
        fee_spend_public_key
    ));
    constants_rs.push_str(&format!(
        "pub const FEE_VIEW_PUBLIC_KEY: [u8; 32] = {:?};\n",
        fee_view_public_key
    ));

    let constants_out = env.out_dir().join("constants.rs");
    // Only write if the contents would change.
    if fs::read_to_string(&constants_out).unwrap_or_default() != constants_rs {
        fs::write(constants_out, constants_rs).expect("Could not write constants.rs");
    }
}
