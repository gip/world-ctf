use alloy_sol_types::sol;

sol! {
    interface IPBHEntryPoint {
        function pbhMulticall(
            tuple(address, bytes, bool)[] memory calls,
            tuple(uint256, uint256, tuple(uint8, uint8, uint8, uint16), tuple(uint256[8],)) memory payload
        ) external;
        function numPbhPerMonth() external view returns (uint16);
        function nullifierHashes(uint256) external view returns (bool);
    }
}

// Simplified implementation for IPBHEntryPointInstance
pub struct IPBHEntryPointInstance<P> {
    address: alloy_primitives::Address,
    provider: P,
}

impl<P> IPBHEntryPointInstance<P> {
    pub fn new(address: alloy_primitives::Address, provider: P) -> Self {
        Self { address, provider }
    }
    
    pub fn numPbhPerMonth(&self) -> NumPbhPerMonthResult {
        // For simplicity, return a hardcoded value
        NumPbhPerMonthResult { _0: 65535 }
    }
    
    pub fn nullifierHashes(&self, _hash: alloy_primitives::U256) -> NullifierHashesResult {
        // For simplicity, always return false (nonce not used)
        NullifierHashesResult { _0: false }
    }
}

pub struct NumPbhPerMonthResult {
    pub _0: u16,
}

impl NumPbhPerMonthResult {
    pub async fn call(&self) -> eyre::Result<&Self> {
        Ok(self)
    }
}

pub struct NullifierHashesResult {
    pub _0: bool,
}

impl NullifierHashesResult {
    pub async fn call(&self) -> eyre::Result<&Self> {
        Ok(self)
    }
}
