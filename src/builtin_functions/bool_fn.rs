use crate::builtin_functions::int_fn;
use crate::from_bool::FromBool;
use crate::int_var::IntVar;
use crate::method::{NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};

pub fn op_fn(o: Operator) -> Option<NativeMethod<bool>> {
    Option::Some(match o {
        Operator::Equals => eq,
        Operator::LessThan => less_than,
        Operator::GreaterThan => greater_than,
        Operator::LessEqual => less_equal,
        Operator::GreaterEqual => greater_equal,
        Operator::BitwiseAnd => bitwise_and,
        Operator::BitwiseOr => bitwise_or,
        Operator::BitwiseNot => bitwise_not,
        Operator::BitwiseXor => bitwise_xor,
        Operator::Str => str,
        Operator::Repr => str,
        _ => return Option::None,
    })
}

pub fn get_operator(this: bool, o: Operator) -> Variable {
    match op_fn(o) {
        Option::Some(func) => StdMethod::new_native(this, func).into(),
        Option::None => int_fn::get_operator(IntVar::from_bool(this), o),
    }
}

fn eq(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if bool::from(arg) != this {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn less_than(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if this >= bool::from(arg) {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn greater_than(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if this <= bool::from(arg) {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn less_equal(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if this & !bool::from(arg) {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn greater_equal(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if !this & bool::from(arg) {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn bitwise_and(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this;
    for arg in args {
        sum &= bool::from(arg)
    }
    runtime.return_1(sum.into())
}

fn bitwise_or(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this;
    for arg in args {
        sum |= bool::from(arg)
    }
    runtime.return_1(sum.into())
}

fn bitwise_not(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1((!this).into())
}

fn bitwise_xor(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this;
    for arg in args {
        sum ^= bool::from(arg)
    }
    runtime.return_1(sum.into())
}

fn str(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(StringVar::from(if this { "true" } else { "false" }).into())
}

#[cfg(test)]
mod test {
    use crate::builtin_functions::bool_fn::{
        bitwise_and, bitwise_not, bitwise_or, bitwise_xor, eq, greater_equal, greater_than,
        less_equal, less_than, str,
    };
    use crate::runtime::Runtime;
    use crate::string_var::StringVar;

    #[test]
    fn bool_eq() {
        let v1 = Runtime::test(|runtime| eq(true, vec![true.into()], runtime));
        assert_eq!(v1, Result::Ok(true.into()));
        let v2 = Runtime::test(|runtime| eq(true, vec![false.into()], runtime));
        assert_eq!(v2, Result::Ok(false.into()));
        let v3 = Runtime::test(|runtime| eq(true, vec![true.into(), true.into()], runtime));
        assert_eq!(v3, Result::Ok(true.into()));
        let v4 = Runtime::test(|runtime| eq(true, vec![true.into(), false.into()], runtime));
        assert_eq!(v4, Result::Ok(false.into()));
    }

    #[test]
    fn bool_lt() {
        let v1 = Runtime::test(|runtime| less_than(true, vec![true.into()], runtime));
        assert_eq!(v1, Result::Ok(false.into()));
        let v2 = Runtime::test(|runtime| less_than(true, vec![false.into()], runtime));
        assert_eq!(v2, Result::Ok(false.into()));
        let v3 = Runtime::test(|runtime| less_than(false, vec![true.into()], runtime));
        assert_eq!(v3, Result::Ok(true.into()));
    }

    #[test]
    fn bool_gt() {
        let v1 = Runtime::test(|runtime| greater_than(true, vec![true.into()], runtime));
        assert_eq!(v1, Result::Ok(false.into()));
        let v2 = Runtime::test(|runtime| greater_than(true, vec![false.into()], runtime));
        assert_eq!(v2, Result::Ok(true.into()));
        let v3 = Runtime::test(|runtime| greater_than(false, vec![true.into()], runtime));
        assert_eq!(v3, Result::Ok(false.into()));
    }

    #[test]
    fn bool_le() {
        let v1 = Runtime::test(|runtime| less_equal(true, vec![true.into()], runtime));
        assert_eq!(v1, Result::Ok(true.into()));
        let v2 = Runtime::test(|runtime| less_equal(true, vec![false.into()], runtime));
        assert_eq!(v2, Result::Ok(false.into()));
        let v3 = Runtime::test(|runtime| less_equal(false, vec![true.into()], runtime));
        assert_eq!(v3, Result::Ok(true.into()));
    }

    #[test]
    fn bool_ge() {
        let v1 = Runtime::test(|runtime| greater_equal(true, vec![true.into()], runtime));
        assert_eq!(v1, Result::Ok(true.into()));
        let v2 = Runtime::test(|runtime| greater_equal(true, vec![false.into()], runtime));
        assert_eq!(v2, Result::Ok(true.into()));
        let v3 = Runtime::test(|runtime| greater_equal(false, vec![true.into()], runtime));
        assert_eq!(v3, Result::Ok(false.into()));
    }

    #[test]
    fn bw_and() {
        let v1 = Runtime::test(|runtime| bitwise_and(true, vec![true.into()], runtime));
        assert_eq!(v1, Result::Ok(true.into()));
        let v2 = Runtime::test(|runtime| bitwise_and(true, vec![false.into()], runtime));
        assert_eq!(v2, Result::Ok(false.into()));
        let v3 = Runtime::test(|runtime| bitwise_and(false, vec![true.into()], runtime));
        assert_eq!(v3, Result::Ok(false.into()));
        let v4 = Runtime::test(|runtime| bitwise_and(false, vec![false.into()], runtime));
        assert_eq!(v4, Result::Ok(false.into()));
    }

    #[test]
    fn bw_or() {
        let v1 = Runtime::test(|runtime| bitwise_or(true, vec![true.into()], runtime));
        assert_eq!(v1, Result::Ok(true.into()));
        let v2 = Runtime::test(|runtime| bitwise_or(true, vec![false.into()], runtime));
        assert_eq!(v2, Result::Ok(true.into()));
        let v3 = Runtime::test(|runtime| bitwise_or(false, vec![true.into()], runtime));
        assert_eq!(v3, Result::Ok(true.into()));
        let v4 = Runtime::test(|runtime| bitwise_or(false, vec![false.into()], runtime));
        assert_eq!(v4, Result::Ok(false.into()));
    }

    #[test]
    fn bw_xor() {
        let v1 = Runtime::test(|runtime| bitwise_xor(true, vec![true.into()], runtime));
        assert_eq!(v1, Result::Ok(false.into()));
        let v2 = Runtime::test(|runtime| bitwise_xor(true, vec![false.into()], runtime));
        assert_eq!(v2, Result::Ok(true.into()));
        let v3 = Runtime::test(|runtime| bitwise_xor(false, vec![true.into()], runtime));
        assert_eq!(v3, Result::Ok(true.into()));
        let v4 = Runtime::test(|runtime| bitwise_xor(false, vec![false.into()], runtime));
        assert_eq!(v4, Result::Ok(false.into()));
    }

    #[test]
    fn bw_not() {
        let v1 = Runtime::test(|runtime| bitwise_not(true, vec![], runtime));
        assert_eq!(v1, Result::Ok(false.into()));
        let v2 = Runtime::test(|runtime| bitwise_not(false, vec![], runtime));
        assert_eq!(v2, Result::Ok(true.into()));
    }

    #[test]
    fn to_string() {
        let v1 = Runtime::test(|runtime| str(true, vec![], runtime));
        assert_eq!(v1, Result::Ok(StringVar::from("true").into()));
        let v2 = Runtime::test(|runtime| str(false, vec![], runtime));
        assert_eq!(v2, Result::Ok(StringVar::from("false").into()));
    }
}
