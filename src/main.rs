extern crate wain_syntax_binary;

use std::fs;
use etk_asm::ops::Imm;
use wain_syntax_binary::parse;
use etk_asm::ops::AbstractOp;
use etk_asm::ops::Abstract;
use etk_asm::ops::Op;
use wain_ast::*;
use wain_ast::FuncKind;

fn main() {
    let source = fs::read("fib.wasm").unwrap();
    let mut commands: Vec<AbstractOp> = Vec::new();
    match parse(&source) {
        Ok(tree) => {
            println!("{:?}", tree.module.funcs);  

            for funcs in tree.module.funcs {
                match &funcs.kind {
                    FuncKind::Import(s) => {},
                    FuncKind::Body {locals, expr} => {
                            commands.append(instructions_handler(expr).as_mut());
                        }
                    }
                };
            }
        Err(err) => eprintln!("Error! {}", err),
    };

    println!("{:?}", commands);  

}

fn instructions_handler (body: &Vec<Instruction>)  -> Vec<AbstractOp> {
    let mut commands: Vec<AbstractOp> = Vec::new();

    for instr in body {
        match &instr.kind {
            InsnKind::Block { ty, body } => {
                commands.append(instructions_handler(body).as_mut());
            },
            InsnKind::I32Add => {
                commands.append(i32Add().as_mut());
            },
            InsnKind::I64Add => {
                commands.append(i64Add().as_mut());
            },
            InsnKind::I32Sub => {
                commands.append(i32Sub().as_mut());
            },
            InsnKind::I64Sub => {
                commands.append(i64Sub().as_mut());
            },
            InsnKind::I32Mul => {
                commands.append(i32Mul().as_mut());
            },
            InsnKind::I64Mul => {
                commands.append(i64Mul().as_mut());
            },
            InsnKind::I32And => {
                commands.append(i32And().as_mut());
            },
            InsnKind::I64And => {
                commands.append(i64And().as_mut());
            },
            InsnKind::I32Or => {
                commands.append(i32Or().as_mut());
            },
            InsnKind::I64Or => {
                commands.append(i64Or().as_mut());
            },
            InsnKind::I32Xor => {
                commands.append(i32Xor().as_mut());
            },
            InsnKind::I64Xor => {
                commands.append(i64Xor().as_mut());
            },
            InsnKind::I32Eq => {
                commands.append(eq().as_mut());
            },
            InsnKind::I32Eqz => {
                commands.append(eqz().as_mut());
            },
            InsnKind::I32Ne => {
                commands.append(ne().as_mut());
            },
            InsnKind::I32LtS => {
                commands.append(i32Lts().as_mut());
            },
            InsnKind::I64LtS => {
                commands.append(i64Lts().as_mut());
            },
            InsnKind::I32GtS => {
                commands.append(i32Gts().as_mut());
            },
            InsnKind::I64GtS => {
                commands.append(i64Gts().as_mut());
            },
            InsnKind::I32LeU => {
                commands.append(Leu().as_mut());
            },
            InsnKind::I32GeU => {
                commands.append(Geu().as_mut());
            },
            InsnKind::I32LeS => {
                commands.append(i32Les().as_mut());
            },
            InsnKind::I64LeS => {
                commands.append(i64Les().as_mut());
            },
            InsnKind::I32GeS => {
                commands.append(i32Ges().as_mut());
            },
            InsnKind::I64GeS => {
                commands.append(i64Ges().as_mut());
            },
            InsnKind::I32DivU => {
                commands.append(Divu().as_mut());
            },
            InsnKind::I32DivS => {
                commands.append(i32Divs().as_mut());
            },
            InsnKind::I64DivS => {
                commands.append(i64Divs().as_mut());
            },
            InsnKind::I32RemU => {
                commands.append(Remu().as_mut());
            },
            InsnKind::I32RemS => {
                commands.append(i32Rems().as_mut());
            },
            InsnKind::I64RemS => {
                commands.append(i64Rems().as_mut());
            },
            InsnKind::I32GtU => {
                commands.append(Gtu().as_mut());
            },
            InsnKind::I32LtU => {
                commands.append(Ltu().as_mut());
            },
            InsnKind::I32ShrS => {
                commands.append(i32Shrs().as_mut());
            },
            InsnKind::I64ShrS => {
                commands.append(i64Shrs().as_mut());
            },
            InsnKind::I32Rotl => {
                commands.append(Rotl().as_mut());
            },
            InsnKind::I32Rotr => {
                commands.append(Rotr().as_mut());
            },
            InsnKind::I32Popcnt => {
                commands.append(Popcnt().as_mut());
            },
            InsnKind::I32Ctz => {
                commands.append(Ctz().as_mut());
            },
            InsnKind::I32Clz => {
                commands.append(Clz().as_mut());
            },
            InsnKind::I32ShrU => {
                commands.append(i32Shru().as_mut());
            },
            InsnKind::I64ShrU => {
                commands.append(i64Shru().as_mut());
            },
            InsnKind::I32ShrS => {
                commands.append(i32Shrs().as_mut());
            },
            InsnKind::I64ShrS => {
                commands.append(i64Shrs().as_mut());
            },
            InsnKind::I32Shl => {
                commands.append(i32Shl().as_mut());
            },
            InsnKind::I64Shl => {
                commands.append(i64Shl().as_mut());
            },
            InsnKind::Nop => {
                commands.append(Nop().as_mut());
            },
            InsnKind::Unreachable => {
                commands.append(Nop().as_mut());
            },
            _ => {},
        };
    }

    commands
}

fn i32Add ()  -> Vec<AbstractOp>{
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Add));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(0xFFFFFFFF as u32))));
    result.push(AbstractOp::Op(Op::And));

    result
}
fn i64Add ()  -> Vec<AbstractOp>{
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Add));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(0xFFFFFFFFFFFFFFFF as u64))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i32Sub () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Sub));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(0xFFFFFFFF as u32))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Sub () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Sub));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(0xFFFFFFFFFFFFFFFF as u64))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::And));

    result
}



fn i32Mul () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Mul));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(0xFFFFFFFF as u32))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Mul () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Mul));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(0xFFFFFFFFFFFFFFFF as u64))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i32And () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(0xFFFFFFFF as u32))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64And () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(0xFFFFFFFFFFFFFFFF as u64))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i32Or () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Or));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(0xFFFFFFFF as u32))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Or () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Or));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(0xFFFFFFFFFFFFFFFF as u64))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i32Xor () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Xor));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(0xFFFFFFFF as u32))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Xor () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Xor));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(0xFFFFFFFFFFFFFFFF as u64))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn eq () -> Vec<AbstractOp> { //TODO might be wrong
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Eq));

    result
}

fn eqz () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::IsZero));

    result
}

fn ne () -> Vec<AbstractOp> { //TODO might be wrong 
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(0x1 as u8))));
    result.push(AbstractOp::Op(Op::Xor));

    result
}

fn extend8s () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(0xff as u8))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(0 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(0xFFFFFFFF as u32))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn extend16s () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push2(Imm::from(0xffff as u16))));
    result.push(AbstractOp::Op(Op::And));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(1 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(0xFFFFFFFF as u32))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i32Lts () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::SLt));

    result
}

fn i64Lts () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::SLt));

    result
}

fn i32Gts () -> Vec<AbstractOp> {
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

fn i64Gts () -> Vec<AbstractOp> {
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

fn Leu () -> Vec<AbstractOp> {
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

fn Geu () -> Vec<AbstractOp> {
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

fn i32Ges () -> Vec<AbstractOp> {
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

fn i64Ges () -> Vec<AbstractOp> {
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

fn i32Les () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::Swap2));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SLt));
    result.push(AbstractOp::Op(Op::Or));

    result
}

fn i64Les () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Dup2));
    result.push(AbstractOp::Op(Op::Eq));
    result.push(AbstractOp::Op(Op::Swap2));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SLt));
    result.push(AbstractOp::Op(Op::Or));

    result
}

fn Divu () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Div));

    result
}

fn i32Divs () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    
    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SDiv));

    result
}

fn i64Divs () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    
    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SDiv));

    result
}

fn Remu () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Mod));

    result
}

fn i32Rems () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SMod));

    result
}

fn i64Rems () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::SMod));
    
    result
}

fn Gtu () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Gt));
    
    result
}

fn Ltu () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();
    
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Lt));
    
    result
}

fn i32Shrs () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(20 as u8))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Mod));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(3 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Sar));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(0xFFFFFFFF as u32))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Shrs () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(40 as u8))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Mod));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Push1(Imm::from(7 as u8))));
    result.push(AbstractOp::Op(Op::SignExtend));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Sar));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(0xFFFFFFFFFFFFFFFF as u64))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn Rotl () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    //TODO PrepareCall
    result
}

fn Rotr () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Swap1));
    //TODO PrepareCall
    result
}

fn Popcnt () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    //TODO PrepareCall
    result
}

fn Ctz () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    //TODO PrepareCall
    result
}

fn Clz () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    //TODO PrepareCall
    result
}

fn i32Shru () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(20 as u8))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Mod));
    result.push(AbstractOp::Op(Op::Shr));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(0xFFFFFFFF as u32))));
    result.push(AbstractOp::Op(Op::And));

    result
}

fn i64Shru () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(40 as u8))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Mod));
    result.push(AbstractOp::Op(Op::Shr));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(0xFFFFFFFFFFFFFFFF as u64))));
    result.push(AbstractOp::Op(Op::And));
    
    result
}

fn Shr () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Shr));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(0xFFFFFFFF as u32))));
    result.push(AbstractOp::Op(Op::And));
    
    result
}

fn i32Shl () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(20 as u8))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Mod));
    result.push(AbstractOp::Op(Op::Shl));
    result.push(AbstractOp::Op(Op::Push4(Imm::from(0xFFFFFFFF as u32))));
    result.push(AbstractOp::Op(Op::And));
    
    result
}

fn i64Shl () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Push1(Imm::from(40 as u8))));
    result.push(AbstractOp::Op(Op::Swap1));
    result.push(AbstractOp::Op(Op::Mod));
    result.push(AbstractOp::Op(Op::Shl));
    result.push(AbstractOp::Op(Op::Push8(Imm::from(0xFFFFFFFFFFFFFFFF as u64))));
    result.push(AbstractOp::Op(Op::And));
    
    result
}

fn Nop () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::JumpDest));

    result
}

fn unreachable () -> Vec<AbstractOp> {
    let mut result: Vec<AbstractOp> = Vec::new();

    result.push(AbstractOp::Op(Op::Invalid));

    result
}
