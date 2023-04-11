use bytes::Bytes;
use revm::db::CacheDB;
use revm::{
    db::in_memory_db::{EmptyDB, InMemoryDB},
    interpreter::{
        analysis::to_analysed, CallInputs, Contract, Gas, InstructionResult,
        Interpreter,
    },
    primitives::{Bytecode, LatestSpec, TransactTo},
    EVMData, Inspector, EVM,
};
use revm_primitives::ExecutionResult;
struct Inspect {}

impl Inspector<InMemoryDB> for Inspect {
    fn step(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, InMemoryDB>,
        _is_static: bool,
    ) -> InstructionResult {
        unsafe {
            //println!("instr pointer on each step: {:#?}", *(_interp.instruction_pointer));
        }
        //println!("data on each step: {:#?}", _data.env.tx.data.get(0));

        InstructionResult::Continue
    }

    fn call(
        &mut self,
        _data: &mut EVMData<'_, InMemoryDB>,
        _inputs: &mut CallInputs,
        _is_static: bool,
    ) -> (InstructionResult, Gas, Bytes) {
        /*println!("call inputs: {:#?}", _inputs.input);
        println!("call contract: {:#?}", _inputs.contract);
        println!("call transfer: {:#?}", _inputs.transfer);
        println!("call gas limit: {:#?}", _inputs.gas_limit);
        println!("call context: {:#?}", _inputs.context);
        println!("call _is_static input: {:#?}", _inputs.is_static);

        println!("call _is_static: {:#?}", _is_static);*/

        (InstructionResult::Continue, Gas::new(0), Bytes::new())
    }
}
use revm_primitives::create_address;
pub fn deploy_contract(hex: String, nonce: &mut u64, db: CacheDB<EmptyDB>) -> (ExecutionResult, String, Option<CacheDB<EmptyDB>>) {
    let contract_data: Bytes = hex::decode(hex).unwrap().into();
    let mut evm: EVM<InMemoryDB> = revm::new();
    evm.env.tx.caller = "0x1000000000000000000000000000000000000000"
        .parse()
        .unwrap();

    evm.env.tx.transact_to = TransactTo::create();

    evm.env.tx.data = contract_data.clone();

    evm.env.cfg.perf_all_precompiles_have_balance = true;
    evm.database(db);

    evm.env.tx.nonce = Some(*nonce);
    let result = evm.inspect_commit::<Inspect>(Inspect {}).unwrap();
    let contract_address = create_address(evm.env.tx.caller, *nonce);
    *nonce += 1;

    return (result, format!("0x{:x}", contract_address), evm.db);
}

pub fn call_contract(
    contract_address: String,
    data: String,
    db: CacheDB<EmptyDB>, 
    nonce: &mut u64
) -> ExecutionResult {
    let mut evm: EVM<InMemoryDB> = revm::new();
    evm.env.tx.caller = "0x1000000000000000000000000000000000000000"
        .parse()
        .unwrap();
    evm.env.tx.transact_to = TransactTo::Call(contract_address.parse().unwrap());
    evm.env.tx.data = hex::decode(data).unwrap().into();
    evm.env.tx.nonce = Some(*nonce);
    evm.env.cfg.perf_all_precompiles_have_balance = true;
    evm.database(db);
    let result = evm.inspect_commit::<Inspect>(Inspect {}).unwrap();
    *nonce += 1;
    result
}
