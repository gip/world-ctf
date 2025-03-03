use alloy_sol_types::sol;

sol! {
    interface IPBHEntryPoint {
        function pbhMulticall(
            tuple(address, bytes, bool)[] memory calls,
            tuple(uint256, uint256, tuple(uint8, uint8, uint8, uint16), tuple(uint256[8],)) memory payload
        ) external;
    }
}
