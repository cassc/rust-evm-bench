use ethers::abi::Address;
use evm::backend::{MemoryAccount, MemoryBackend, MemoryVicinity};
use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};
use evm::{Capture, Config, Handler};
use eyre::{ContextCompat, Result};
use microbench::{self, Options};
use primitive_types::{H160, U256};
use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::{Duration, Instant};

fn printline() {
    println!("{}", "=".repeat(80));
}

fn main() -> Result<()> {
    // Default sender address
    let owner = H160::from_str("0xf000000000000000000000000000000000000000")?;
    // Deployed Contract address
    let mut contract_address: Address = Address::default();

    // Create EVM instance
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
        H160::from_str("0x1000000000000000000000000000000000000000")?,
        MemoryAccount {
            nonce: U256::one(),
            balance: U256::from(1000000000000u64),
            storage: BTreeMap::new(),
            code: hex::decode("6080604052348015600f57600080fd5b506004361060285760003560e01c80630f14a40614602d575b600080fd5b605660048036036020811015604157600080fd5b8101908080359060200190929190505050606c565b6040518082815260200191505060405180910390f35b6000806000905060005b83811015608f5760018201915080806001019150506076565b508091505091905056fea26469706673582212202bc9ec597249a9700278fe4ce78da83273cb236e76d4d6797b441454784f901d64736f6c63430007040033")?,
        },
    );
    state.insert(
        owner,
        MemoryAccount {
            nonce: U256::one(),
            balance: U256::from(10000000000000000u64),
            storage: BTreeMap::new(),
            code: Vec::new(),
        },
    );

    // Start EVM
    let backend = MemoryBackend::new(&vicinity, state);
    let metadata = StackSubstateMetadata::new(u64::MAX, &config);
    let state = MemoryStackState::new(metadata, &backend);
    let precompiles = BTreeMap::new();
    let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

    printline();
    let contract = hex::decode("608060405234801561001057600080fd5b50610402806100206000396000f300608060405260043610610057576000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff16806318160ddd1461005c57806370a0823114610087578063a9059cbb146100de575b600080fd5b34801561006857600080fd5b50610071610143565b6040518082815260200191505060405180910390f35b34801561009357600080fd5b506100c8600480360381019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919050505061014d565b6040518082815260200191505060405180910390f35b3480156100ea57600080fd5b50610129600480360381019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050610195565b604051808215151515815260200191505060405180910390f35b6000600154905090565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050919050565b60008073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16141515156101d257600080fd5b6000803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054821115151561021f57600080fd5b610273600183016000803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020546103ba90919063ffffffff16565b6000803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550610309600183016000808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020546103ba90919063ffffffff16565b6000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508273ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef846040518082815260200191505060405180910390a36001905092915050565b600081830190508281101515156103cd57fe5b809050929150505600a165627a7a72305820c80f9a59303c6c6d3a43ba89c4628606b06e441ee007c7516df3a6e356f3b2f20029")?;
    let method = hex::decode("a9059cbb00000000000000000000000000000000000000000000000000000000deadbeef0000000000000000000000000000000000000000000000000000000000000000")?;

    println!("EXECUTE contract deploy");
    let reason = executor.create(
        owner,
        evm::CreateScheme::Legacy { caller: owner },
        U256::default(),
        contract.clone(),
        None,
    );
    println!("RETURNS {reason:?} ");

    if let Capture::Exit((_reason, address, _return_data)) = reason {
        println!("Contract deployed to adderss {address:?} ");
        contract_address = address.context("Missing contract address, deployment failed")?;
    }

    // Microbenchmark
    let bench_options = Options::default().time(Duration::new(30, 0));
    microbench::bench(&bench_options, "execute_contract_method", || {
        let _reason = executor.transact_call(
            owner,
            contract_address,
            U256::zero(),
            method.clone(),
            u64::MAX,
            Vec::new(),
        );
        // println!("RETURNS {reason:?} ");
    });

    // Manuall execution. Compare this with microbenchmarck results to see if it's consistent
    let start = Instant::now();
    let num_execs = 100_000;
    (0..num_execs).for_each(|_| {
        let _reason = executor.transact_call(
            owner,
            contract_address,
            U256::zero(),
            method.clone(),
            u64::MAX,
            Vec::new(),
        );
        // println!("RETURNS {reason:?} ");
    });
    let duration = start.elapsed();
    println!(
        "{} runs, total {}ms, average: {}ms",
        num_execs,
        duration.as_millis(),
        duration.as_millis() as f32 / num_execs as f32,
    );

    Ok(())
}
