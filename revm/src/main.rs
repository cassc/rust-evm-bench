use eyre::Result;
use microbench::{self, Options};
use revm::{
    context::TxEnv,
    database::InMemoryDB,
    primitives::{hex::FromHex, Address, Bytes, TxKind, U256},
    state::{AccountInfo, Bytecode},
    Context, ExecuteEvm, MainBuilder, MainContext,
};
use std::time::{Duration, Instant};

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

fn bench_revm() -> Result<()> {
    let from = Address::from_hex(OWNER_ADDR)?;
    let to = Address::from_hex(CONTRACT_ADDR)?;

    let bytecode = Bytecode::new_raw(Bytes::from(hex::decode(CONTRACT_BIN).unwrap()));

    let mut db = InMemoryDB::default();

    // Add owner account
    let account = AccountInfo {
        balance: U256::MAX,
        ..AccountInfo::default()
    };

    db.insert_account_info(from, account);

    // Add contract account
    let account = AccountInfo::from_bytecode(bytecode);
    db.insert_account_info(to, account);

    let tx = TxEnv {
        caller: from,
        kind: TxKind::Call(to),
        data: hex::decode(METHOD_SUCCESS_BIN)?.into(),
        ..TxEnv::default()
    };

    let context = Context::mainnet().with_db(db).with_tx(tx.clone());

    let mut evm = context.clone().build_mainnet();

    let bench_options = Options::default().time(TEST_DURATION);

    microbench::bench(
        &bench_options,
        "execute_contract_method_success_from_revm",
        || {
            let r = evm.transact_previous().unwrap();
            assert!(
                r.result.is_success(),
                "REVM Method call should succeed: {:#?}",
                r.result
            );
        },
    );

    let mut tx = tx.clone();
    tx.data = hex::decode(METHOD_REVERT_BIN)?.into();
    let context = context.clone().with_tx(tx);
    let mut evm = context.build_mainnet();

    microbench::bench(
        &bench_options,
        "execute_contract_method_reverted_from_revm",
        || {
            let r = evm.transact_previous().unwrap();
            assert!(
                !r.result.is_success(),
                "REVM Method call should revert, r: {:#?}",
                r.result
            );
        },
    );

    Ok(())
}

fn main() -> Result<()> {
    bench_revm()
}
