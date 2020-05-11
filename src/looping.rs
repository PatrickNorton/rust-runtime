use crate::custom_types::exceptions::stop_iteration;
use crate::runtime::Runtime;
use crate::variable::{FnResult, Variable};

pub fn for_next(val: Variable, runtime: &mut Runtime) -> Result<Option<Variable>, ()> {
    let result = runtime.call_attr(val, "next".into(), Vec::new());
    match result {
        FnResult::Ok(_) => Result::Ok(Option::Some(runtime.pop_return())),
        FnResult::Err(_) => {
            let error = runtime.pop();
            if error.get_type() == stop_iteration() {
                Result::Ok(Option::None)
            } else {
                runtime.push(error);
                Result::Err(())
            }
        }
    }
}
