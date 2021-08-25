mod autogenerated_code {
    // Expose proto data types from included third-party/external proto files.
    pub use mc_api::external;
    pub use mc_attest_api::attest;
    pub use mc_fog_api::{report, report_grpc};
    pub use protobuf::well_known_types::Empty;

    // Needed due to how to the auto-generated code references the Empty message.
    pub mod empty {
        pub use protobuf::well_known_types::Empty;
    }

    // Include the auto-generated code.
    include!(concat!(env!("OUT_DIR"), "/protos-auto-gen/mod.rs"));
}

pub use autogenerated_code::*;

pub mod report_parse;

use fog_uri::{IngestPeerUri, UriParseError};
use grpcio::{CallOption, Metadata};
use std::{collections::BTreeSet, str::FromStr};

// For tests, we need to implement Eq on view::QueryRequest
// They implement PartialEq but not Eq for some reason
impl Eq for autogenerated_code::view::QueryRequest {}
impl Eq for autogenerated_code::view::QueryRequestAAD {}
impl Eq for autogenerated_code::kex_rng::KexRngPubkey {}
impl Eq for autogenerated_code::kex_rng::StoredRng {}

// Extra functions for IngestSummary to avoid repetition
impl ingest_common::IngestSummary {
    pub fn get_sorted_peers(&self) -> Result<BTreeSet<IngestPeerUri>, UriParseError> {
        self.peers
            .iter()
            .map(|x| IngestPeerUri::from_str(x))
            .collect()
    }
}

// Implement the EnclaveGrpcChannel trait on attested service types.
// If we don't do this in this crate, then newtype wrappers must be used,
// because of orphan rules
use fog_enclave_connection::EnclaveGrpcChannel;

impl EnclaveGrpcChannel for view_grpc::FogViewApiClient {
    fn auth(
        &mut self,
        msg: &attest::AuthMessage,
        call_option: CallOption,
    ) -> Result<(Option<Metadata>, attest::AuthMessage, Option<Metadata>), grpcio::Error> {
        <Self>::auth_full(self, msg, call_option)
    }
    fn enclave_request(
        &mut self,
        msg: &attest::Message,
        call_option: CallOption,
    ) -> Result<(Option<Metadata>, attest::Message, Option<Metadata>), grpcio::Error> {
        <Self>::query_full(self, msg, call_option)
    }
}

impl EnclaveGrpcChannel for ledger_grpc::FogKeyImageApiClient {
    fn auth(
        &mut self,
        msg: &attest::AuthMessage,
        call_option: CallOption,
    ) -> Result<(Option<Metadata>, attest::AuthMessage, Option<Metadata>), grpcio::Error> {
        <Self>::auth_full(self, msg, call_option)
    }
    fn enclave_request(
        &mut self,
        msg: &attest::Message,
        call_option: CallOption,
    ) -> Result<(Option<Metadata>, attest::Message, Option<Metadata>), grpcio::Error> {
        <Self>::check_key_images_full(self, msg, call_option)
    }
}

impl EnclaveGrpcChannel for ledger_grpc::FogMerkleProofApiClient {
    fn auth(
        &mut self,
        msg: &attest::AuthMessage,
        call_option: CallOption,
    ) -> Result<(Option<Metadata>, attest::AuthMessage, Option<Metadata>), grpcio::Error> {
        <Self>::auth_full(self, msg, call_option)
    }
    fn enclave_request(
        &mut self,
        msg: &attest::Message,
        call_option: CallOption,
    ) -> Result<(Option<Metadata>, attest::Message, Option<Metadata>), grpcio::Error> {
        <Self>::get_outputs_full(self, msg, call_option)
    }
}
