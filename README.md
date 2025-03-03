# World CTF

A minimal repository for testing gas consumption with two transaction types:
1. Direct transactions to a smart contract
2. PBH (Priority Blockspace for Humans) transactions

## Components

- `contracts/GasConsumer.sol`: A simple smart contract that consumes a variable amount of gas
- `src/main.rs`: A Rust program that sends both direct and PBH transactions

## Usage

```bash
cargo run -- --address <contract_address> --iterations <number> --private-key <your_private_key> --world-id <your_world_id>
```
