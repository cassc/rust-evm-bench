// SPDX-License-Identifier: MIT
pragma solidity 0.8.0;

contract Sample{
    uint8 v1  = 128;
    function add(uint8 n) external view returns (uint8){
        require(n < 128, "n is too large");
        return v1 + n;
    }
}
