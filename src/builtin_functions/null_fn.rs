use crate::method::{NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};

pub fn op_fn(o: Operator) -> NativeMethod<()> {
    match o {
        Operator::Equals => eq,
        Operator::Str => str,
        Operator::Repr => str,
        Operator::Bool => bool,
        x => unimplemented!("null.{}", x.name()),
    }
}

pub fn get_operator(o: Operator) -> Variable {
    let func = op_fn(o);
    StdMethod::new_native((), func).into()
}

fn eq(_this: (), args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if !arg.is_null() {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn str(_this: (), args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(StringVar::from("null").into())
}

fn bool(_this: (), args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(false.into())
}

#[cfg(test)]
mod test {
    use crate::builtin_functions::null_fn::{bool, eq, str};
    use crate::runtime::Runtime;
    use crate::string_var::StringVar;
    use crate::variable::Variable;

    #[test]
    fn equal() {
        let a = Variable::null();
        let result = Runtime::test(|runtime| eq((), vec![a], runtime));
        assert_eq!(result, Result::Ok(true.into()));
    }

    #[test]
    fn string() {
        let result = Runtime::test(|runtime| str((), vec![], runtime));
        assert_eq!(result, Result::Ok(StringVar::from("null").into()))
    }

    #[test]
    fn boolean() {
        let result = Runtime::test(|runtime| bool((), vec![], runtime));
        assert_eq!(result, Result::Ok(false.into()));
    }
}
