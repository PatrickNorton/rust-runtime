use crate::custom_var::CustomVar;
use crate::int_var::IntVar;
use crate::looping;
use crate::looping::{IterResult, NativeIterator};
use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::tuple::LangTuple;
use crate::variable::{FnResult, Variable};
use num::traits::Zero;
use num::BigInt;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Enumerate {
    iterable: looping::Iterator,
    i: RefCell<BigInt>,
}

impl Enumerate {
    pub fn new(iterable: looping::Iterator) -> Rc<Enumerate> {
        Rc::new(Enumerate {
            iterable,
            i: RefCell::new(Zero::zero()),
        })
    }

    iter_no_next!();

    fn next_fn(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        match self.inner_next(runtime)? {
            Option::Some(value) => runtime.return_n(vec![
                Option::Some(value.0).into(),
                Option::Some(value.1).into(),
            ]),
            Option::None => runtime.return_1(Option::None.into()),
        }
    }

    fn inner_next(&self, runtime: &mut Runtime) -> Result<Option<(Variable, Variable)>, ()> {
        if let Option::Some(val) = self.iterable.next(runtime)? {
            let i = self.i.borrow_mut();
            let index = IntVar::from(i.clone()).into();
            Result::Ok(Option::Some((index, val)))
        } else {
            Result::Ok(Option::None)
        }
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!()
    }

    fn enumerate_type() -> Type {
        custom_class!(Enumerate, create, "Enumerate")
    }
}

impl CustomVar for Enumerate {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        name.do_each(|o| self.get_op(o), |s| self.get_attribute(s))
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Self::enumerate_type()
    }
}

impl NativeIterator for Enumerate {
    fn next(self: Rc<Self>, runtime: &mut Runtime) -> IterResult {
        Result::Ok(
            self.inner_next(runtime)?
                .map(|(x, y)| LangTuple::new(Rc::from(vec![x, y])).into()),
        )
    }
}
