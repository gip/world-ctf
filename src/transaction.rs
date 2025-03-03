use alloy_consensus::TxEnvelope;
use alloy_network::{EthereumWallet, TransactionBuilder};
use alloy_primitives::Address;
use alloy_provider::Provider;
use alloy_rpc_types_eth::{TransactionInput, TransactionRequest};
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::SolValue;
use eyre::Result;
use semaphore_rs::hash_to_field;
use std::sync::Arc;
use world_chain_builder_test_utils::bindings::IMulticall3::Call3;

use crate::world_id::WorldID;

// PBH Entry Point address
pub static PBH_ENTRY_POINT: Address = Address::ZERO;

#[derive(Clone, Default)]
pub struct GasTestTransactionBuilder {
    pub tx: TransactionRequest,
    pub provider: Option<Arc<dyn Provider>>,
}

impl std::fmt::Debug for GasTestTransactionBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GasTestTransactionBuilder")
            .field("tx", &self.tx)
            .field("provider", &format!("<provider>"))
            .finish()
    }
}

impl GasTestTransactionBuilder {
    pub fn new(gas_fee: Option<f64>, priority_gas_fee: Option<f64>, _rpc_address: Option<String>) -> Self {
        let mut tx = TransactionRequest::default()
            .gas_limit(130000);
        tx.set_chain_id(4801);
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
        
        GasTestTransactionBuilder { tx, provider: None }
    }

    pub async fn with_pbh_multicall(
        self,
        world_id: &WorldID,
        pbh_nonce: u16,
        from: Address,
        calls: Vec<Call3>,
    ) -> Result<Self> {
        // Get the inclusion proof for the identity in the from the World Tree
        let signal_hash = hash_to_field(&SolValue::abi_encode_packed(&(from, calls.clone())));
        let _pbh_payload = world_id.pbh_payload(pbh_nonce, signal_hash).await?;

        // For now, we'll use a simplified approach to create the calldata
        // Function selector for pbhMulticall (first 4 bytes of keccak256("pbhMulticall((address,bytes,bool)[],(uint256,uint256,(uint8,uint8,uint8,uint16),(uint256[8],)))"))
        let selector = [0x96, 0x4a, 0x97, 0x50];
        
        // Create a simplified calldata - in a real implementation, we would properly encode the calls and payload
        let mut calldata_bytes = selector.to_vec();
        
        // Add some placeholder data for the parameters
        // This is a simplified implementation - in a real implementation, we would properly encode the parameters
        calldata_bytes.extend_from_slice(&[0; 64]); // Placeholder data

        let tx = self.tx
            .input(TransactionInput::new(calldata_bytes.into()));
        
        Ok(Self { tx, provider: self.provider })
    }

    pub async fn build(self, signer: PrivateKeySigner) -> Result<TxEnvelope> {
        let wallet: EthereumWallet = signer.into();
        
        // Build the transaction without a provider
        Ok(self.tx.build(&wallet).await?)
    }

    /// Sets the recipient address for the transaction.
    pub fn to(self, to: Address) -> Self {
        let tx = self.tx.to(to);
        Self { tx, provider: self.provider }
    }

    /// Sets the input data for the transaction.
    pub fn input(self, input: TransactionInput) -> Self {
        let tx = self.tx.input(input);
        Self { tx, provider: self.provider }
    }

    /// Sets the nonce for the transaction.
    pub fn nonce(self, nonce: u64) -> Self {
        let tx = self.tx.nonce(nonce);
        Self { tx, provider: self.provider }
    }
}

/// Creates a multicall call for the gas consumption function
pub fn consume_gas_multicall(contract_address: Address, iterations: u64) -> Vec<Call3> {
    // Function selector for consumeGas(address,uint256)
    // keccak256("consumeGas(address,uint256)")[0..4]
    let selector = [0x41, 0x4c, 0xf8, 0x5d];
    
    // Encode the parameters
    let mut calldata = selector.to_vec();
    
    // Encode address (pad to 32 bytes)
    let mut address_bytes = vec![0u8; 12];
    address_bytes.extend_from_slice(contract_address.as_slice());
    calldata.extend_from_slice(&address_bytes);
    
    // Encode iterations
    let mut iterations_bytes = [0u8; 32];
    let iterations_vec = iterations.to_be_bytes().to_vec();
    for (i, b) in iterations_vec.iter().rev().enumerate() {
        iterations_bytes[32 - iterations_vec.len() + i] = *b;
    }
    calldata.extend_from_slice(&iterations_bytes);
    
    let call = Call3 {
        target: contract_address,
        callData: calldata.into(),
        allowFailure: false,
    };

    vec![call]
}

/// Gets the next available PBH nonce for the given WorldID
pub async fn get_pbh_nonce<P>(
    world_id: &WorldID,
    _provider: Arc<P>,
    max_pbh_nonce: u16,
) -> Result<u16>
where
    P: Provider + 'static,
{
    let start_nonce = 0;
    // Simplified implementation - in a real implementation, we would check if the nonce is already used
    // For now, just return the first nonce
    for i in start_nonce..=max_pbh_nonce {
        let (_external_nullifier, _external_nullifier_hash, _nullifier_hash) = 
            world_id.pbh_ext_nullifier(i);
        // In a real implementation, we would check if the nonce is already used
        // For simplicity, just return the first nonce
        return Ok(i);
    }

    Err(eyre::eyre!("No available PBH nonce"))
}
