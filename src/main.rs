use alloy_primitives::{Address, Bytes, U256};
use alloy_rpc_types_eth::TransactionInput;
use alloy_signer_local::PrivateKeySigner;
use clap::Parser;
use eyre::Result;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

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
}

// Command line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Number of iterations for gas consumption
    #[clap(long)]
    iterations: u64,
    
    /// RPC provider URI
    #[clap(long, default_value = "https://worldchain-sepolia.infura.io/v3/your-api-key")]
    provider_uri: String,
    
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

// Function to create calldata for the consumeGas function
fn consume_gas_calldata(address: Address, iterations: U256) -> Bytes {
    // Function selector for consumeGas(address,uint256)
    // keccak256("consumeGas(address,uint256)")[0..4]
    let selector = [0x41, 0x4c, 0xf8, 0x5d];
    
    // Encode the parameters
    let mut calldata = selector.to_vec();
    
    // Encode address (pad to 32 bytes)
    let mut address_bytes = vec![0u8; 12];
    address_bytes.extend_from_slice(address.as_slice());
    calldata.extend_from_slice(&address_bytes);
    
    // Encode iterations
    let mut iterations_bytes = [0u8; 32];
    let iterations_vec = iterations.to_be_bytes_vec();
    for (i, b) in iterations_vec.iter().rev().enumerate() {
        iterations_bytes[32 - iterations_vec.len() + i] = *b;
    }
    calldata.extend_from_slice(&iterations_bytes);
    
    calldata.into()
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
    let calldata = consume_gas_calldata(contract_address, iterations);
    
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
    
    // Create and send the transaction
    let tx = if args.use_pbh {
        // Create a WorldID from the world_id in the config
        let world_id = WorldID::new(&config.world_id)?;
        
        // Create a multicall for the consumeGas function
        let calls = consume_gas_multicall(contract_address, args.iterations);
        
        // Create and send a PBH transaction
        GasTestTransactionBuilder::new(args.gas_fee, args.priority_gas_fee)
            .with_pbh_multicall(&world_id, args.pbh_nonce, signer.address(), calls)
            .await?
            .build(signer)
            .await?
    } else {
        // Create and send a direct transaction
        GasTestTransactionBuilder::new(args.gas_fee, args.priority_gas_fee)
            .to(contract_address)
            .input(TransactionInput::new(calldata))
            .build(signer)
            .await?
    };
    
    println!("Transaction sent: {:?}", tx);
    
    Ok(())
}
