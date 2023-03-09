use bytes::Bytes;
use revm::{
    Inspector,
    EVMData,
    db::in_memory_db::{InMemoryDB, EmptyDB},
    interpreter::{analysis::to_analysed, BytecodeLocked, Contract, Gas, Interpreter, InstructionResult, CallInputs},
    primitives::{LatestSpec, Bytecode, TransactTo},
    EVM, 
};

struct Inspect {

}

impl Inspector<InMemoryDB> for Inspect {
    
    fn step(&mut self,_interp: &mut Interpreter,_data: &mut EVMData<'_, InMemoryDB>,_is_static: bool) -> InstructionResult {
        unsafe {
            println!("instr pointer on each step: {:#?}", *(_interp.instruction_pointer));
        }
        println!("data on each step: {:#?}", _data.env.tx.data.get(0));

        InstructionResult::Continue
    }

    fn call(
        &mut self,
        _data: &mut EVMData<'_, InMemoryDB>,
        _inputs: &mut CallInputs,
        _is_static: bool,
    ) -> (InstructionResult, Gas, Bytes) {
        println!("call inputs: {:#?}", _inputs.input);
        println!("call contract: {:#?}", _inputs.contract);
        println!("call transfer: {:#?}", _inputs.transfer);
        println!("call gas limit: {:#?}", _inputs.gas_limit);
        println!("call context: {:#?}", _inputs.context);
        println!("call _is_static input: {:#?}", _inputs.is_static);

        println!("call _is_static: {:#?}", _is_static);

        (InstructionResult::Continue, Gas::new(0), Bytes::new())
    }
    
}
use revm_primitives::create_address;
fn main() {
    let contract_data : Bytes = hex::decode("610023600f6000396100236000f30061004060006000376300000000516300000020510163ffffffff1660005260206000f3").unwrap().into();
    let mut evm: EVM<InMemoryDB> = revm::new();
    evm.env.tx.caller = "0x1000000000000000000000000000000000000000"
        .parse()
        .unwrap();

    evm.env.tx.transact_to = TransactTo::create();

    evm.env.tx.data = contract_data.clone();
    let bytecode_raw = to_analysed::<LatestSpec>(Bytecode::new_raw(contract_data.clone()));

    evm.env.cfg.perf_all_precompiles_have_balance = true;
    evm.database(InMemoryDB::new(EmptyDB::default()));

    let env = evm.env.clone();
    evm.env.tx.nonce = Some(0);
    let result = evm.inspect_commit::<Inspect>(Inspect{}).unwrap();

    println!("tx {:#?}", evm.env.tx);
    println!("{:#?}", result);
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
    let result = evm.inspect_commit::<Inspect>(Inspect{}).unwrap();

    println!("tx {:#?}", evm.env.tx);
    println!("{:?}", result);

}
