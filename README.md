# Benchmark Rust EVM and REVM

# Comparing REVM and Go-ethereum executor: ERC20 Transfer and UniSwap Swap

Comparing the single thread performance of REVM and Go-ethereum executor, all states are kept in memory during the test.

> You can also run the benchmark tests in docker using the prebuilt image [evm-benches](https://hub.docker.com/r/augustus/evm-benches)


``` bash
cargo run --release --locked  --bin revm_real_txs  -- -t 5000 erc20.bench.input.json
cargo run --release --locked  --bin revm_real_txs  -- -t 5000 uniswap.bench.input.json

benchmark erc20.bench.input.json (5.0s) ...       8_010.154 ns/iter (1.000 R²)
benchmark uniswap.bench.input.json (5.1s) ...     175_011.400 ns/iter (1.000 R²)

cd go-evm
go run goevm_real_txs/main.go -n 10000 ../erc20.bench.input.json
go run goevm_real_txs/main.go -n 10000 ../uniswap.bench.input.json

Running 10000 transactions for each test case
Benchmark ../erc20.bench.input.json     0.36 ms / transaction
Benchmark ../uniswap.bench.input.json   2.13 ms / transaction
```



## Simple transaction

For each EVM,
* we'll create an external account and a contract account holding the contract `sample.sol`
* micro bench the `add(1)` method for 5 seconds
* micro bench the `add(0xc7)` method for 5 seconds


To run the benchmark,

``` bash
# To run benchmark test for revm
cargo run --release  --bin revm-bench

# To run benchmark test for rust-evm
cargo run --release  --bin rust-evm-bench

# To run benchmark test for go-evm
cd go-evm
go run main/main.go
```


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

go-evm results:

``` text
Ran 10000 iterations of success test in 39.495276ms
Average time per iteration: 3.949µs
Ran 10000 iterations of revert test in 32.057048ms
Average time per iteration: 3.205µs
```
