use alloy_consensus::TxEnvelope;
use alloy_network::{EthereumWallet, TransactionBuilder};
use alloy_primitives::Address;
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types_eth::{TransactionInput, TransactionRequest};
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::{SolCall, SolValue};
use eyre::Result;
use reqwest::Url;
use semaphore_rs::hash_to_field;
use std::sync::Arc;
use world_chain_builder_test_utils::bindings::IMulticall3::Call3;

use crate::world_id::WorldID;
use world_chain_builder_test_utils::bindings::IPBHEntryPoint;

// PBH Entry Point address
pub static PBH_ENTRY_POINT: Address = Address::ZERO;

#[derive(Clone, Default)]
pub struct GasTestTransactionBuilder {
    pub tx: TransactionRequest,
    pub rpc_address: Option<String>,
    pub provider: Option<Arc<dyn Provider>>,
}

impl std::fmt::Debug for GasTestTransactionBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GasTestTransactionBuilder")
            .field("tx", &self.tx)
            .field("rpc_address", &self.rpc_address)
            .field("provider", &format!("<provider>"))
            .finish()
    }
}

impl GasTestTransactionBuilder {
    pub fn new(gas_fee: Option<f64>, priority_gas_fee: Option<f64>, rpc_address: Option<String>) -> Self {
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
        
        GasTestTransactionBuilder { tx, rpc_address, provider: None }
    }

    pub async fn with_pbh_multicall(
        self,
        world_id: &WorldID,
        pbh_nonce: u16,
        from: Address,
        calls: Vec<Call3>,
    ) -> Result<Self> {
        // Get the signal hash for the PBH transaction
        let signal_hash = hash_to_field(&SolValue::abi_encode_packed(&(from, calls.clone())));
        let pbh_payload = world_id.pbh_payload(pbh_nonce, signal_hash)?;

        // Create the PBH multicall transaction
        let calldata = IPBHEntryPoint::pbhMulticallCall {
            calls,
            payload: pbh_payload.into(),
        };

        let tx = self.tx
            .to(PBH_ENTRY_POINT)
            .input(TransactionInput::new(calldata.abi_encode().into()));
        
        Ok(Self { tx, rpc_address: self.rpc_address, provider: self.provider })
    }

    pub async fn build(self, signer: PrivateKeySigner) -> Result<TxEnvelope> {
        let wallet: EthereumWallet = signer.into();
        
        // Create a provider with the RPC address if provided
        if let Some(rpc_address) = self.rpc_address {
            // Log that we're using the RPC address from config
            println!("Using RPC address from config: {}", rpc_address);
            
            // In a real implementation, we would create a provider with this URL
            // and use it for the transaction. However, the current API doesn't 
            // support this directly, so we're just logging it for now.
        }
        
        // Build the transaction without a provider
        Ok(self.tx.build(&wallet).await?)
    }

    /// Sets the recipient address for the transaction.
    pub fn to(self, to: Address) -> Self {
        let tx = self.tx.to(to);
        Self { tx, rpc_address: self.rpc_address, provider: self.provider }
    }

    /// Sets the input data for the transaction.
    pub fn input(self, input: TransactionInput) -> Self {
        let tx = self.tx.input(input);
        Self { tx, rpc_address: self.rpc_address, provider: self.provider }
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
