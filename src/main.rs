use alloy_network::{eip2718::Encodable2718};
use alloy_primitives::{Address, Bytes, U256};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types_eth::TransactionInput;
use alloy_signer_local::PrivateKeySigner;
use clap::Parser;
use eyre::Result;
use reqwest::Url;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::str::FromStr;
use tiny_keccak::{Keccak, Hasher};

mod transaction;
mod world_id;
mod bindings;

use transaction::{GasTestTransactionBuilder, consume_gas_multicall};
use world_id::WorldID;

// Configuration from TOML file
#[derive(Deserialize, Debug)]
struct Config {
    contract_address: String,
    world_id: String,
    rpc_address: String,
}

// Command line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Number of iterations for gas consumption
    #[clap(long)]
    iterations: u64,
    
    /// RPC provider URI (overrides config file)
    #[clap(long)]
    provider_uri: Option<String>,
    
    /// PBH Entry Point contract address
    #[clap(long, default_value = "0x6e37bAB9d23bd8Bdb42b773C58ae43C6De43A590")]
    pbh_entry_point: String,
    
    /// Gas fee in Gwei
    #[clap(long)]
    gas_fee: Option<f64>,
    
    /// Priority gas fee in Gwei
    #[clap(long)]
    priority_gas_fee: Option<f64>,
    
    /// Path to configuration file
    #[clap(long, default_value = "config.toml")]
    config_file: String,
    
    /// Use PBH transaction instead of direct transaction
    #[clap(long)]
    use_pbh: bool,
    
    /// PBH nonce (only used with --use-pbh)
    #[clap(long, default_value = "0")]
    pbh_nonce: u16,
}


pub fn consume_gas_calldata(address: &Address, iterations: U256) -> Bytes {
    // Calculate function selector for consumeGas(address,uint256)
    let mut keccak = Keccak::v256();
    let mut hash = [0u8; 32];
    keccak.update(b"consumeGas(address,uint256)");
    keccak.finalize(&mut hash);
    
    // Take first 4 bytes of the hash as function selector
    let function_selector = &hash[0..4];
    
    // Create buffer for the calldata
    let mut calldata = Vec::with_capacity(4 + 32 + 32); // 4 bytes for selector + 32 bytes for address + 32 bytes for iterations
    
    // Add function selector
    calldata.extend_from_slice(function_selector);
    
    // Add address parameter (padded to 32 bytes)
    let mut address_bytes = [0u8; 32];
    address_bytes[12..32].copy_from_slice(address.as_slice());
    calldata.extend_from_slice(&address_bytes);
    
    // Add iterations parameter (padded to 32 bytes)
    let mut iterations_bytes = [0u8; 32];
    iterations_bytes.copy_from_slice(&iterations.to_be_bytes::<32>());
    calldata.extend_from_slice(&iterations_bytes);
    
    calldata.into() // Convert Vec<u8> to Bytes
}


#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Read configuration from TOML file
    let config_path = Path::new(&args.config_file);
    if !config_path.exists() {
        return Err(eyre::eyre!("Configuration file not found: {}", args.config_file));
    }
    
    let config_content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_content)?;
    
    // Get private key from environment variable
    let private_key = env::var("PRIVATE_KEY")
        .map_err(|_| eyre::eyre!("PRIVATE_KEY environment variable not set"))?;
    
    // Parse the private key
    let signer = private_key.parse::<PrivateKeySigner>()?;
    
    // Convert string addresses to Address type
    let contract_address = config.contract_address.parse::<Address>()?;
    let pbh_entry_point = args.pbh_entry_point.parse::<Address>()?;
    
    // Create calldata for the consumeGas function
    let iterations = U256::from(args.iterations);
    let calldata = consume_gas_calldata(&contract_address, iterations);
    
    println!("Gas Test Application");
    println!("-------------------");
    println!("Contract Address: {}", contract_address);
    println!("PBH Entry Point: {}", pbh_entry_point);
    println!("Iterations: {}", iterations);
    
    // Print gas fee information if provided
    if let Some(gas_fee) = args.gas_fee {
        println!("Gas Fee: {} Gwei", gas_fee);
    }
    
    if let Some(priority_gas_fee) = args.priority_gas_fee {
        println!("Priority Gas Fee: {} Gwei", priority_gas_fee);
    }
    
    // Print transaction type
    if args.use_pbh {
        println!("Transaction Type: PBH");
        println!("PBH Nonce: {}", args.pbh_nonce);
    } else {
        println!("Transaction Type: Direct");
    }
    
    println!();
    println!("Sending transaction to the contract...");
    
    // Determine which RPC address to use (command line takes precedence over config)
    let rpc_address = args.provider_uri.or(Some(config.rpc_address.clone()));
    
    // Create a provider with the RPC address if provided
    let provider = if let Some(rpc_uri) = rpc_address.clone() {
        println!("Using RPC address: {}", rpc_uri);
        let provider = ProviderBuilder::new()
            .on_http(rpc_uri.parse::<Url>()?);
        Some(Arc::new(provider))
    } else {
        println!("No RPC address provided, transaction will not be sent");
        None
    };
    
    // Create and send the transaction
    let tx = if args.use_pbh {
        // Print transaction type
        println!("Transaction Type: PBH");
        
        // Create a WorldID from the world_id in the config
        let world_id = WorldID::new(&config.world_id)?;
        
        // Define the PBH entry point address - this is used in the transaction
        let _pbh_entry_point = Address::from_str("0x6e37bAB9d23bd8Bdb42b773C58ae43C6De43A590").unwrap();
        
        // Get the PBH nonce limit and the next available nonce if a provider is available
        let pbh_nonce = if let Some(_provider_ref) = provider.as_ref() {
            // For simplicity, use a hardcoded PBH nonce limit
            let _pbh_nonce_limit: u16 = 65535;
            
            // For now, just use the provided nonce since we have provider compatibility issues
            println!("Using provided PBH Nonce: {}", args.pbh_nonce);
            args.pbh_nonce
        } else {
            println!("No provider available, using provided PBH Nonce: {}", args.pbh_nonce);
            args.pbh_nonce
        };
        
        // Create a multicall for the consumeGas function
        let calls = consume_gas_multicall(contract_address, args.iterations);
        
        // Create a transaction builder
        let mut tx_builder = GasTestTransactionBuilder::new(args.gas_fee, args.priority_gas_fee, None);
        
        // Get the account nonce if a provider is available
        if let Some(provider_ref) = provider.as_ref() {
            // Get the current nonce for the signer's address
            let account_nonce = provider_ref.get_transaction_count(signer.address()).await?;
            println!("Using account nonce: {}", account_nonce);
            
            // Set the nonce on the transaction builder
            tx_builder = tx_builder.nonce(account_nonce);
        } else {
            println!("No provider available, using default nonce");
        }
        
        // Create and send a PBH transaction
        tx_builder
            .with_pbh_multicall(&world_id, pbh_nonce, signer.address(), calls)
            .await?
            .build(signer)
            .await?
    } else {
        // Print transaction type
        println!("Transaction Type: Direct");
        
        // Create a transaction builder
        let mut tx_builder = GasTestTransactionBuilder::new(args.gas_fee, args.priority_gas_fee, None);
        
        // Get the account nonce if a provider is available
        if let Some(provider_ref) = provider.as_ref() {
            // Get the current nonce for the signer's address
            let account_nonce = provider_ref.get_transaction_count(signer.address()).await?;
            println!("Using account nonce: {}", account_nonce);
            
            // Set the nonce on the transaction builder
            tx_builder = tx_builder.nonce(account_nonce);
        } else {
            println!("No provider available, using default nonce");
        }
        
        // Create and send a direct transaction
        tx_builder
            .to(contract_address)
            .input(TransactionInput::new(calldata))
            .build(signer)
            .await?
    };
    
    // Send the transaction using the provider if available
    if let Some(provider) = provider {
        // Send the transaction using the provider
        let pending_tx = provider.send_raw_transaction(&tx.encoded_2718()).await?;
        println!("Transaction sent: {:?}", pending_tx.tx_hash());
    } else {
        // Just print the transaction details if no provider is available
        println!("Transaction built but not sent (no provider available):");
        println!("  Transaction: {:?}", tx);
    }
    
    Ok(())
}
