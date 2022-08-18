# Benchmark Rust EVM



To run the benchmark, 

``` bash
cargo run --release
```

For each EVM, 
* we'll create an external account and a contract account holding the contract `sample.sol`
* call the `add(1)` method for 100_000 times
* call the `add(0xc7)` method for 100_000 times


Sample results:

``` text
execute_contract_method_success_from_revm (5.0s) ...       8_573.829 ns/iter (0.999 R²)
execute_contract_method_reverted_from_revm (5.0s) ...       8_188.534 ns/iter (0.999 R²)
execute_contract_method_success_from_rust_evm (5.0s) ...       8_419.229 ns/iter (0.999 R²)
execute_contract_method_reverted_from_rust_evm (5.0s) ...       7_759.615 ns/iter (0.995 R²)
```
