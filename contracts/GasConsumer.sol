// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract GasConsumer {
    // A simple function that consumes gas based on the number of iterations
    function consumeGas(address _address, uint256 _iterations) public {
        // Store the address in memory to make the function do something with the parameter
        address recipient = _address;
        
        // Loop for the specified number of iterations to consume gas
        for (uint256 i = 0; i < _iterations; i++) {
            // Perform some operation to consume gas
            // This is a simple operation that will use gas for each iteration
            recipient = address(uint160(uint256(keccak256(abi.encodePacked(recipient, i)))));
        }
        
        // The function doesn't need to return anything
        // The gas consumption is the main purpose
    }
}
