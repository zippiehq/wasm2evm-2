use bytes::Bytes;
use revm::{
    db::BenchmarkDB,
    interpreter::{analysis::to_analysed, BytecodeLocked, Contract, DummyHost, Interpreter},
    primitives::{LatestSpec, Bytecode, TransactTo},
    EVM, 
};
use revm_primitives::create_address;

fn run_revm () {
    let contract_data : Bytes = hex::decode("610023600f6000396100236000f30061004060006000376300000000516300000020510163ffffffff1660005260206000f3").unwrap().into();
    let mut evm: EVM<BenchmarkDB> = revm::new();
    evm.env.tx.caller = "0x1000000000000000000000000000000000000000"
        .parse()
        .unwrap();

    evm.env.tx.transact_to = TransactTo::Call(
        "0x0000000000000000000000000000000000000000"
            .parse()
            .unwrap(),
    );

    evm.env.tx.data = contract_data.clone();
    let bytecode_raw = to_analysed::<LatestSpec>(Bytecode::new_raw(contract_data.clone()));

    evm.env.cfg.perf_all_precompiles_have_balance = true;
    evm.database(BenchmarkDB::new_bytecode(bytecode_raw.clone()));

    let env = evm.env.clone();
    let result = evm.transact().unwrap();

    println!("{:#?}", result.state);
    let contract_address = create_address(evm.env.tx.caller, 0);

    let contract = Contract {
        address : contract_address,
        ..Default::default()
    };

    evm.env.tx.transact_to = TransactTo::Call(
        contract_address,
    );
    evm.env.tx.data = hex::decode("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001").unwrap().into();
    evm.env.tx.nonce = Some(1);
    let result = evm.transact().unwrap();

    println!();
    println!("{:?}", result.result);
}