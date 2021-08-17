use crate::runtime::Runtime;
use crate::variable::{FnResult, InnerVar, Variable};
use std::time::Instant;

pub fn test_internal(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let start = Instant::now();
    let length = args.len();
    let mut failed = 0usize;
    for (i, arg) in args.into_iter().enumerate() {
        let function = match arg {
            Variable::Normal(InnerVar::Function(f)) => f,
            _ => panic!("Expected a function"),
        };
        let result = function.call((vec![], runtime));
        if result.is_err() {
            let error = runtime.pop_err().unwrap();
            println!("Test {} failed:\n{}", i, error.str(runtime).unwrap());
            failed += 1;
        }
    }
    let duration = start.elapsed();
    if failed == 0 {
        println!("All tests passed in {:?}", duration);
    } else {
        println!("{}/{} tests failed in {:?}", failed, length, duration);
    }
    FnResult::Ok(())
}
