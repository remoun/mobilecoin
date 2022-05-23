//! Testing cryptography helpers.
//!
//! We assume signing, context changes, mutability, etc. is tested at lower
//! level, and just do a round-trip.

use mc_blockchain_types::{crypto::*, BlockID, BlockMetadataContents};
use mc_crypto_keys::Ed25519Pair;
use mc_transaction_core_test_utils::{make_quorum_set, make_verification_report};
use mc_util_from_random::{FromRandom, Rng};
use mc_util_test_helper::run_with_several_seeds;

#[test]
fn block_metadata() {
    run_with_several_seeds(|mut csprng| {
        let rng = &mut csprng;
        let block_id = BlockID(FromRandom::from_random(rng));
        let contents = BlockMetadataContents::new(
            block_id,
            Some(make_quorum_set(rng.gen_range(1..=42), rng)),
            Some(make_verification_report(rng)),
        );
        let signer = Ed25519Pair::from_random(rng);

        let sig = signer
            .sign_metadata(&contents)
            .expect("Could not sign metadata contents");

        signer
            .public_key()
            .verify_metadata(&contents, &sig)
            .expect("Could not verify signature over metadata contents");
    })
}
