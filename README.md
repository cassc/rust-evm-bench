# Benchmark Rust EVM

To run the benchmark, 

``` bash
cargo run --release
```

It will 
* First deploy a compiled ERC20 contract
* Get the address of the deployed contract
* Execute a transfer call method from the contract by microbenchmark for 30secs
* Call the same method for 100_000 times


Sample results:

``` text
================================================================================
EXECUTE contract deploy
RETURNS Exit((Succeed(Returned), Some(0xc15d2ba57d126e6603240e89437efd419ce329d2), []))
Contract deployed to adderss Some(0xc15d2ba57d126e6603240e89437efd419ce329d2)
execute_contract_method (30.2s) ...      20_697.744 ns/iter (0.991 R²)
100000 runs, total 2043ms, average: 0.02043ms
```
