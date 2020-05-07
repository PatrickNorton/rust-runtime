use crate::bytecode::{bytecode_size, Bytecode};
use crate::custom_types::dict::Dict;
use crate::custom_types::exceptions::stop_iteration;
use crate::custom_types::list::List;
use crate::custom_types::set::Set;
use crate::int_tools::bytes_index;
use crate::operator::Operator;
use crate::quick_functions::{
    quick_add, quick_bitwise_and, quick_bitwise_not, quick_bitwise_or, quick_bitwise_xor,
    quick_div, quick_equals, quick_floor_div, quick_greater_equal, quick_greater_than,
    quick_left_bitshift, quick_less_equal, quick_less_than, quick_mod, quick_mul, quick_power,
    quick_right_bitshift, quick_sub, quick_subscript, quick_u_minus, QuickResult,
};
use crate::runtime::Runtime;
use crate::variable::{FnResult, Name, Variable};
use num::traits::FromPrimitive;
use num::Zero;
use std::ops::SubAssign;

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
    match byte_count {
        0 => 0,
        2 => bytes_index::<u16>(bytes, &mut start) as u32,
        4 => bytes_index::<u32>(bytes, &mut start),
        _ => panic!("Invalid number for bytes: {}", byte_count),
    }
}

fn call_operator(o: Operator, argc: u16, runtime: &mut Runtime) -> FnResult {
    let argv = runtime.load_args(argc);
    let caller = runtime.pop();
    caller.call_op_or_goto(o, argv, runtime)
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

#[inline]
fn quick_op_1(runtime: &mut Runtime, func: fn(Variable, &mut Runtime) -> QuickResult) -> FnResult {
    let x = runtime.pop();
    let result = func(x, runtime)?;
    runtime.push(result);
    FnResult::Ok(())
}

#[inline]
fn quick_op_2(
    runtime: &mut Runtime,
    func: fn(Variable, Variable, &mut Runtime) -> QuickResult,
) -> FnResult {
    let y = runtime.pop();
    let x = runtime.pop();
    let result = func(x, y, runtime)?;
    runtime.push(result);
    FnResult::Ok(())
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
        Bytecode::Plus => quick_op_2(runtime, quick_add)?,
        Bytecode::Minus => quick_op_2(runtime, quick_sub)?,
        Bytecode::Times => quick_op_2(runtime, quick_mul)?,
        Bytecode::Divide => quick_op_2(runtime, quick_div)?,
        Bytecode::FloorDiv => quick_op_2(runtime, quick_floor_div)?,
        Bytecode::Mod => quick_op_2(runtime, quick_mod)?,
        Bytecode::Subscript => quick_op_2(runtime, quick_subscript)?,
        Bytecode::Power => quick_op_2(runtime, quick_power)?,
        Bytecode::LBitshift => quick_op_2(runtime, quick_left_bitshift)?,
        Bytecode::RBitshift => quick_op_2(runtime, quick_right_bitshift)?,
        Bytecode::BitwiseAnd => quick_op_2(runtime, quick_bitwise_and)?,
        Bytecode::BitwiseOr => quick_op_2(runtime, quick_bitwise_or)?,
        Bytecode::BitwiseXor => quick_op_2(runtime, quick_bitwise_xor)?,
        Bytecode::Compare => call_operator(Operator::Compare, 1, runtime)?,
        Bytecode::DelSubscript => call_operator(Operator::DelAttr, 2, runtime)?,
        Bytecode::UMinus => quick_op_1(runtime, quick_u_minus)?,
        Bytecode::BitwiseNot => quick_op_1(runtime, quick_bitwise_not)?,
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
        Bytecode::Equal => quick_op_2(runtime, quick_equals)?,
        Bytecode::LessThan => quick_op_2(runtime, quick_less_than)?,
        Bytecode::GreaterThan => quick_op_2(runtime, quick_greater_than)?,
        Bytecode::LessEqual => quick_op_2(runtime, quick_less_equal)?,
        Bytecode::GreaterEqual => quick_op_2(runtime, quick_greater_equal)?,
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
            var.index(Name::Attribute(fn_name))
                .call_or_goto((args, runtime))?;
        }
        Bytecode::CallTos => {
            let argc = bytes_0 as u16;
            runtime.call_tos_or_goto(argc)?;
        }
        Bytecode::CallFunction => runtime.call_quick(bytes_0 as u16),
        Bytecode::TailMethod => unimplemented!(),
        Bytecode::TailTos => unimplemented!(),
        Bytecode::TailFunction => unimplemented!(),
        Bytecode::Return => runtime.pop_stack(),
        Bytecode::Throw => {
            let result = runtime.pop();
            return runtime.throw(result);
        }
        Bytecode::ThrowQuick => {
            let msg = runtime.pop();
            let exc_type = runtime.pop();
            if let Variable::Type(t) = exc_type {
                let msg_str = msg.str(runtime).unwrap();
                return runtime.throw_quick(t, msg_str);
            } else {
                panic!("ThrowQuick must be called with a type, not {:?}", exc_type);
            }
        }
        Bytecode::EnterTry => {
            let mut exc_pos = bytes_0 as usize;
            while {
                let bc: Option<Bytecode> = FromPrimitive::from_u8(runtime.current_fn()[exc_pos]);
                bc.unwrap()
            } == Bytecode::ExceptN
            {
                exc_pos += 1;
                let const_index = bytes_index::<u32>(runtime.current_fn(), &mut exc_pos);
                let exc_type = runtime.load_const(const_index as u16).clone();
                runtime.add_exception_handler(exc_type, exc_pos as u32);
            }
        }
        Bytecode::ExceptN => panic!("Bytecode::ExceptN should never be called"),
        Bytecode::Finally => panic!("Bytecode::Finally should never be called"),
        Bytecode::EndTry => {
            let count = bytes_0 as u16;
            for _ in 0..count {
                runtime.pop_handler();
            }
        }
        Bytecode::FuncDef => {
            unimplemented!("Bytecode::FuncDef is a marker bytecode and should not appear in code")
        }
        Bytecode::ClassDef => {
            unimplemented!("Bytecode::ClassDef is a marker bytecode and should not appear in code")
        }
        Bytecode::EndClass => {
            unimplemented!("Bytecode::EndClass is a marker bytecode and should not appear in code")
        }
        Bytecode::ForIter => {
            let iterated = runtime.pop();
            let jump_loc = bytes_0;
            runtime.add_exception_handler(stop_iteration().into(), jump_loc);
            runtime.call_attr(iterated.clone(), "next".into(), Vec::new())?;
            if runtime.current_pos() != jump_loc as usize {
                runtime.pop_handler();
                let arg = runtime.pop();
                runtime.push(iterated);
                runtime.push(arg);
            }
        }
        Bytecode::ListCreate => {
            let argc = bytes_0 as u16;
            let value = List::from_values(runtime.load_args(argc));
            runtime.push(value.into())
        }
        Bytecode::SetCreate => {
            let argc = bytes_0 as u16;
            let argv = runtime.load_args(argc);
            let value = Set::new(argv, runtime)?;
            runtime.push(value.into())
        }
        Bytecode::DictCreate => {
            let count = bytes_0 as u16;
            let mut keys: Vec<Variable> = Vec::with_capacity(count as usize);
            let mut values: Vec<Variable> = Vec::with_capacity(count as usize);
            for _ in 0..count {
                keys.push(runtime.pop());
                values.push(runtime.pop());
            }
            let value = Dict::from_args(keys, values, runtime)?;
            runtime.push(value.into())
        }
        Bytecode::ListAdd => {
            let added = runtime.pop();
            let list = runtime.pop();
            runtime.call_attr(list.clone(), "add".into(), vec![added])?;
            runtime.push(list)
        }
        Bytecode::SetAdd => {
            let added = runtime.pop();
            let set = runtime.pop();
            runtime.call_attr(set.clone(), "add".into(), vec![added])?;
            runtime.push(set)
        }
        Bytecode::DictAdd => {
            let value = runtime.pop();
            let key = runtime.pop();
            let dict = runtime.pop();
            runtime.call_op(dict.clone(), Operator::SetAttr, vec![key, value])?;
            runtime.push(dict);
        }
        Bytecode::Dotimes => {
            let mut value = runtime.pop().int(runtime)?;
            let jump = bytes_0;
            if !value.is_zero() {
                runtime.goto(jump);
            } else {
                value.sub_assign(1.into());
                runtime.push(value.into());
            }
        }
        Bytecode::DoStatic => {
            if !runtime.do_static() {
                runtime.goto(bytes_0);
            }
        }
        Bytecode::StoreStatic => {
            let var = runtime.pop();
            runtime.store_static(bytes_0 as usize, var);
        }
        Bytecode::LoadStatic => {
            let var = runtime.load_static(bytes_0 as usize);
            runtime.push(var);
        }
        Bytecode::LoadFunction => todo!(),
        Bytecode::GetType => {
            let value = runtime.pop();
            runtime.push(value.get_type().into());
        }
    }
    FnResult::Ok(())
}
