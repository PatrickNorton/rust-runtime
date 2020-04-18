use crate::bytecode::{bytecode_size, Bytecode};
use crate::int_tools::bytes_index;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::variable::{FnResult, Name, Variable};
use num_traits::FromPrimitive;

pub fn execute(runtime: &mut Runtime) -> FnResult {
    while !runtime.is_native() && runtime.current_pos() as usize != runtime.current_fn().len() {
        let bytes = runtime.current_fn();
        let b: Bytecode = FromPrimitive::from_u8(bytes[runtime.current_pos()]).unwrap();
        let byte_size = bytecode_size(b);
        let byte_start: usize = runtime.current_pos() + 1;
        let byte_0 = get_bytes(bytes, byte_start, byte_size.0);
        let byte_1 = get_bytes(bytes, byte_start + byte_size.0, byte_size.1);
        runtime.advance((byte_size.0 + byte_size.1 + 1) as u32);
        parse(b, byte_0, byte_1, runtime)?;
        if runtime.current_pos() == runtime.current_fn().len() && !runtime.is_bottom_stack() {
            runtime.pop_stack();
        }
    }
    Result::Ok(())
}

fn get_bytes(bytes: &Vec<u8>, mut start: usize, byte_count: usize) -> u32 {
    return match byte_count {
        0 => 0,
        2 => bytes_index::<u16>(bytes, &mut start) as u32,
        4 => bytes_index::<u32>(bytes, &mut start),
        _ => panic!("Invalid number for bytes: {}", byte_count),
    };
}

fn call_operator(o: Operator, argc: u16, runtime: &mut Runtime) -> FnResult {
    let argv = runtime.load_args(argc);
    let caller = runtime.pop();
    runtime.call_op(caller, o, argv)
}

fn bool_op(b: Bytecode, runtime: &mut Runtime) -> Result<bool, ()> {
    let x = runtime.pop_bool()?;
    let y = runtime.pop_bool()?;
    Result::Ok(match b {
        Bytecode::BoolAnd => x && y,
        Bytecode::BoolOr => x || y,
        Bytecode::BoolXor => x ^ y,
        _ => unreachable!(),
    })
}

fn parse(b: Bytecode, bytes_0: u32, bytes_1: u32, runtime: &mut Runtime) -> FnResult {
    match b {
        Bytecode::Nop => {}
        Bytecode::LoadNull => {
            runtime.push(Variable::Null());
        }
        Bytecode::LoadConst => {
            let const_val = runtime.load_const(bytes_0 as u16).clone();
            runtime.push(const_val)
        }
        Bytecode::LoadValue => {
            let value = runtime.load_value(bytes_0 as u16).clone();
            runtime.push(value)
        }
        Bytecode::LoadDot => {
            let dot_val = runtime.load_const(bytes_0 as u16).clone();
            let index = runtime.pop().index(Name::Attribute(dot_val.str(runtime)?));
            runtime.push(index)
        }
        Bytecode::LoadSubscript => call_operator(Operator::GetAttr, bytes_0 as u16, runtime)?,
        Bytecode::LoadOp => {
            let top = runtime.pop();
            let index: Operator = FromPrimitive::from_u16(bytes_0 as u16).unwrap();
            runtime.push(top.index(Name::Operator(index)))
        }
        Bytecode::PopTop => {
            runtime.pop();
        }
        Bytecode::DupTop => {
            let top = runtime.top().clone();
            runtime.push(top)
        }
        Bytecode::Swap2 => {
            let old_top = runtime.pop();
            let new_top = runtime.pop();
            runtime.push(old_top);
            runtime.push(new_top);
        }
        Bytecode::Swap3 => {
            let old_top = runtime.pop();
            let middle = runtime.pop();
            let new_top = runtime.pop();
            runtime.push(middle);
            runtime.push(old_top);
            runtime.push(new_top);
        }
        Bytecode::SwapN => {
            let swapped = bytes_0 as u16;
            let mut popped: Vec<Variable> = Vec::with_capacity(swapped as usize);
            for i in 0..swapped {
                popped[i as usize] = runtime.pop();
            }
            let last = popped.pop().unwrap();
            for _ in 0..swapped - 1 {
                runtime.push(popped.pop().unwrap());
            }
            runtime.push(last);
        }
        Bytecode::Store => {
            let stored = runtime.pop();
            runtime.store_variable(bytes_0 as u16, stored);
        }
        Bytecode::StoreSubscript => {
            let result = runtime.pop();
            let index = runtime.pop();
            let stored = runtime.pop();
            runtime.call_op(stored, Operator::SetAttr, vec![index, result])?;
        }
        Bytecode::StoreAttr => {
            let stored = runtime.pop();
            let value = runtime.pop();
            let attr_name = runtime.load_const(bytes_0 as u16).clone();
            let str_name = attr_name.str(runtime)?;
            value.set(str_name.into(), stored, runtime);
        }
        Bytecode::Plus => call_operator(Operator::Add, 1, runtime)?,
        Bytecode::Minus => call_operator(Operator::Subtract, 1, runtime)?,
        Bytecode::Times => call_operator(Operator::Multiply, 1, runtime)?,
        Bytecode::Divide => call_operator(Operator::Divide, 1, runtime)?,
        Bytecode::FloorDiv => call_operator(Operator::FloorDiv, 1, runtime)?,
        Bytecode::Mod => call_operator(Operator::Modulo, 1, runtime)?,
        Bytecode::Subscript => call_operator(Operator::GetAttr, 1, runtime)?,
        Bytecode::Power => call_operator(Operator::Power, 1, runtime)?,
        Bytecode::LBitshift => call_operator(Operator::LeftBitshift, 1, runtime)?,
        Bytecode::RBitshift => call_operator(Operator::RightBitshift, 1, runtime)?,
        Bytecode::BitwiseAnd => call_operator(Operator::BitwiseAnd, 1, runtime)?,
        Bytecode::BitwiseOr => call_operator(Operator::BitwiseOr, 1, runtime)?,
        Bytecode::BitwiseXor => call_operator(Operator::BitwiseXor, 1, runtime)?,
        Bytecode::Compare => call_operator(Operator::Compare, 1, runtime)?,
        Bytecode::DelSubscript => call_operator(Operator::DelAttr, 2, runtime)?,
        Bytecode::UMinus => call_operator(Operator::USubtract, 0, runtime)?,
        Bytecode::BitwiseNot => call_operator(Operator::BitwiseNot, 0, runtime)?,
        Bytecode::BoolAnd => {
            let result = bool_op(Bytecode::BoolAnd, runtime)?;
            runtime.push(Variable::Bool(result))
        }
        Bytecode::BoolOr => {
            let result = bool_op(Bytecode::BoolOr, runtime)?;
            runtime.push(Variable::Bool(result))
        }
        Bytecode::BoolNot => {
            let result = !runtime.pop_bool()?;
            runtime.push(Variable::Bool(result))
        }
        Bytecode::BoolXor => {
            let result = bool_op(Bytecode::BoolXor, runtime)?;
            runtime.push(Variable::Bool(result))
        }
        Bytecode::Identical => {
            let x = runtime.pop();
            let y = runtime.pop();
            runtime.push(Variable::Bool(x.identical(&y)))
        }
        Bytecode::Instanceof => {
            let x = runtime.pop();
            let y = runtime.pop();
            runtime.push(Variable::Bool(y.is_type_of(&x)))
        }
        Bytecode::CallOp => {
            let op: Operator = FromPrimitive::from_u32(bytes_0)
                .expect(format!("operator {} not found", bytes_0).as_ref());
            call_operator(op, bytes_1 as u16, runtime)?
        }
        Bytecode::PackTuple => unimplemented!(),
        Bytecode::UnpackTuple => unimplemented!(),
        Bytecode::Equal => call_operator(Operator::Equals, 1, runtime)?,
        Bytecode::LessThan => call_operator(Operator::LessThan, 1, runtime)?,
        Bytecode::GreaterThan => call_operator(Operator::GreaterThan, 1, runtime)?,
        Bytecode::LessEqual => call_operator(Operator::LessEqual, 1, runtime)?,
        Bytecode::GreaterEqual => call_operator(Operator::GreaterEqual, 1, runtime)?,
        Bytecode::Contains => call_operator(Operator::In, 1, runtime)?,
        Bytecode::Jump => runtime.goto(bytes_0),
        Bytecode::JumpTrue => {
            if runtime.pop().to_bool(runtime)? {
                runtime.goto(bytes_0)
            }
        }
        Bytecode::JumpFalse => {
            if !runtime.pop().to_bool(runtime)? {
                runtime.goto(bytes_0)
            }
        }
        Bytecode::JumpNN => {
            if let Variable::Null() = runtime.pop() {
            } else {
                runtime.goto(bytes_0)
            }
        }
        Bytecode::JumpNull => {
            if let Variable::Null() = runtime.pop() {
                runtime.goto(bytes_0)
            }
        }
        Bytecode::CallMethod => {
            let fn_index = bytes_0 as u16;
            let fn_var = runtime.load_const(fn_index).clone();
            let fn_name = fn_var.str(runtime)?;
            let argc = bytes_1 as u16;
            let args = runtime.load_args(argc);
            let var = runtime.pop();
            var.index(Name::Attribute(fn_name)).call((args, runtime))?;
        }
        Bytecode::CallTos => {
            let argc = bytes_0 as u16;
            runtime.call_tos(argc)?;
        }
        Bytecode::CallFunction => runtime.call_quick(bytes_0 as u16)?,
        Bytecode::TailMethod => unimplemented!(),
        Bytecode::TailTos => unimplemented!(),
        Bytecode::Return => runtime.pop_stack(),
        _ => unimplemented!("Bytecode {:?} not implemented", b),
    }
    FnResult::Ok(())
}
