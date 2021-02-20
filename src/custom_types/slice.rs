use crate::custom_types::exceptions::value_error;
use crate::custom_types::range::Range;
use crate::custom_var::CustomVar;
use crate::int_var::IntVar;
use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, InnerVar, Variable};
use crate::{first, first_three};
use num::{One, Signed, Zero};
use std::borrow::Cow;
use std::rc::Rc;

#[derive(Debug)]
pub struct Slice {
    start: Option<IntVar>,
    stop: Option<IntVar>,
    step: Option<IntVar>,
}

impl Slice {
    pub fn new(start: Option<IntVar>, stop: Option<IntVar>, step: Option<IntVar>) -> Slice {
        Slice { start, stop, step }
    }

    pub fn from_vars(start: Variable, stop: Variable, step: Variable) -> Rc<Slice> {
        Rc::new(Slice::new(
            unwrapped_to_int(start),
            unwrapped_to_int(stop),
            unwrapped_to_int(step),
        ))
    }

    fn str(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.str_value().into())
    }

    fn str_value(&self) -> StringVar {
        format!(
            "slice({}, {}, {})",
            stringify(&self.start),
            stringify(&self.stop),
            stringify(&self.step),
        )
        .into()
    }

    fn make_range(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        /*
        [::] or [:] -> [0:len:1]
        [:x:] or [:x] -> [0:x:1]
        [x::] or [x:] -> [x:len:1]
        [x:y:] or [x:y] -> [x:y:1]
        [:-x:] or [:-x] -> [0:len-x:1]
        [-x::] or [-x:] -> [len-x:len:1]
        [::-y] -> [len-1:-1:-y]
        [x::-y] -> [x:-1:-y]
        [:x:-y] -> [len-1:x:-y]
        */
        let len = IntVar::from(first(args));
        let step = self.step.clone().unwrap_or_else(One::one);
        if step.is_zero() {
            runtime.throw_quick(value_error(), "Step cannot be 0")
        } else if step.is_positive() {
            let start = self
                .start
                .as_ref()
                .map(|x| if x.is_negative() { &len + x } else { x.clone() })
                .unwrap_or_else(Zero::zero);
            let stop = self
                .stop
                .as_ref()
                .map(|x| if x.is_negative() { &len + x } else { x.clone() })
                .unwrap_or(len);
            runtime.return_1(Rc::new(Range::new(start, stop, step)).into())
        } else {
            // step.is_negative()
            let start = self
                .start
                .as_ref()
                .map(|x| if x.is_negative() { &len + x } else { x.clone() })
                .unwrap_or_else(|| &len - &1.into());
            let stop = self
                .start
                .as_ref()
                .map(|x| if x.is_negative() { &len + x } else { x.clone() })
                .unwrap_or_else(|| (-1).into());
            runtime.return_1(Rc::new(Range::new(start, stop, step)).into())
        }
    }

    fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 3);
        let (start, stop, step) = first_three(args);
        let val = Slice::new(var_to_int(start), var_to_int(stop), var_to_int(step));
        if val.step.as_ref().map_or_else(|| false, Zero::is_zero) {
            runtime.throw_quick(value_error(), "Step cannot be 0")
        } else {
            runtime.return_1(Rc::new(val).into())
        }
    }

    pub fn slice_type() -> Type {
        custom_class!(Slice, create, "Slice")
    }
}

impl CustomVar for Slice {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Self::slice_type()
    }

    fn get_operator(self: Rc<Self>, op: Operator) -> Variable {
        match op {
            Operator::Str | Operator::Repr => StdMethod::new_native(self, Self::str).into(),
            _ => unimplemented!(),
        }
    }

    fn get_attribute(self: Rc<Self>, name: &str) -> Variable {
        match name {
            "start" => int_to_var(self.start.clone()),
            "stop" => int_to_var(self.stop.clone()),
            "step" => int_to_var(self.step.clone()),
            "toRange" => StdMethod::new_native(self, Self::make_range).into(),
            _ => unimplemented!(),
        }
    }

    fn str(self: Rc<Self>, _runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(self.str_value())
    }

    fn repr(self: Rc<Self>, _runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(self.str_value())
    }
}

fn int_to_var(value: Option<IntVar>) -> Variable {
    value.map(Variable::from).into()
}

fn var_to_int(value: Variable) -> Option<IntVar> {
    if let Variable::Option(i, val) = value {
        if i == 1 {
            val.map(InnerVar::into).map(Variable::into)
        } else {
            panic!(
                "var_to_int expected a one-deep option, not {:?}",
                Variable::Option(i, val)
            )
        }
    } else {
        panic!("var_to_int expected an option, not {:?}", value)
    }
}

fn unwrapped_to_int(value: Variable) -> Option<IntVar> {
    match value {
        Variable::Normal(InnerVar::Null()) => Option::None,
        x => Option::Some(x.into()),
    }
}

fn stringify(val: &Option<IntVar>) -> Cow<'static, str> {
    match val {
        Option::Some(x) => format!("Some({})", x).into(),
        Option::None => "None".into(),
    }
}
