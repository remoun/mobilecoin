//! Convert to/from external::TxPrefix.

use crate::{external, ConversionError};
use mc_transaction_core::tx;

/// Convert tx::TxPrefix --> external::TxPrefix.
impl From<&tx::TxPrefix> for external::TxPrefix {
    fn from(source: &tx::TxPrefix) -> Self {
        let mut tx_prefix = external::TxPrefix::new();

        let inputs: Vec<external::TxIn> = source.inputs.iter().map(external::TxIn::from).collect();
        tx_prefix.set_inputs(inputs.into());

        let outputs: Vec<external::TxOut> =
            source.outputs.iter().map(external::TxOut::from).collect();
        tx_prefix.set_outputs(outputs.into());

        tx_prefix.set_fee(source.fee);

        tx_prefix.set_fee_token_id(source.fee_token_id);

        tx_prefix.set_tombstone_block(source.tombstone_block);

        tx_prefix
    }
}

/// Convert external::TxPrefix --> tx::TxPrefix.
impl TryFrom<&external::TxPrefix> for tx::TxPrefix {
    type Error = ConversionError;

    fn try_from(source: &external::TxPrefix) -> Result<Self, Self::Error> {
        let mut inputs: Vec<tx::TxIn> = Vec::new();
        for out in source.inputs() {
            let tx_out = tx::TxIn::try_from(out)?;
            inputs.push(tx_out);
        }

        let mut outputs: Vec<tx::TxOut> = Vec::new();
        for out in source.outputs() {
            let tx_out = tx::TxOut::try_from(out)?;
            outputs.push(tx_out);
        }

        let tx_prefix = tx::TxPrefix {
            inputs,
            outputs,
            fee: source.fee(),
            fee_token_id: source.fee_token_id(),
            tombstone_block: source.tombstone_block(),
        };
        Ok(tx_prefix)
    }
}
