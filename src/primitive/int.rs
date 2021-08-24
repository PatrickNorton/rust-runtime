use crate::custom_var::{downcast_var, CustomVar};
use crate::first;
use crate::method::{NativeMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::convert::TryInto;
use std::rc::Rc;

macro_rules! int_impl {
    ($name:ident, $primitive:ty) => {
        #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
        struct $name {
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
                    Operator::LeftBitshift => Self::shl,
                    Operator::RightBitshift => Self::shr,
                    Operator::BitwiseAnd => Self::and,
                    Operator::BitwiseOr => Self::or,
                    Operator::BitwiseNot => Self::not,
                    Operator::BitwiseXor => Self::xor,
                    _ => unimplemented!("{}.{}", stringify!($name), o.name()),
                }
            }

            fn attr_fn(s: &str) -> NativeMethod<Rc<Self>> {
                unimplemented!("{}.{}", stringify!($name), s)
            }

            fn to_int(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert!(args.is_empty());
                runtime.return_1((self.value as i64).into())
            }

            fn hash(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert!(args.is_empty());
                runtime.return_1(self.value.into())
            }

            fn to_bool(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert!(args.is_empty());
                runtime.return_1((self.value != 0).into())
            }

            fn str(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert!(args.is_empty());
                runtime.return_1(StringVar::from(self.value.to_string()).into())
            }

            primitive_arithmetic!();

            fn pow(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert_eq!(args.len(), 1);
                let other = downcast_var::<Self>(first(args)).unwrap();
                let result = Self::new(
                    other
                        .value
                        .try_into()
                        .map_or_else(|_| 0, |x| self.value.pow(x)),
                );
                runtime.return_1(Rc::new(result).into())
            }

            fn shl(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert_eq!(args.len(), 1);
                let other = downcast_var::<Self>(first(args)).unwrap();
                let result = Self::new(self.value << other.value);
                runtime.return_1(Rc::new(result).into())
            }

            fn shr(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert_eq!(args.len(), 1);
                let other = downcast_var::<Self>(first(args)).unwrap();
                let result = Self::new(self.value >> other.value);
                runtime.return_1(Rc::new(result).into())
            }

            fn and(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert_eq!(args.len(), 1);
                let other = downcast_var::<Self>(first(args)).unwrap();
                let result = Self::new(self.value & other.value);
                runtime.return_1(Rc::new(result).into())
            }

            fn or(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert_eq!(args.len(), 1);
                let other = downcast_var::<Self>(first(args)).unwrap();
                let result = Self::new(self.value | other.value);
                runtime.return_1(Rc::new(result).into())
            }

            fn not(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert_eq!(args.len(), 0);
                let result = Self::new(!self.value);
                runtime.return_1(Rc::new(result).into())
            }

            fn xor(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert_eq!(args.len(), 1);
                let other = downcast_var::<Self>(first(args)).unwrap();
                let result = Self::new(self.value ^ other.value);
                runtime.return_1(Rc::new(result).into())
            }

            fn negate(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert!(args.is_empty());
                runtime.return_1(Rc::new(Self::new(-self.value)).into())
            }

            primitive_comparisons!();
        }

        primitive_custom!($name);
    };
}

int_impl!(Int8, i8);
int_impl!(Int16, i16);
int_impl!(Int32, i32);
int_impl!(Int64, i64);
int_impl!(Int128, i128);
