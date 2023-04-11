extern crate wain_syntax_binary;
mod revm_run;
use ethabi::{encode, Token};
use etk_asm::asm::Assembler;
use etk_asm::ops::AbstractOp;
use etk_asm::ops::Imm;
use etk_asm::ops::Op;
use primitive_types::U256;
use rand::Rng;
use revm::db::CacheDB;
use revm::db::EmptyDB;
use revm::InMemoryDB;
use revm_primitives::ExecutionResult;
use std::collections::HashMap;
use std::fs;
use wain_ast::FuncKind;
use wain_ast::*;
use wain_exec::trap::Result;
use wain_syntax_binary::parse;

#[derive(Debug, Default)]
pub struct Context {
    labels: Vec<String>,
}
#[derive(Debug)]
pub enum Value {
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
}

pub struct Runtime<'module, 'source> {
    pub module: &'module Module<'source>,
    pub functions: HashMap<String, String>,
    db: CacheDB<EmptyDB>,
    nonce: u64,
}
macro_rules! to_big_endian {
    ($e:expr) => {{
        let u256 = U256::from(*$e as u64);
        let mut big_endian_bytes: [u8; 32] = [0; 32];
        u256.to_big_endian(&mut big_endian_bytes);
        &hex::encode(&big_endian_bytes)
    }};
}

impl<'m, 's> Runtime<'m, 's> {
    pub fn instantiate(module: &'m Module<'s>) -> Result<Self> {
        let mut runtime = Self {
            module: module,
            functions: HashMap::new(),
            db: InMemoryDB::new(EmptyDB::default()),
            nonce: 0,
        };

        fn find_func_name_by_id<'s>(id: u32, exports: &[Export<'s>]) -> Option<String> {
            for export in exports {
                match export.kind {
                    wain_ast::ExportKind::Func(idx) => {
                        if idx == id {
                            return Some(export.name.0.to_string());
                        }
                    }
                    _ => {}
                };
            }
            None
        }
        println!("funcs idx {:#?}", &module.funcs);
        println!("funcs idx {:#?}", &module.exports);

        for i in 0..module.funcs.len() {
            let funcs = module.funcs.get(i).unwrap();
            let mut commands: Vec<AbstractOp> = Vec::new();

            let mut globals: Context = Context { labels: Vec::new() };

            let length = module.types.get(funcs.idx as usize).unwrap().params.len();
            commands.push(AbstractOp::Op(Op::Push2(Imm::from(
                length as u16 * 0x20 as u16,
            ))));

            commands.push(AbstractOp::Op(Op::Push1(Imm::from(0 as u8))));
            commands.push(AbstractOp::Op(Op::Push1(Imm::from(0 as u8))));

            commands.push(AbstractOp::Op(Op::CallDataCopy));

            match &funcs.kind {
                FuncKind::Import(s) => {}
                FuncKind::Body { locals, expr } => {
                    commands.append(instructions_handler(expr, &mut globals).as_mut());
                }
            }
            commands.push(AbstractOp::Op(Op::Push1(Imm::from(0 as u8))));
            commands.push(AbstractOp::Op(Op::MStore));
            commands.push(AbstractOp::Op(Op::Push1(Imm::from(32 as u8))));
            commands.push(AbstractOp::Op(Op::Push1(Imm::from(0 as u8))));
            commands.push(AbstractOp::Op(Op::Return));
            let mut asm = Assembler::new();

            asm.push_all(commands).unwrap();
            let mut output = asm.take();
            asm.finish().unwrap();
            let mut deployment: Vec<AbstractOp> = Vec::new();
            deployment.push(AbstractOp::Op(Op::Push2(Imm::from(output.len() as u16))));
            deployment.push(AbstractOp::Op(Op::Push1(Imm::from(15 as u8))));
            deployment.push(AbstractOp::Op(Op::Push1(Imm::from(0 as u8))));
            deployment.push(AbstractOp::Op(Op::CodeCopy));
            deployment.push(AbstractOp::Op(Op::Push2(Imm::from(output.len() as u16))));
            deployment.push(AbstractOp::Op(Op::Push1(Imm::from(0 as u8))));
            deployment.push(AbstractOp::Op(Op::Return));
            deployment.push(AbstractOp::Op(Op::Stop));
            let mut asm2 = Assembler::new();
            asm2.push_all(deployment).unwrap();
            let mut output2 = asm2.take();
            asm2.finish().unwrap();
            assert!(15 == output2.len());
            output2.append(&mut output);
            let address = revm_run::deploy_contract(
                hex::encode(output2.clone()),
                &mut runtime.nonce,
                runtime.db,
            );
            runtime.functions.insert(
                find_func_name_by_id(i as u32, &module.exports).unwrap(),
                address.1,
            );
            runtime.db = address.2.unwrap();
        }

        Ok(runtime)
    }
    pub fn invoke(&mut self, name: &String, args: &[Value]) -> Option<ExecutionResult> {
        let mut arguments = String::new();
        for args in args {
            match args {
                Value::U32(e) => {
                    arguments += to_big_endian!(e);
                }
                Value::U64(e) => {
                    arguments += to_big_endian!(e);
                }
                Value::I32(e) => {
                    arguments += to_big_endian!(e);
                }
                Value::I64(e) => {
                    arguments += to_big_endian!(e);
                }
            };
        }
        return Some(revm_run::call_contract(
            self.functions.get(name).unwrap().clone(),
            arguments.to_string(),
            self.db.clone(),
            &mut self.nonce,
        ));
    }
}

const BYTES8: u64 = 0xFFFFFFFFFFFFFFFF;
const BYTES4: u32 = 0xFFFFFFFF;
pub fn instructions_handler(body: &Vec<Instruction>, context: &mut Context) -> Vec<AbstractOp> {
    let mut commands: Vec<AbstractOp> = Vec::new();

    for instr in body {
        match &instr.kind {
            InsnKind::Block { ty, body } => {
                let mut rng = rand::thread_rng();
                let id: u32 = rng.gen();
                context.labels.push(id.to_string());
                commands.append(instructions_handler(body, context).as_mut());
                commands.push(AbstractOp::Label(id.to_string()));
                commands.push(AbstractOp::Op(Op::JumpDest));
                context.labels.pop();
            }
            InsnKind::Loop { ty, body } => {
                let mut rng = rand::thread_rng();
                let id: u32 = rng.gen();
                context.labels.push(id.to_string());
                commands.push(AbstractOp::Label(id.to_string()));
                commands.push(AbstractOp::Op(Op::JumpDest));
                commands.append(instructions_handler(body, context).as_mut());
                context.labels.pop();
            }
            InsnKind::I32Add => {
                commands.append(i32Add().as_mut());
            }
            InsnKind::I64Add => {
                commands.append(i64Add().as_mut());
            }
            InsnKind::I32Sub => {
                commands.append(i32Sub().as_mut());
            }
            InsnKind::I64Sub => {
                commands.append(i64Sub().as_mut());
            }
            InsnKind::I32Mul => {
                commands.append(i32Mul().as_mut());
            }
            InsnKind::I64Mul => {
                commands.append(i64Mul().as_mut());
            }
            InsnKind::I32And => {
                commands.append(i32And().as_mut());
            }
            InsnKind::I64And => {
                commands.append(i64And().as_mut());
            }
            InsnKind::I32Or => {
                commands.append(i32Or().as_mut());
            }
            InsnKind::I64Or => {
                commands.append(i64Or().as_mut());
            }
            InsnKind::I32Xor => {
                commands.append(i32Xor().as_mut());
            }
            InsnKind::I64Xor => {
                commands.append(i64Xor().as_mut());
            }
            InsnKind::I32Eq => {
                commands.append(eq().as_mut());
            }
            InsnKind::I32Eqz => {
                commands.append(eqz().as_mut());
            }
            InsnKind::I32Ne => {
                commands.append(ne().as_mut());
            }
            InsnKind::I64Eq => {
                commands.append(eq().as_mut());
            }
            InsnKind::I64Eqz => {
                commands.append(eqz().as_mut());
            }
            InsnKind::I64Ne => {
                commands.append(ne().as_mut());
            }
            InsnKind::I32LtS => {
                commands.append(i32Lts().as_mut());
            }
            InsnKind::I64LtS => {
                commands.append(i64Lts().as_mut());
            }
            InsnKind::I32GtS => {
                commands.append(i32Gts().as_mut());
            }
            InsnKind::I64GtS => {
                commands.append(i64Gts().as_mut());
            }
            InsnKind::I32LeU => {
                commands.append(Leu().as_mut());
            }
            InsnKind::I64LeU => {
                commands.append(Leu().as_mut());
            }
            InsnKind::I32GeU => {
                commands.append(Geu().as_mut());
            }
            InsnKind::I64GeU => {
                commands.append(Geu().as_mut());
            }
            InsnKind::I32LeS => {
                commands.append(i32Les().as_mut());
            }
            InsnKind::I64LeS => {
                commands.append(i64Les().as_mut());
            }
            InsnKind::I32GeS => {
                commands.append(i32Ges().as_mut());
            }
            InsnKind::I64GeS => {
                commands.append(i64Ges().as_mut());
            }
            InsnKind::I32DivU => {
                commands.append(i32Divu().as_mut());
            }
            InsnKind::I64DivU => {
                commands.append(i64Divu().as_mut());
            }
            InsnKind::I32DivS => {
                commands.append(i32Divs().as_mut());
            }
            InsnKind::I64DivS => {
                commands.append(i64Divs().as_mut());
            }
            InsnKind::I32RemU => {
                commands.append(i32Remu().as_mut());
            }
            InsnKind::I64RemU => {
                commands.append(i64Remu().as_mut());
            }
            InsnKind::I32RemS => {
                commands.append(i32Rems().as_mut());
            }
            InsnKind::I64RemS => {
                commands.append(i64Rems().as_mut());
            }
            InsnKind::I32GtU => {
                commands.append(i32Gtu().as_mut());
            }
            InsnKind::I64GtU => {
                commands.append(i64Gtu().as_mut());
            }
            InsnKind::I32LtU => {
                commands.append(i32Ltu().as_mut());
            }
            InsnKind::I64LtU => {
                commands.append(i64Ltu().as_mut());
            }
            InsnKind::I32ShrS => {
                commands.append(i32Shrs().as_mut());
            }
            InsnKind::I64ShrS => {
                commands.append(i64Shrs().as_mut());
            }
            InsnKind::I32Rotl => {
                commands.append(i32Rotl().as_mut());
            }
            InsnKind::I64Rotl => {
                commands.append(i64Rotl().as_mut());
            }
            InsnKind::I32Rotr => {
                commands.append(i32Rotr().as_mut());
            }
            InsnKind::I64Rotr => {
                commands.append(i64Rotr().as_mut());
            }
            InsnKind::I32Popcnt => {
                commands.append(i32Popcnt().as_mut());
            }
            InsnKind::I64Popcnt => {
                commands.append(i64Popcnt().as_mut());
            }
            InsnKind::I32Ctz => {
                commands.append(i32Ctz().as_mut());
            }
            InsnKind::I64Ctz => {
                commands.append(i64Ctz().as_mut());
            }
            InsnKind::I32Clz => {
                commands.append(i32Clz().as_mut());
            }
            InsnKind::I64Clz => {
                commands.append(i64Clz().as_mut());
            }
            InsnKind::I32ShrU => {
                commands.append(i32Shru().as_mut());
            }
            InsnKind::I64ShrU => {
                commands.append(i64Shru().as_mut());
            }
            InsnKind::I32Shl => {
                commands.append(i32Shl().as_mut());
            }
            InsnKind::I64Shl => {
                commands.append(i64Shl().as_mut());
            }
            InsnKind::Nop => {
                commands.append(Nop().as_mut());
            }
            InsnKind::Unreachable => {
                commands.append(unreachable().as_mut());
            }
            InsnKind::LocalGet(idx) => {
                commands.append(Local_get(idx).as_mut());
            }
            InsnKind::LocalSet(idx) => {
                commands.append(Local_set(idx).as_mut());
            }
            InsnKind::LocalTee(idx) => {
                commands.append(Local_tee(idx).as_mut());
            }
            InsnKind::BrIf(idx) => {
                commands.append(br_if(context, idx).as_mut());
            }
            InsnKind::Br(idx) => {
                commands.append(br(context, idx).as_mut());
            }
            InsnKind::Drop => {
                commands.append(drop().as_mut());
            }
            InsnKind::Select => {
                commands.append(select().as_mut());
            }
            InsnKind::If {
                ty,
                then_body,
                else_body,
            } => {
                commands.append(if_fn().as_mut());
            }
            InsnKind::Call(fnidx) => {
                commands.append(call(fnidx).as_mut());
            }
            InsnKind::BrTable {
                labels,
                default_label,
            } => {
                commands.append(br_table().as_mut());
            }
            InsnKind::Return => {
                commands.append(return_fn().as_mut());
            }
            InsnKind::CallIndirect(typidx) => {
                commands.append(call_indirect(typidx).as_mut());
            }
            InsnKind::GlobalGet(globalidx) => {
                commands.append(global_get(globalidx).as_mut());
            }
            InsnKind::GlobalSet(globalidx) => {
                commands.append(global_set(globalidx).as_mut());
            }
            InsnKind::I32Load8S(mem) => {
                commands.append(i32_load_8s(mem).as_mut());
            }
            InsnKind::I32Load8U(mem) => {
                commands.append(i32_load_8u(mem).as_mut());
            }
            InsnKind::I64Load8S(mem) => {
                commands.append(i64_load_8s(mem).as_mut());
            }
            InsnKind::I64Load8U(mem) => {
                commands.append(i64_load_8u(mem).as_mut());
            }
            InsnKind::I32Load16S(mem) => {
                commands.append(i32_load_16s(mem).as_mut());
            }
            InsnKind::I32Load16U(mem) => {
                commands.append(i32_load_16u(mem).as_mut());
            }
            InsnKind::I64Load16S(mem) => {
                commands.append(i64_load_16s(mem).as_mut());
            }
            InsnKind::I64Load16U(mem) => {
                commands.append(i64_load_16u(mem).as_mut());
            }
            InsnKind::I64Load32S(mem) => {
                commands.append(i64_load_32s(mem).as_mut());
            }
            InsnKind::I64Load32U(mem) => {
                commands.append(i64_load_32u(mem).as_mut());
            }
            InsnKind::I32Load(mem) => {
                commands.append(i32_load(mem).as_mut());
            }
            InsnKind::I64Load(mem) => {
                commands.append(i64_load(mem).as_mut());
            }
            InsnKind::I32Store(mem) => {
                commands.append(i32_store(mem).as_mut());
            }
            InsnKind::I64Store(mem) => {
                commands.append(i64_store(mem).as_mut());
            }
            InsnKind::I32Store8(mem) => {
                commands.append(i32_store8(mem).as_mut());
            }
            InsnKind::I32Store16(mem) => {
                commands.append(i32_store16(mem).as_mut());
            }
            InsnKind::I64Store8(mem) => {
                commands.append(i64_store8(mem).as_mut());
            }
            InsnKind::I64Store16(mem) => {
                commands.append(i64_store16(mem).as_mut());
            }
            InsnKind::I64Store32(mem) => {
                commands.append(i64_store32(mem).as_mut());
            }
            InsnKind::MemorySize => {
                commands.append(memory_size().as_mut());
            }
            InsnKind::MemoryGrow => {
                commands.append(memory_grow().as_mut());
            }
            InsnKind::I32Const(c) => {
                commands.append(i32_const_fn(c).as_mut());
            }
            InsnKind::I64Const(c) => {
                commands.append(i64_const_fn(c).as_mut());
            }
            InsnKind::I32WrapI64 => {
                commands.append(i32_wrap_i64().as_mut());
            }
            InsnKind::I64ExtendI32S => {
                commands.append(i64_extend_i32s().as_mut());
            }
            InsnKind::I64ExtendI32U => {
                commands.append(i64_extend_i32u().as_mut());
            }
            _ => {
                commands.push(AbstractOp::Op(Op::Invalid));
            }
        };
    }

    commands
}

fn if_fn() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64_wrap_i32u() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32_wrap_i64() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64_extend_i32s() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(0 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64_extend_i32u() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64_wrap_i32s() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32_const_fn(c: &i32) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64_const_fn(c: &i64) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn call(fnidx: &u32) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push4(Imm::from(*fnidx as u32))));
    result.push(AbstractOp::Op(Op::Call));

    result
}

fn memory_size() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn memory_grow() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn br_table() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn return_fn() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    result.push(AbstractOp::Op(Op::Push1(Imm::from(0 as u8))));
    result.push(AbstractOp::Op(Op::MStore));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(32 as u8))));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(0 as u8))));
    result.push(AbstractOp::Op(Op::Return));
    result
}

fn call_indirect(typidx: &u32) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn global_get(globalIdx: &u32) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn global_set(globalIdx: &u32) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32_load_8s(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32_load_8u(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32_load_16s(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32_load_16u(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64_load_8s(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64_load_8u(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}
fn i32_load(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64_load(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32_store(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32_store8(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32_store16(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64_store(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64_store8(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64_store16(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64_store32(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32Add() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    result.push(AbstractOp::Op(Op::Add));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Add() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Add));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64_load_16s(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64_load_16u(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64_load_32s(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64_load_32u(mem: &Mem) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}
fn i32Sub() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Sub));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Sub() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Sub));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i32Mul() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Mul));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Mul() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Mul));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i32And() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64And() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i32Or() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Or));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Or() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Or));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i32Xor() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Xor));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Xor() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Xor));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn eq() -> Vec<AbstractOp> {
    //TODO might be wrong
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Eq));

    result
}

fn eqz() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::IsZero));

    result
}

fn ne() -> Vec<AbstractOp> {
    //TODO might be wrong
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(0x1 as u8))));
    result.push(AbstractOp::Op(Op::Xor));

    result
}

fn extend8s() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(0xff as u8))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(0 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn extend16s() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push2(Imm::from(0xffff as u16))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(1 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i32Lts() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::SLt));

    result
}

fn i64Lts() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::SLt));

    result
}

fn i32Gts() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SLt));

    result
}

fn i64Gts() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SLt));

    result
}

fn Leu() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::Swap2));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Lt));
    result.push(AbstractOp::Op(Op::Or));

    result
}
fn Geu() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::Swap2));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Gt));
    result.push(AbstractOp::Op(Op::Or));

    result
}

fn i32Ges() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::Swap2));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SGt));
    result.push(AbstractOp::Op(Op::Or));

    result
}

fn i64Ges() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::Swap2));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SGt));
    result.push(AbstractOp::Op(Op::Or));

    result
}

fn i32Les() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::Swap2));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SLt));
    result.push(AbstractOp::Op(Op::Or));

    result
}

fn i64Les() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::Swap2));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SLt));
    result.push(AbstractOp::Op(Op::Or));

    result
}

fn i32Divu() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    result.push(AbstractOp::Op(Op::Dup1));
    result.push(AbstractOp::Op(Op::IsZero));
    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("div_by_zero"))));
    result.push(AbstractOp::Op(Op::JumpI));

    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Swap1));

    result.push(AbstractOp::Op(Op::Div));

    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("end"))));
    result.push(AbstractOp::Op(Op::Jump));

    result.push(AbstractOp::Label("div_by_zero".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));
    result.push(AbstractOp::Op(Op::Push22(Imm::from(
        b"integer divide by zero".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Revert));

    result.push(AbstractOp::Label("end".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));

    result
}

fn i64Divu() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Dup1));
    result.push(AbstractOp::Op(Op::IsZero));
    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("div_by_zero"))));
    result.push(AbstractOp::Op(Op::JumpI));

    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Swap1));

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Div));

    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("end"))));
    result.push(AbstractOp::Op(Op::Jump));

    result.push(AbstractOp::Label("div_by_zero".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));
    result.push(AbstractOp::Op(Op::Push22(Imm::from(
        b"integer divide by zero".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Revert));

    result.push(AbstractOp::Label("end".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));

    result
}

fn i32Divs() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Push32(Imm::from(
        b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\xff\xff\xff\xff\x80\0\0\0".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::Dup2));

    result.push(AbstractOp::Op(Op::Push32(Imm::from(
        b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\xff\xff\xff\xff\xff\xff\xff\xff".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Push2(Imm::with_label(
        "integer_overflow",
    ))));
    result.push(AbstractOp::Op(Op::JumpI));

    result.push(AbstractOp::Op(Op::Dup1));
    result.push(AbstractOp::Op(Op::IsZero));
    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("div_by_zero"))));
    result.push(AbstractOp::Op(Op::JumpI));

    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));

    result.push(AbstractOp::Op(Op::Swap1));

    result.push(AbstractOp::Op(Op::SDiv));

    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("end"))));
    result.push(AbstractOp::Op(Op::Jump));

    result.push(AbstractOp::Label("div_by_zero".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));
    result.push(AbstractOp::Op(Op::Push22(Imm::from(
        b"integer divide by zero".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Revert));

    result.push(AbstractOp::Label("integer_overflow".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));

    result.push(AbstractOp::Op(Op::Push16(Imm::from(
        b"integer overflow".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Revert));

    result.push(AbstractOp::Label("end".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));

    result
}

fn i64Divs() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Push32(Imm::from(
        b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x80\0\0\0\0\0\0\0".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::Dup2));

    result.push(AbstractOp::Op(Op::Push32(Imm::from(
        b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\xff\xff\xff\xff\xff\xff\xff\xff".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Push2(Imm::with_label(
        "integer_overflow",
    ))));
    result.push(AbstractOp::Op(Op::JumpI));

    result.push(AbstractOp::Op(Op::Dup1));
    result.push(AbstractOp::Op(Op::IsZero));
    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("div_by_zero"))));
    result.push(AbstractOp::Op(Op::JumpI));

    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SDiv));

    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("end"))));
    result.push(AbstractOp::Op(Op::Jump));

    result.push(AbstractOp::Label("div_by_zero".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));
    result.push(AbstractOp::Op(Op::Push22(Imm::from(
        b"integer divide by zero".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Revert));

    result.push(AbstractOp::Label("integer_overflow".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));

    result.push(AbstractOp::Op(Op::Push16(Imm::from(
        b"integer overflow".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Revert));

    result.push(AbstractOp::Label("end".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));
    result
}

fn i32Remu() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    result.push(AbstractOp::Op(Op::Dup1));
    result.push(AbstractOp::Op(Op::IsZero));
    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("div_by_zero"))));
    result.push(AbstractOp::Op(Op::JumpI));

    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Swap1));

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Mod));

    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("end"))));
    result.push(AbstractOp::Op(Op::Jump));

    result.push(AbstractOp::Label("div_by_zero".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));
    result.push(AbstractOp::Op(Op::Push22(Imm::from(
        b"integer divide by zero".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Revert));

    result.push(AbstractOp::Label("end".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));

    result
}

fn i64Remu() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    result.push(AbstractOp::Op(Op::Dup1));
    result.push(AbstractOp::Op(Op::IsZero));
    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("div_by_zero"))));
    result.push(AbstractOp::Op(Op::JumpI));

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Mod));

    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("end"))));
    result.push(AbstractOp::Op(Op::Jump));

    result.push(AbstractOp::Label("div_by_zero".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));
    result.push(AbstractOp::Op(Op::Push22(Imm::from(
        b"integer divide by zero".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Revert));

    result.push(AbstractOp::Label("end".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));

    result
}

fn i32Rems() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    result.push(AbstractOp::Op(Op::Dup1));
    result.push(AbstractOp::Op(Op::IsZero));
    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("div_by_zero"))));
    result.push(AbstractOp::Op(Op::JumpI));

    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SMod));
    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("end"))));
    result.push(AbstractOp::Op(Op::Jump));

    result.push(AbstractOp::Label("div_by_zero".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));
    result.push(AbstractOp::Op(Op::Push22(Imm::from(
        b"integer divide by zero".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Revert));

    result.push(AbstractOp::Label("end".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));

    result
}

fn i64Rems() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Dup1));
    result.push(AbstractOp::Op(Op::IsZero));
    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("div_by_zero"))));
    result.push(AbstractOp::Op(Op::JumpI));

    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SMod));

    result.push(AbstractOp::Op(Op::Push2(Imm::with_label("end"))));
    result.push(AbstractOp::Op(Op::Jump));

    result.push(AbstractOp::Label("div_by_zero".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));
    result.push(AbstractOp::Op(Op::Push22(Imm::from(
        b"integer divide by zero".clone(),
    ))));
    result.push(AbstractOp::Op(Op::Revert));

    result.push(AbstractOp::Label("end".to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));

    result
}

fn i32Gtu() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Gt));

    result
}

fn i64Gtu() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Gt));

    result
}

fn i32Ltu() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Lt));

    result
}

fn i64Ltu() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Lt));

    result
}

fn i32Shrs() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(32 as u8))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Mod));
    result.push(AbstractOp::Op(Op::Swap1));

    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Sar));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Shrs() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(64 as u8))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Mod));
    result.push(AbstractOp::Op(Op::Swap1));

    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Sar));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i32Rotl() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64Rotl() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32Rotr() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64Rotr() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32Popcnt() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64Popcnt() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32Ctz() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64Ctz() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32Clz() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i64Clz() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn i32Shru() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Swap1));

    result.push(AbstractOp::Op(Op::Push1(Imm::from(32 as u8))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Mod));
    result.push(AbstractOp::Op(Op::Shr));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Shru() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Swap1));

    result.push(AbstractOp::Op(Op::Push1(Imm::from(64 as u8))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Mod));
    result.push(AbstractOp::Op(Op::Shr));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn Shr() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Shr));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i32Shl() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    result.push(AbstractOp::Op(Op::Push1(Imm::from(31 as u8))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Shl));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Shl() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(63 as u8))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Shl));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(BYTES8))));
    result.push(AbstractOp::Op(Op::And));
    result
}

fn Nop() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::JumpDest));

    result
}

fn unreachable() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}

fn Local_get(idx: &u32) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push4(Imm::from(idx * 0x20 as u32))));
    result.push(AbstractOp::Op(Op::MLoad));

    result
}

fn Local_set(idx: &u32) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push4(Imm::from(idx * 0x20 as u32))));
    result.push(AbstractOp::Op(Op::MStore));

    result
}

fn Local_tee(idx: &u32) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Dup1));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(idx * 0x20 as u32))));
    result.push(AbstractOp::Op(Op::MStore));

    result
}

fn br_if(context: &Context, idx: &u32) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push2(Imm::with_label(
        context.labels.get(*idx as usize).unwrap(),
    ))));
    result.push(AbstractOp::Op(Op::JumpI));

    result
}

fn br(context: &Context, idx: &u32) -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push2(Imm::with_label(
        context.labels.get(*idx as usize).unwrap(),
    ))));
    result.push(AbstractOp::Op(Op::Jump));

    result
}

fn drop() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push4(Imm::from(BYTES4))));

    result
}

fn select() -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    let mut rng = rand::thread_rng();
    let random_nonzero: u32 = rng.gen();
    result.push(AbstractOp::Op(Op::Push2(Imm::with_label(
        random_nonzero.to_string(),
    ))));
    result.push(AbstractOp::Op(Op::JumpI));
    result.push(AbstractOp::Op(Op::Pop));

    let random_exit: u32 = rng.gen();
    result.push(AbstractOp::Op(Op::Push2(Imm::with_label(
        random_exit.to_string(),
    ))));
    result.push(AbstractOp::Op(Op::Jump));

    result.push(AbstractOp::Label(random_nonzero.to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Pop));

    result.push(AbstractOp::Label(random_exit.to_string()));
    result.push(AbstractOp::Op(Op::JumpDest));

    result
}
