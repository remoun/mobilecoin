// Copyright (c) 2018-2022 The MobileCoin Foundation

use alloc::vec::Vec;
use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(
    Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Digestible,
)]
/// Hash of a Tx.
pub struct TxHash(pub [u8; TxHash::BYTE_LENGTH]);

impl TxHash {
    /// Transaction hash length, in bytes.
    pub const BYTE_LENGTH: usize = 32;

    #[inline]
    /// A reference to the underlying byte array.
    pub fn as_bytes(&self) -> &[u8; TxHash::BYTE_LENGTH] {
        &self.0
    }

    #[inline]
    /// Copies `self` to a new Vec.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl core::ops::Deref for TxHash {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<&[u8; TxHash::BYTE_LENGTH]> for TxHash {
    #[inline]
    fn from(a: &[u8; TxHash::BYTE_LENGTH]) -> Self {
        Self(*a)
    }
}

impl From<[u8; TxHash::BYTE_LENGTH]> for TxHash {
    #[inline]
    fn from(a: [u8; TxHash::BYTE_LENGTH]) -> Self {
        Self(a)
    }
}

impl<'bytes> TryFrom<&'bytes [u8]> for TxHash {
    type Error = ();

    #[inline]
    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        if src.len() != TxHash::BYTE_LENGTH {
            return Err(());
        }
        let mut bytes = [0u8; TxHash::BYTE_LENGTH];
        bytes.copy_from_slice(src);
        Ok(Self::from(bytes))
    }
}

impl Display for TxHash {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", hex_fmt::HexFmt(&self.0[0..6]))
    }
}

impl Debug for TxHash {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Tx#{}", self)
    }
}
