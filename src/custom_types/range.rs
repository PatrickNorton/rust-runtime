use crate::custom_types::exceptions::index_error;
use crate::custom_var::{downcast_var, CustomVar};
use crate::int_var::IntVar;
use crate::looping::{IterResult, NativeIterator};
use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use num::{Signed, Zero};
use std::cell::RefCell;
use std::mem::replace;
use std::rc::Rc;

#[derive(Debug)]
pub struct Range {
    start: IntVar,
    stop: IntVar,
    step: IntVar,
}

impl Range {
    pub fn new(start: IntVar, stop: IntVar, step: IntVar) -> Range {
        Range { start, stop, step }
    }

    pub fn get_start(&self) -> &IntVar {
        &self.start
    }

    pub fn get_stop(&self) -> &IntVar {
        &self.stop
    }

    pub fn get_step(&self) -> &IntVar {
        &self.step
    }

    fn get_op(self: &Rc<Self>, op: Operator) -> Variable {
        let func = match op {
            Operator::Str => Self::str,
            Operator::Repr => Self::str,
            Operator::Equals => Self::eq,
            Operator::Iter => Self::iter,
            Operator::GetAttr => Self::index,
            Operator::In => Self::contains,
            Operator::Reversed => Self::reversed,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn str(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.to_str().into())
    }

    fn eq(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let is_eq = match downcast_var::<Range>(replace(&mut args[0], Variable::Null())) {
            Option::None => false,
            Option::Some(other) => {
                self.start == other.start && self.stop == other.stop && self.step == other.step
            }
        };
        runtime.return_1(is_eq.into())
    }

    fn iter(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Rc::new(RangeIter::new(self.clone())).into())
    }

    fn index(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let index = IntVar::from(replace(&mut args[0], Variable::Null()));
        let result = self.start.clone() + index * self.step.clone();
        let error = result == self.stop || (self.step.is_negative() ^ (result > self.stop));
        if error {
            runtime.throw_quick(
                index_error(),
                format!("Index {} out of bounds for {}", result, self.to_str()).into(),
            )
        } else {
            runtime.return_1(result.into())
        }
    }

    fn contains(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let value = IntVar::from(replace(&mut args[0], Variable::Null()));
        let result = if self.step.is_positive() {
            value >= self.start && value < self.stop
        } else {
            value <= self.start && value > self.stop
        } && ((&value - &self.start) % self.step.clone()).is_zero();
        runtime.return_1(result.into())
    }

    fn reversed(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let new_stop = &self.start - &self.step;
        let new_start = &self.stop - &self.step;
        let new_step = -self.step.clone();
        runtime.return_1(Rc::new(Self::new(new_start, new_stop, new_step)).into())
    }

    fn to_str(&self) -> StringVar {
        format!("[{}:{}:{}]", self.start, self.stop, self.step).into()
    }

    fn create(mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 3);
        let step = args.remove(2);
        let stop = args.remove(1);
        let start = args.remove(0);
        let range = Range::new(start.into(), stop.into(), step.into());
        runtime.return_1(Rc::new(range).into())
    }

    pub fn range_type() -> Type {
        custom_class!(Range, create, "range")
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
    current: RefCell<IntVar>,
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
        runtime.return_1(self.true_next().map(Variable::from).into())
    }

    fn true_next(&self) -> Option<IntVar> {
        if &*self.current.borrow() != self.value.get_stop() {
            let result = self.current.borrow().clone();
            *self.current.borrow_mut() += self.value.get_step().clone();
            Option::Some(result)
        } else {
            Option::None
        }
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!()
    }

    fn range_iter_type() -> Type {
        custom_class!(RangeIter, create, "RangeIter")
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

impl NativeIterator for RangeIter {
    fn next(self: Rc<Self>, _runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.true_next().map(Variable::from))
    }
}
