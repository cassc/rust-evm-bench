use bytes::Bytes;
use evm::backend::{MemoryAccount, MemoryBackend, MemoryVicinity};
use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};
use evm::Config;
use eyre::Result;
use microbench::{self, Options};
use primitive_types::{H160, U256};
use revm::{AccountInfo, Bytecode, InMemoryDB, Return, TransactTo};
use std::collections::BTreeMap;
use std::str::FromStr;

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

fn bench_rust_evm() -> Result<()> {
    let owner = H160::from_str(OWNER_ADDR)?;
    let contract = H160::from_str(CONTRACT_ADDR)?;

    let config = Config::istanbul();
    let vicinity = MemoryVicinity {
        gas_price: U256::default(),
        origin: H160::default(),
        chain_id: U256::one(),
        block_hashes: Vec::new(),
        block_number: Default::default(),
        block_coinbase: Default::default(),
        block_timestamp: Default::default(),
        block_difficulty: Default::default(),
        block_gas_limit: Default::default(),
        block_base_fee_per_gas: U256::zero(),
    };

    // EVM initial states
    let mut state = BTreeMap::new();
    state.insert(
        owner,
        MemoryAccount {
            nonce: U256::one(),
            balance: U256::from(1000000000000u64),
            storage: BTreeMap::new(),
            code: Vec::new(),
        },
    );
    state.insert(
        contract,
        MemoryAccount {
            nonce: U256::one(),
            balance: U256::from(2000000000000u64),
            storage: BTreeMap::new(),
            code: hex::decode(CONTRACT_BIN)?,
        },
    );

    // Start EVM
    let backend = MemoryBackend::new(&vicinity, state);
    let metadata = StackSubstateMetadata::new(u64::MAX, &config);
    let state = MemoryStackState::new(metadata, &backend);
    let mut executor = StackExecutor::new_with_precompiles(state, &config, &());

    let method = hex::decode(METHOD_SUCCESS_BIN)?;

    // Microbenchmark
    let bench_options = Options::default();
    microbench::bench(
        &bench_options,
        "execute_contract_method_success_from_rust_evm",
        || {
            let (reason, _) = executor.transact_call(
                owner,
                contract,
                U256::zero(),
                method.clone(),
                u64::MAX,
                Vec::new(),
            );
            assert!(
                reason.is_succeed(),
                "RUST_EVM Method call should succeed: {:?}",
                reason
            );
        },
    );

    let method = hex::decode(METHOD_REVERT_BIN)?;
    microbench::bench(
        &bench_options,
        "execute_contract_method_reverted_from_rust_evm",
        || {
            let (reason, data) = executor.transact_call(
                owner,
                contract,
                U256::zero(),
                method.clone(),
                u64::MAX,
                Vec::new(),
            );
            assert!(
                reason.is_revert(),
                "RUST_EVM Method call should revert: {:?} {:#?}",
                reason,
                data
            );
        },
    );

    Ok(())
}

fn bench_revm() -> Result<()> {
    let from = H160::from_str(OWNER_ADDR)?;
    let to = H160::from_str(CONTRACT_ADDR)?;
    let contract_bin: Bytes = hex::decode(CONTRACT_BIN)?.into();

    let mut evm = revm::new();
    let mut db = InMemoryDB::default();

    // Add owner account
    let mut account = AccountInfo::default();
    account.balance = U256::MAX;
    db.insert_account_info(from, account);

    // Add contract account
    let account = AccountInfo::new(U256::MAX, 0u64, Bytecode::new_raw(contract_bin));
    db.insert_account_info(to, account);

    evm.database(db);
    evm.env.tx.caller = from;
    evm.env.tx.transact_to = TransactTo::Call(to);

    let bench_options = Options::default();

    evm.env.tx.data = hex::decode(METHOD_SUCCESS_BIN)?.into();
    microbench::bench(
        &bench_options,
        "execute_contract_method_success_from_revm",
        || {
            let (r,_) = evm.transact();
            assert!(
                matches!(r.exit_reason, Return::Return),
                "REVM Method call should succeed: {:#?}",
                r.exit_reason
            );
        },
    );

    evm.env.tx.data = hex::decode(METHOD_REVERT_BIN)?.into();
    microbench::bench(
        &bench_options,
        "execute_contract_method_reverted_from_revm",
        || {
            let (result,_) = evm.transact();
            assert!(
                matches!(result.exit_reason, Return::Revert),
                "REVM Method call should revert, r: {:#?}",
                result.exit_reason,
            );
        },
    );

    Ok(())
}

fn main() -> Result<()> {
    bench_rust_evm()?;
    bench_revm()
}
