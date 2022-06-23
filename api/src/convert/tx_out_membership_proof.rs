//! Convert to/from external::TxOutMembershipProof

use crate::{external, ConversionError};
use mc_transaction_core::{
    membership_proofs::Range,
    tx::{TxOutMembershipElement, TxOutMembershipProof},
};

/// Convert TxOutMembershipProof -> external::MembershipProof.
impl From<&TxOutMembershipProof> for external::TxOutMembershipProof {
    fn from(tx_out_membership_proof: &TxOutMembershipProof) -> Self {
        let mut membership_proof = external::TxOutMembershipProof::new();
        membership_proof.set_index(tx_out_membership_proof.index);
        membership_proof.set_highest_index(tx_out_membership_proof.highest_index);

        let elements: Vec<external::TxOutMembershipElement> = tx_out_membership_proof
            .elements
            .iter()
            .map(external::TxOutMembershipElement::from)
            .collect();

        membership_proof.set_elements(elements);
        membership_proof
    }
}

/// Convert external::MembershipProof --> TxOutMembershipProof.
impl TryFrom<&external::TxOutMembershipProof> for TxOutMembershipProof {
    type Error = ConversionError;

    fn try_from(membership_proof: &external::TxOutMembershipProof) -> Result<Self, Self::Error> {
        let index: u64 = membership_proof.index();
        let highest_index: u64 = membership_proof.highest_index();

        let mut elements = Vec::<TxOutMembershipElement>::default();
        for element in membership_proof.elements() {
            let range = Range::new(element.range().from(), element.range().to())
                .map_err(|_e| ConversionError::Other)?;

            let bytes: &[u8] = element.hash().data();
            let mut hash = [0u8; 32];
            if bytes.len() != hash.len() {
                return Err(ConversionError::ArrayCastError);
            }
            hash.copy_from_slice(bytes);
            elements.push(TxOutMembershipElement::new(range, hash));
        }
        let tx_out_membership_proof = TxOutMembershipProof::new(index, highest_index, elements);
        Ok(tx_out_membership_proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Convert TxOutMembershipProof -> external::TxOutMembershipProof.
    fn test_membership_proof_from() {
        let index: u64 = 128_465;
        let highest_index: u64 = 781_384_772_994;
        let hashes = vec![
            // Add some arbitrary hashes.
            TxOutMembershipElement::new(Range::new(0, 1).unwrap(), [2u8; 32]),
            TxOutMembershipElement::new(Range::new(0, 3).unwrap(), [4u8; 32]),
            TxOutMembershipElement::new(Range::new(0, 7).unwrap(), [8u8; 32]),
        ];
        let tx_out_membership_proof =
            TxOutMembershipProof::new(index, highest_index, hashes.clone());

        let membership_proof = external::TxOutMembershipProof::from(&tx_out_membership_proof);
        assert_eq!(membership_proof.index(), index);
        assert_eq!(membership_proof.highest_index(), highest_index);

        let elements = membership_proof.elements();
        assert_eq!(elements.len(), hashes.len());

        for (idx, element) in elements.iter().enumerate() {
            let range = Range::new(element.range().from(), element.range().to()).unwrap();
            assert_eq!(range, hashes.get(idx).unwrap().range);
            let expected_hash = &hashes.get(idx).unwrap().hash;
            let bytes = element.hash().data();
            assert_eq!(bytes.len(), expected_hash.as_ref().len());
            assert_eq!(bytes, expected_hash.as_ref());
        }
    }
}
