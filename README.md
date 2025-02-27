# Benchmark Rust EVM and REVM



To run the benchmark,

``` bash
# To run benchmark test for revm
cargo run --release  --bin revm-bench

# To run benchmark test for rust-evm
cargo run --release  --bin rust-evm-bench
```

For each EVM,
* we'll create an external account and a contract account holding the contract `sample.sol`
* micro bench the `add(1)` method for 5 seconds
* micro bench the `add(0xc7)` method for 5 seconds


Sample results on  AMD Ryzen 5 3600 6-Core Processor with 32GB RAM DDR4 3600MHz Desktop:

rust-evm results:
``` text
execute_contract_method_success_from_rust_evm (5.0s) ...      13_246.844 ns/iter (1.000 R²)
execute_contract_method_reverted_from_rust_evm (5.0s) ...      11_580.123 ns/iter (1.000 R²)
```


revm results:

``` text
execute_contract_method_success_from_revm (5.0s) ...       5_213.270 ns/iter (1.000 R²)
execute_contract_method_reverted_from_revm (5.0s) ...       4_698.849 ns/iter (1.000 R²)
```
