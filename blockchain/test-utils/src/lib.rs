// Copyright (c) 2018-2022 The MobileCoin Foundation

//! Helpers for block-related tests.
#![deny(missing_docs)]

use mc_blockchain_types::{
    BlockID, BlockMetadata, BlockMetadataContents, QuorumNode, QuorumSet, VerificationReport,
};
use mc_crypto_keys::Ed25519Pair;
use mc_util_from_random::{random_bytes_vec, CryptoRng, FromRandom, Rng, RngCore, SeedableRng};
use mc_util_test_helper::RngType as FixedRng;

/// Deterministically creates a [QuorumNode] with the given ID number.
pub fn make_test_node(node_id: u32) -> QuorumNode {
    make_test_node_and_signer(node_id).0
}

/// Deterministically creates a [QuorumNode] and [Ed25519Pair] signer with the
/// given ID number.
pub fn make_test_node_and_signer(node_id: u32) -> (QuorumNode, Ed25519Pair) {
    let mut seed_bytes = [0u8; 32];
    let node_id_bytes = node_id.to_be_bytes();
    seed_bytes[..node_id_bytes.len()].copy_from_slice(&node_id_bytes[..]);
    let mut seeded_rng = FixedRng::from_seed(seed_bytes);

    let signer_keypair = Ed25519Pair::from_random(&mut seeded_rng);
    let public_key = signer_keypair.public_key();
    (
        QuorumNode {
            responder_id: format!("node{}.test.com:8443", node_id),
            public_key,
        },
        signer_keypair,
    )
}

/// Generate a [QuorumSet] with the specified number of randomly generated node
/// IDs.
pub fn make_quorum_set_with_count<RNG: RngCore + CryptoRng>(
    num_nodes: u32,
    rng: &mut RNG,
) -> QuorumSet {
    let threshold = rng.gen_range(1..=num_nodes);
    let node_ids = (0..num_nodes).map(make_test_node).collect();
    QuorumSet::new_with_node_ids(threshold, node_ids)
}

/// Generate a [QuorumSet] with a random number of randomly generated node IDs.
pub fn make_quorum_set<RNG: RngCore + CryptoRng>(rng: &mut RNG) -> QuorumSet {
    make_quorum_set_with_count(rng.gen_range(1..=42), rng)
}

/// Generate a [VerificationReport] from random bytes.
pub fn make_verification_report<RNG: RngCore + CryptoRng>(rng: &mut RNG) -> VerificationReport {
    let sig = random_bytes_vec(42, rng).into();
    let chain_len = rng.gen_range(2..42);
    let chain = (1..=chain_len)
        .map(|n| random_bytes_vec(n as usize, rng))
        .collect();
    VerificationReport {
        sig,
        chain,
        http_body: "testing".to_owned(),
    }
}

/// Generate a [BlockMetadata] for the given block ID, and otherwise random
/// contents.
pub fn make_block_metadata<RNG: RngCore + CryptoRng>(
    block_id: BlockID,
    rng: &mut RNG,
) -> BlockMetadata {
    let signer = Ed25519Pair::from_random(rng);
    let metadata = BlockMetadataContents::new(
        block_id,
        Some(make_quorum_set(rng)),
        Some(make_verification_report(rng)),
    );
    BlockMetadata::from_contents_and_keypair(metadata, &signer)
        .expect("BlockMetadata::from_contents_and_keypair")
}
