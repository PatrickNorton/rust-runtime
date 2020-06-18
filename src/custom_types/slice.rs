use crate::custom_types::range::Range;
use crate::custom_var::CustomVar;
use crate::int_var::IntVar;
use crate::method::StdMethod;
use crate::name::Name;
use crate::option::LangOption;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::variable::{FnResult, Variable};
use num::{One, Zero};
use std::mem::take;
use std::rc::Rc;

#[derive(Debug)]
pub struct Slice {
    start: Option<IntVar>,
    stop: Option<IntVar>,
    step: Option<IntVar>,
}

impl Slice {
    fn new(start: Option<IntVar>, stop: Option<IntVar>, step: Option<IntVar>) -> Slice {
        Slice { start, stop, step }
    }

    fn make_range(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let iterable_len = IntVar::from(take(&mut args[0]));
        let start = self.start.clone().unwrap_or_else(Zero::zero);
        let stop = self.stop.clone().unwrap_or(iterable_len);
        let step = self.step.clone().unwrap_or_else(One::one);
        runtime.return_1(Rc::new(Range::new(start, stop, step)).into())
    }

    fn create(mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 3);
        let start = var_to_int(take(&mut args[0]));
        let stop = var_to_int(take(&mut args[1]));
        let step = var_to_int(take(&mut args[2]));
        let val = Slice::new(start, stop, step);
        runtime.return_1(Rc::new(val).into())
    }

    pub fn slice_type() -> Type {
        custom_class!(Slice, create, "Slice")
    }
}

impl CustomVar for Slice {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        if let Name::Attribute(s) = name {
            match s.as_str() {
                "start" => int_to_var(self.start.clone()),
                "stop" => int_to_var(self.stop.clone()),
                "step" => int_to_var(self.step.clone()),
                "toRange" => Variable::Method(StdMethod::new_native(self, Self::make_range)),
                _ => unimplemented!(),
            }
        } else {
            unimplemented!()
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Self::slice_type()
    }
}

fn int_to_var(value: Option<IntVar>) -> Variable {
    LangOption::new(value.map(Variable::from)).into()
}

fn var_to_int(value: Variable) -> Option<IntVar> {
    if value.is_null() {
        Option::None
    } else {
        Option::Some(value.into())
    }
}
