use evm::backend::OverlayedBackend;
use evm::standard::{Config, Etable, EtableResolver, Invoker, TransactArgs};
use evm_precompile::StandardPrecompileSet;
use eyre::Result;
use jsontests::in_memory::{InMemoryAccount, InMemoryBackend, InMemoryEnvironment};
use microbench::{self, Options};
use primitive_types::H160;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::Duration;

/// Contract address
const OWNER_ADDR: &str = "0xf000000000000000000000000000000000000000";
const CONTRACT_ADDR: &str = "0x0000000000000000000000000000000000000000";
/// sample.sol: runtime binary compiled with solc 0.8.0
const CONTRACT_BIN: &str = "608060405234801561001057600080fd5b506004361061002b5760003560e01c806302067e6a14610030575b600080fd5b61004a600480360381019061004591906100dd565b610060565b6040516100579190610175565b60405180910390f35b600060808260ff16106100a8576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161009f90610155565b60405180910390fd5b8160008054906101000a900460ff166100c191906101a1565b9050919050565b6000813590506100d781610214565b92915050565b6000602082840312156100ef57600080fd5b60006100fd848285016100c8565b91505092915050565b6000610113600e83610190565b91507f6e20697320746f6f206c617267650000000000000000000000000000000000006000830152602082019050919050565b61014f816101d8565b82525050565b6000602082019050818103600083015261016e81610106565b9050919050565b600060208201905061018a6000830184610146565b92915050565b600082825260208201905092915050565b60006101ac826101d8565b91506101b7836101d8565b92508260ff038211156101cd576101cc6101e5565b5b828201905092915050565b600060ff82169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b61021d816101d8565b811461022857600080fd5b5056fea2646970667358221220adf34242ee77b97aa044e069ebc15c434c1296636f349e726a9678ab019ab2f264736f6c63430008000033";
/// sample.sol: add(1)
const METHOD_SUCCESS_BIN: &str =
    "02067e6a0000000000000000000000000000000000000000000000000000000000000001";

/// sample.sol: add(0xc7)
const METHOD_REVERT_BIN: &str =
    "02067e6a00000000000000000000000000000000000000000000000000000000000000c7";

const TEST_DURATION: Duration = Duration::from_millis(5000);

fn bench_rust_evm() -> Result<()> {
    let owner = H160::from_str(OWNER_ADDR)?;
    let contract = H160::from_str(CONTRACT_ADDR)?;

    let config = Config::shanghai();
    let env = InMemoryEnvironment {
        block_hashes: Default::default(),
        block_number: Default::default(),
        block_coinbase: Default::default(),
        block_timestamp: Default::default(),
        block_difficulty: Default::default(),
        block_randomness: Default::default(),
        block_gas_limit: Default::default(),
        block_base_fee_per_gas: Default::default(),
        chain_id: Default::default(),
    };

    // EVM initial states
    let mut state = BTreeMap::new();
    state.insert(
        owner,
        InMemoryAccount {
            nonce: 1.into(),
            balance: 1000000000000u64.into(),
            storage: BTreeMap::new(),
            code: Default::default(),
            transient_storage: Default::default(),
        },
    );
    state.insert(
        contract,
        InMemoryAccount {
            nonce: 1u64.into(),
            balance: 2000000000000u64.into(),
            storage: BTreeMap::new(),
            code: hex::decode(CONTRACT_BIN)?,
            transient_storage: Default::default(),
        },
    );

    // Start EVM
    let backend = InMemoryBackend {
        environment: env,
        state,
    };

    let args = TransactArgs::Call {
        caller: owner,
        address: contract,
        value: Default::default(),
        data: hex::decode(METHOD_SUCCESS_BIN)?,
        gas_limit: u64::MAX.into(),
        gas_price: Default::default(),
        access_list: vec![],
    };

    let gas_etable = Etable::single(evm::standard::eval_gasometer);
    let exec_etable = Etable::runtime();
    let etable = (gas_etable, exec_etable);
    let precompiles = StandardPrecompileSet::new(&config);
    let resolver = EtableResolver::new(&config, &precompiles, &etable);
    let invoker = Invoker::new(&config, &resolver);

    // Microbenchmark
    let bench_options = Options::default().time(TEST_DURATION);

    let mut run_backend = OverlayedBackend::new(&backend, Default::default(), &config);

    microbench::bench(
        &bench_options,
        "execute_contract_method_success_from_rust_evm",
        || assert!(evm::transact(args.clone(), Some(4), &mut run_backend, &invoker).is_ok()),
    );

    let args = TransactArgs::Call {
        caller: owner,
        address: contract,
        value: Default::default(),
        data: hex::decode(METHOD_REVERT_BIN)?,
        gas_limit: u64::MAX.into(),
        gas_price: Default::default(),
        access_list: vec![],
    };

    microbench::bench(
        &bench_options,
        "execute_contract_method_reverted_from_rust_evm",
        || assert!(evm::transact(args.clone(), Some(4), &mut run_backend, &invoker).is_err()),
    );

    Ok(())
}

fn main() -> Result<()> {
    bench_rust_evm()
}
