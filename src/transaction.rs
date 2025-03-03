use alloy_consensus::TxEnvelope;
use alloy_network::{EthereumWallet, TransactionBuilder};
use alloy_primitives::Address;
use alloy_rpc_types_eth::{TransactionInput, TransactionRequest};
use alloy_signer_local::PrivateKeySigner;
use eyre::Result;

#[derive(Debug, Clone, Default)]
pub struct GasTestTransactionBuilder {
    pub tx: TransactionRequest,
}

impl GasTestTransactionBuilder {
    pub fn new(gas_fee: Option<f64>, priority_gas_fee: Option<f64>) -> Self {
        let mut tx = TransactionRequest::default()
            .gas_limit(130000);
        
        // Set gas fees if provided
        if let Some(gas_fee) = gas_fee {
            tx = tx.max_fee_per_gas((gas_fee * 1e9) as u128);
        } else {
            tx = tx.max_fee_per_gas(1e8 as u128);
        }
        
        if let Some(priority_gas_fee) = priority_gas_fee {
            tx = tx.max_priority_fee_per_gas((priority_gas_fee * 1e9) as u128);
        } else {
            tx = tx.max_priority_fee_per_gas(1e8 as u128);
        }
        
        GasTestTransactionBuilder { tx }
    }

    pub async fn build(self, signer: PrivateKeySigner) -> Result<TxEnvelope> {
        Ok(self.tx.build::<EthereumWallet>(&signer.into()).await?)
    }

    /// Sets the recipient address for the transaction.
    pub fn to(self, to: Address) -> Self {
        let tx = self.tx.to(to);
        Self { tx }
    }

    /// Sets the input data for the transaction.
    pub fn input(self, input: TransactionInput) -> Self {
        let tx = self.tx.input(input);
        Self { tx }
    }
}
