use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

pub type NativeFunction = fn(Vec<Variable>, &mut Runtime) -> FnResult;

#[derive(Clone, Copy)]
pub enum Function {
    Standard(usize, u32),
    Native(NativeFunction),
}

impl Function {
    pub fn call(&self, (args, runtime): (Vec<Variable>, &mut Runtime)) -> FnResult {
        match self {
            Function::Standard(file_no, fn_no) => {
                runtime.call_now(0, *fn_no as u16, args, *file_no)
            }
            Function::Native(func) => runtime.call_native(*func, args),
        }
    }

    pub fn call_or_goto(&self, (args, runtime): (Vec<Variable>, &mut Runtime)) -> FnResult {
        match self {
            Function::Standard(file_no, fn_no) => {
                runtime.push_stack(0, *fn_no as u16, args, *file_no);
                FnResult::Ok(())
            }
            Function::Native(func) => runtime.call_native(*func, args),
        }
    }

    pub fn to_str(&self, runtime: &mut Runtime) -> StringVar {
        match self {
            Function::Standard(file_no, fn_no) => runtime.get_fn_name(*file_no, *fn_no),
            Function::Native(func) => format!("fn@{}", *func as usize).into(),
        }
    }

    pub fn id(&self) -> usize {
        match self {
            Function::Standard(_file_no, _fn_no) => todo!("Unique ids for standard functions"),
            Function::Native(func) => func as *const _ as usize,
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Function::Standard(a1, a2), Function::Standard(b1, b2)) => a1 == b1 && a2 == b2,
            (Function::Native(x), Function::Native(y)) => *x as usize == *y as usize,
            _ => false,
        }
    }
}

impl Eq for Function {}

impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Function::Standard(a, b) => {
                a.hash(state);
                b.hash(state);
            }
            Function::Native(a) => {
                state.write_usize(*a as usize);
            }
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Standard(i, j) => f.debug_tuple("Standard").field(i).field(j).finish(),
            Function::Native(fn_) => f
                .debug_tuple("Native")
                .field(&format!("fn@{}", *fn_ as usize))
                .finish(),
        }
    }
}
