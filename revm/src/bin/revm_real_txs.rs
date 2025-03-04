use eyre::Result;
use microbench::{self, Options};
use revm::{
    context::TxEnv,
    database::InMemoryDB,
    primitives::{keccak256, TxKind},
    state::{AccountInfo, Bytecode},
    Context, ExecuteEvm, MainBuilder, MainContext,
};
use revm_statetest_types::TestSuite;
use std::{fs::File, io::BufReader, time::Duration};

/// Contract address
const TEST_DURATION: Duration = Duration::from_millis(5000);

fn bench_revm(input_json: &str) -> Result<()> {
    let suite: TestSuite = serde_json::from_reader(BufReader::new(File::open(input_json)?))?;

    let mut db = InMemoryDB::default();

    let test = suite.0;
    let test = test.first_key_value().unwrap().1;

    test.pre.iter().for_each(|(address, account)| {
        let code = match account.code.len() > 2 {
            true => Some(Bytecode::new_raw(account.code.clone())),
            false => None,
        };

        let code_hash = keccak256(&account.code);
        let account_info = AccountInfo {
            nonce: account.nonce,
            balance: account.balance,
            code,
            code_hash,
        };
        db.insert_account_info(*address, account_info);

        account.storage.iter().for_each(|(key, val)| {
            db.insert_account_storage(*address, *key, *val)
                .expect("Insert storage failed")
        });
    });

    let caller = test.transaction.sender.expect("Missing sender address");
    let to = test.transaction.to.expect("Missing to address");
    let data = test.transaction.data[0].clone();
    let value = test.transaction.value[0];
    let gas_price = test
        .transaction
        .gas_price
        .map(|v| v.saturating_to::<u128>())
        .unwrap_or_default();

    let tx = TxEnv {
        caller,
        data,
        kind: TxKind::Call(to),
        value,
        gas_price,
        ..TxEnv::default()
    };

    let mut context = Context::mainnet().with_db(db).with_tx(tx.clone());
    context.block.basefee = test
        .env
        .current_base_fee
        .unwrap_or_default()
        .saturating_to::<u64>();
    context.block.gas_limit = test.env.current_gas_limit.saturating_to::<u64>();

    let mut evm = context.clone().build_mainnet();

    let bench_options = Options::default().time(TEST_DURATION);

    microbench::bench(&bench_options, &format!("benchmark {}", input_json), || {
        let r = evm.transact_previous().unwrap();
        let output = r.result.output().unwrap_or_default();
        assert!(
            r.result.is_success(),
            "REVM Method call should succeed: {:#?} \nOutput: {}",
            r.result,
            String::from_utf8_lossy(output),
        );
    });

    Ok(())
}

fn main() -> Result<()> {
    bench_revm("erc20.bench.input.json")?;
    bench_revm("uniswap.bench.input.json")
}
