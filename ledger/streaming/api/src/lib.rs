// Copyright (c) 2018-2022 The MobileCoin Foundation

//! Ledger Streaming API.

#![feature(assert_matches)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![deny(missing_docs)]

mod autogenerated_code {
    use mc_api::blockchain;

    // Include the auto-generated code.
    include!(concat!(env!("OUT_DIR"), "/protos-auto-gen/mod.rs"));
}
mod error;
mod traits;

#[cfg(any(test, feature = "test_utils"))]
pub mod test_utils;

pub use crate::{
    autogenerated_code::*,
    error::{Error, Result},
    traits::{Fetcher, Streamer},
};

pub use mc_api::blockchain::{ArchiveBlock, ArchiveBlocks};
pub use mc_blockchain_types::{BlockData, BlockIndex};
