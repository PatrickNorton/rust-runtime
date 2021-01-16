use crate::function::Function;
use crate::int_var::IntVar;
use crate::lang_union::LangUnion;
use crate::looping;
use crate::method::Method;
use crate::rational_var::RationalVar;
use crate::std_type::Type;
use crate::std_variable::StdVariable;
use crate::string_var::StringVar;
use crate::tuple::LangTuple;
use crate::variable::{InnerVar, OptionVar, Variable};
use ascii::AsciiChar;
use num::BigInt;

impl From<InnerVar> for Variable {
    fn from(x: InnerVar) -> Self {
        Variable::Normal(x)
    }
}

impl From<(usize, Option<InnerVar>)> for Variable {
    fn from(x: (usize, Option<InnerVar>)) -> Self {
        Variable::Option(x.0, x.1)
    }
}

impl From<OptionVar> for Option<Variable> {
    fn from(x: OptionVar) -> Self {
        if x.0 == 1 {
            x.1.map(Variable::from)
        } else {
            Option::Some(Variable::Option(x.0 - 1, x.1))
        }
    }
}

impl From<IntVar> for Variable {
    fn from(x: IntVar) -> Self {
        Variable::Normal(InnerVar::Bigint(x))
    }
}

impl From<RationalVar> for Variable {
    fn from(x: RationalVar) -> Self {
        Variable::Normal(InnerVar::Decimal(x))
    }
}

impl From<StdVariable> for Variable {
    fn from(x: StdVariable) -> Self {
        Variable::Normal(InnerVar::Standard(x))
    }
}

impl From<LangUnion> for Variable {
    fn from(x: LangUnion) -> Self {
        Variable::Normal(InnerVar::Union(x))
    }
}

impl From<String> for Variable {
    fn from(x: String) -> Self {
        Variable::Normal(InnerVar::String(x.into()))
    }
}

impl From<StringVar> for Variable {
    fn from(x: StringVar) -> Self {
        Variable::Normal(InnerVar::String(x))
    }
}

impl From<Type> for Variable {
    fn from(x: Type) -> Self {
        Variable::Normal(InnerVar::Type(x))
    }
}

impl From<LangTuple> for Variable {
    fn from(x: LangTuple) -> Self {
        Variable::Normal(InnerVar::Tuple(x))
    }
}

impl From<Option<Variable>> for Variable {
    fn from(x: Option<Variable>) -> Self {
        match x {
            Option::None => Variable::Option(1, Option::None),
            Option::Some(Variable::Normal(x)) => Variable::Option(1, Option::Some(x)),
            Option::Some(Variable::Option(i, val)) => Variable::Option(i + 1, val),
        }
    }
}

impl From<Box<dyn Method>> for Variable {
    fn from(x: Box<dyn Method>) -> Self {
        Variable::Normal(InnerVar::Method(x))
    }
}

impl<T> From<Box<T>> for Variable
where
    T: Method + 'static,
{
    fn from(x: Box<T>) -> Self {
        Variable::Normal(InnerVar::Method(x))
    }
}

impl From<Function> for Variable {
    fn from(x: Function) -> Self {
        Variable::Normal(InnerVar::Function(x))
    }
}

impl From<bool> for Variable {
    fn from(x: bool) -> Self {
        Variable::Normal(InnerVar::Bool(x))
    }
}

impl From<char> for Variable {
    fn from(x: char) -> Self {
        Variable::Normal(InnerVar::Char(x))
    }
}

impl From<AsciiChar> for Variable {
    fn from(chr: AsciiChar) -> Self {
        chr.as_char().into()
    }
}

impl From<()> for Variable {
    fn from(_: ()) -> Self {
        Variable::Normal(InnerVar::Null())
    }
}

impl From<Variable> for IntVar {
    fn from(var: Variable) -> Self {
        match var {
            Variable::Normal(InnerVar::Bigint(i)) => i,
            Variable::Normal(InnerVar::Bool(b)) => if b { 1 } else { 0 }.into(),
            x => panic!(
                "Attempted to turn a variable not a superclass of int ({}) into an int",
                x.get_type().str()
            ),
        }
    }
}

impl From<Variable> for RationalVar {
    fn from(var: Variable) -> Self {
        if let Variable::Normal(InnerVar::Decimal(d)) = var {
            d
        } else {
            panic!(
                "Attempted to turn a variable not a superclass of dec ({}) into a dec",
                var.get_type().str()
            )
        }
    }
}

impl From<Variable> for StringVar {
    fn from(var: Variable) -> Self {
        if let Variable::Normal(InnerVar::String(s)) = var {
            s
        } else {
            panic!(
                "Attempted to turn a variable not a superclass of str ({}) into a str",
                var.get_type().str()
            )
        }
    }
}

impl From<Variable> for LangTuple {
    fn from(var: Variable) -> Self {
        if let Variable::Normal(InnerVar::Tuple(t)) = var {
            t
        } else {
            panic!(
                "Attempted to turn a variable not a superclass of tuple ({}) into a tuple",
                var.get_type().str()
            )
        }
    }
}

impl From<Variable> for bool {
    fn from(var: Variable) -> Self {
        if let Variable::Normal(InnerVar::Bool(b)) = var {
            b
        } else {
            panic!(
                "Attempted to turn a variable not a superclass of bool ({}) into a bool",
                var.get_type().str()
            )
        }
    }
}

impl From<Variable> for char {
    fn from(var: Variable) -> Self {
        if let Variable::Normal(InnerVar::Char(c)) = var {
            c
        } else {
            panic!(
                "Attempted to turn a variable not a superclass of char ({}) into a char",
                var.get_type().str()
            )
        }
    }
}

impl From<Variable> for Type {
    fn from(var: Variable) -> Self {
        if let Variable::Normal(InnerVar::Type(t)) = var {
            t
        } else {
            panic!(
                "Attempted to turn a variable not a type ({}) into a type",
                var.get_type().str()
            )
        }
    }
}

impl From<Variable> for looping::Iterator {
    fn from(var: Variable) -> Self {
        match var {
            Variable::Normal(InnerVar::Custom(var)) => var.into_inner().into_iter(),
            Variable::Normal(InnerVar::Standard(var)) => looping::Iterator::NonNative(var),
            _ => unimplemented!(),
        }
    }
}

impl Default for Variable {
    fn default() -> Self {
        Variable::Normal(InnerVar::Null())
    }
}

pub(crate) trait FromBool {
    fn from_bool(x: bool) -> Self;
}

impl FromBool for BigInt {
    fn from_bool(x: bool) -> Self {
        if x { 1u8 } else { 0u8 }.into()
    }
}

impl FromBool for IntVar {
    fn from_bool(x: bool) -> Self {
        if x { 1u8 } else { 0u8 }.into()
    }
}
