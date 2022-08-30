# Benchmark Rust EVM and REVM



To run the benchmark, 

``` bash
cargo run --release
```

For each EVM, 
* we'll create an external account and a contract account holding the contract `sample.sol`
* micro bench the `add(1)` method for 5 seconds
* micro bench the `add(0xc7)` method for 5 seconds


Sample results on Thinkpad X13 Gen1:

``` text
execute_contract_method_success_from_rust_evm (5.0s) ...       9_431.962 ns/iter (0.999 R²)
execute_contract_method_reverted_from_rust_evm (5.0s) ...       7_720.943 ns/iter (1.000 R²)
execute_contract_method_success_from_revm (5.0s) ...       8_617.185 ns/iter (1.000 R²)
execute_contract_method_reverted_from_revm (5.0s) ...       8_594.083 ns/iter (0.999 R²)
```

After updating revm branch on XPS (i7-10750H CPU @ 2.60GHz):
```text
execute_contract_method_success_from_rust_evm (1.0s) ...       8_549.149 ns/iter (0.999 R²)
execute_contract_method_reverted_from_rust_evm (1.0s) ...       7_504.395 ns/iter (1.000 R²)
execute_contract_method_success_from_revm (1.0s) ...       2_414.513 ns/iter (1.000 R²)
execute_contract_method_reverted_from_revm (1.0s) ...       2_194.849 ns/iter (1.000 R²)
```