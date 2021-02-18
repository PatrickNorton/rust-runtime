use crate::int_var::IntVar;
use crate::looping::{self, IterAttrs, IterResult, NativeIterator};
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::variable::{FnResult, Variable};
use num::traits::Zero;
use num::One;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Enumerate {
    iterable: looping::Iterator,
    i: RefCell<IntVar>,
}

impl Enumerate {
    pub fn new(iterable: looping::Iterator) -> Rc<Enumerate> {
        Rc::new(Enumerate {
            iterable,
            i: RefCell::new(Zero::zero()),
        })
    }

    fn inner_next(&self, runtime: &mut Runtime) -> Result<Option<(Variable, Variable)>, ()> {
        if let Option::Some(val) = self.iterable.next(runtime)?.take_first() {
            let i = self.i.replace_with(|x| &*x + &IntVar::one());
            let index = i.into();
            Result::Ok(Option::Some((index, val)))
        } else {
            Result::Ok(Option::None)
        }
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!()
    }
}

impl IterAttrs for Enumerate {
    fn next_fn(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        match self.inner_next(runtime)? {
            Option::Some(value) => runtime.return_n(vec![
                Option::Some(value.0).into(),
                Option::Some(value.1).into(),
            ]),
            Option::None => runtime.return_1(Option::None.into()),
        }
    }

    fn get_type() -> Type {
        custom_class!(Enumerate, create, "Enumerate")
    }
}

impl NativeIterator for Enumerate {
    fn next(self: Rc<Self>, runtime: &mut Runtime) -> IterResult {
        Result::Ok(self.inner_next(runtime)?.map(|(x, y)| vec![x, y]).into())
    }
}
