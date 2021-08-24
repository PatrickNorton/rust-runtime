use crate::custom_var::{downcast_var, CustomVar};
use crate::method::{NativeMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use crate::{first, first_n};
use std::rc::Rc;

macro_rules! float_impl {
    ($name:ident, $primitive:ty) => {
        #[derive(Debug, Copy, Clone)]
        pub struct $name {
            value: $primitive,
        }

        impl $name {
            fn new(value: $primitive) -> Self {
                Self { value }
            }

            fn op_fn(o: Operator) -> NativeMethod<Rc<Self>> {
                match o {
                    Operator::Add => Self::add,
                    Operator::Subtract => Self::sub,
                    Operator::USubtract => Self::negate,
                    Operator::Multiply => Self::mul,
                    Operator::Divide => Self::div,
                    Operator::Power => Self::pow,
                    Operator::Equals => Self::eq,
                    Operator::NotEquals => Self::ne,
                    Operator::GreaterThan => Self::gt,
                    Operator::LessThan => Self::lt,
                    Operator::GreaterEqual => Self::ge,
                    Operator::LessEqual => Self::le,
                    Operator::Modulo => Self::modulo,
                    Operator::Str => Self::str,
                    Operator::Repr => Self::str,
                    Operator::Bool => Self::to_bool,
                    Operator::Int => Self::to_int,
                    Operator::Hash => Self::hash,
                    _ => unimplemented!("{}.{}", stringify!($name), o.name()),
                }
            }

            fn attr_fn(s: &str) -> NativeMethod<Rc<Self>> {
                match s {
                    "mulAdd" => Self::mul_add,
                    _ => unimplemented!("{}.{}", stringify!($name), s),
                }
            }

            fn to_int(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert!(args.is_empty());
                runtime.return_1((self.value as i64).into())
            }

            fn hash(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert!(args.is_empty());
                runtime.return_1(self.value.to_bits().into())
            }

            fn to_bool(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert!(args.is_empty());
                runtime.return_1((self.value != 0.).into())
            }

            fn str(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert!(args.is_empty());
                runtime.return_1(StringVar::from(self.value.to_string()).into())
            }

            primitive_arithmetic!();

            fn pow(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert_eq!(args.len(), 1);
                let other = downcast_var::<Self>(first(args)).unwrap();
                let result = Self::new(self.value.powf(other.value));
                runtime.return_1(Rc::new(result).into())
            }

            fn negate(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert!(args.is_empty());
                runtime.return_1(Rc::new(Self::new(-self.value)).into())
            }

            primitive_comparisons!();

            fn mul_add(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert_eq!(args.len(), 2);
                let [a, b] = first_n(args);
                let first = downcast_var::<Self>(a).unwrap();
                let second = downcast_var::<Self>(b).unwrap();
                let result = self.value.mul_add(first.value, second.value);
                runtime.return_1(Rc::new(Self::new(result)).into())
            }
        }

        primitive_custom!($name);
    };
}

float_impl!(Float32, f32);
float_impl!(Float64, f64);
