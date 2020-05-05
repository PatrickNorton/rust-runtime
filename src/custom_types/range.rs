use crate::custom_types::types::CustomType;
use crate::custom_var::{downcast_var, CustomVar};
use crate::function::Function;
use crate::method::StdMethod;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Name, Variable};
use num::BigInt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::replace;
use std::rc::Rc;

#[derive(Debug)]
pub struct Range {
    start: BigInt,
    stop: BigInt,
    step: BigInt,
}

impl Range {
    fn new(start: BigInt, stop: BigInt, step: BigInt) -> Range {
        Range { start, stop, step }
    }

    fn get_stop(&self) -> &BigInt {
        &self.stop
    }

    fn get_step(&self) -> &BigInt {
        &self.step
    }

    fn get_op(self: &Rc<Self>, op: Operator) -> Variable {
        let func = match op {
            Operator::Str => Self::str,
            Operator::Repr => Self::str,
            Operator::Equals => Self::eq,
            Operator::Iter => Self::iter,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn str(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.push(format!("[{}:{}:{}]", self.start, self.stop, self.step).into());
        FnResult::Ok(())
    }

    fn eq(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let is_eq = match downcast_var::<Range>(replace(&mut args[0], Variable::Null())) {
            Option::None => false,
            Option::Some(other) => {
                self.start == other.start && self.stop == other.stop && self.step == other.step
            }
        };
        runtime.push(is_eq.into());
        FnResult::Ok(())
    }

    fn iter(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.push(Rc::new(RangeIter::new(self.clone())).into());
        FnResult::Ok(())
    }

    fn create(mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 3);
        let step = args.remove(2);
        let stop = args.remove(1);
        let start = args.remove(0);
        let range = Range::new(start.into(), stop.into(), step.into());
        runtime.push(Rc::new(range).into());
        FnResult::Ok(())
    }

    pub fn range_type() -> Type {
        lazy_static! {
            static ref TYPE: CustomType<Range> = CustomType::new(
                "list".into(),
                Vec::new(),
                Function::Native(Range::create),
                HashMap::new()
            );
        }
        Type::Custom(&*TYPE)
    }
}

impl CustomVar for Range {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Operator(op) => self.get_op(op),
            Name::Attribute(_) => unimplemented!(),
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Self::range_type()
    }
}

#[derive(Debug)]
struct RangeIter {
    current: RefCell<BigInt>,
    value: Rc<Range>,
}

impl RangeIter {
    pub fn new(value: Rc<Range>) -> RangeIter {
        RangeIter {
            current: RefCell::new(value.start.clone()),
            value,
        }
    }

    fn get_attribute(self: &Rc<Self>, val: StringVar) -> Variable {
        let func = match val.as_str() {
            "next" => Self::next_fn,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn next_fn(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        match self.next() {
            Option::Some(value) => {
                runtime.push(value.into());
                FnResult::Ok(())
            }
            Option::None => runtime.throw_quick(Type::String, "".into()),
        }
    }

    fn next(&self) -> Option<BigInt> {
        if &*self.current.borrow() != self.value.get_stop() {
            let result = self.current.borrow().clone();
            *self.current.borrow_mut() += self.value.get_step();
            Option::Some(result)
        } else {
            Option::None
        }
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!()
    }

    fn range_iter_type() -> Type {
        lazy_static! {
            static ref TYPE: CustomType<RangeIter> = CustomType::new(
                "list".into(),
                Vec::new(),
                Function::Native(RangeIter::create),
                HashMap::new()
            );
        }
        Type::Custom(&*TYPE)
    }
}

impl CustomVar for RangeIter {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Operator(_) => unimplemented!(),
            Name::Attribute(s) => self.get_attribute(s),
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Self::range_iter_type()
    }
}
