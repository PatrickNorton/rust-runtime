use crate::custom_types::exceptions::{invalid_state, io_error};
use crate::custom_var::CustomVar;
use crate::method::StdMethod;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Name, Variable};
use std::cell::RefCell;
use std::fs::File;
use std::mem::replace;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug)]
enum FileOption {
    Open(Option<File>),
    Closed(PathBuf),
}

#[derive(Debug)]
pub struct Open {
    file: RefCell<FileOption>,
}

impl Open {
    fn get_operator(self: Rc<Self>, op: Operator) -> Variable {
        let func = match op {
            Operator::Enter => Self::open,
            Operator::Exit => Self::close,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self, func))
    }

    fn open(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        match &*self.file.borrow() {
            FileOption::Open(_) => runtime.throw_quick(
                invalid_state(),
                "File cannot be opened more than once".into(),
            )?,
            FileOption::Closed(val) => match File::open(val) {
                Result::Ok(file) => {
                    self.file.replace(FileOption::Open(Option::Some(file)));
                }
                Result::Err(err) => runtime.throw_quick(io_error(), format!("{}", err).into())?,
            },
        }
        runtime.return_0()
    }

    fn close(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.file.replace(FileOption::Open(Option::None));
        runtime.return_0()
    }

    fn create(mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let path = StringVar::from(replace(&mut args[0], Variable::Null()));
        runtime.return_1(
            Rc::new(Open {
                file: RefCell::new(FileOption::Closed(PathBuf::from(path.as_str()))),
            })
            .into(),
        )
    }

    pub fn open_type() -> Type {
        custom_class!(Open, create, "open")
    }
}

impl CustomVar for Open {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Attribute(_) => unimplemented!(),
            Name::Operator(op) => self.get_operator(op),
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Self::open_type()
    }
}
