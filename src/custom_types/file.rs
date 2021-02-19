use crate::custom_types::exceptions::{invalid_state, io_error};
use crate::custom_types::list::List;
use crate::custom_var::CustomVar;
use crate::first;
use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug)]
pub struct FileObj {
    path: PathBuf,
    file: RefCell<Option<File>>,
}

impl FileObj {
    fn open(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        if self.file.borrow().is_none() {
            match File::open(&self.path) {
                Result::Ok(file) => {
                    self.file.replace(Option::Some(file));
                }
                Result::Err(err) => runtime.throw_quick(io_error(), format!("{}", err))?,
            }
        } else {
            runtime.throw_quick(invalid_state(), "File cannot be opened more than once")?
        }
        runtime.return_1(self.into())
    }

    fn close(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.file.replace(Option::None);
        runtime.return_0()
    }

    fn read_lines(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let mut result = String::new();
        if self.file_do(|f| f.read_to_string(&mut result)).is_err() {
            runtime.throw_quick(io_error(), "Could not read from file")
        } else {
            let list: Vec<Variable> = result
                .lines()
                .map(|a| StringVar::from(a.to_owned()).into())
                .collect();
            runtime.return_1(List::from_values(Type::String, list).into())
        }
    }

    fn read(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let mut result = String::new();
        if self.file_do(|f| f.read_to_string(&mut result)).is_err() {
            runtime.throw_quick(io_error(), "Could not read from file")
        } else {
            runtime.return_1(result.into())
        }
    }

    fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let path = StringVar::from(first(args));
        runtime.return_1(
            Rc::new(FileObj {
                path: (*path).into(),
                file: RefCell::new(Option::None),
            })
            .into(),
        )
    }

    fn file_do<T>(&self, func: impl FnOnce(&mut File) -> T) -> T {
        match &mut *self.file.borrow_mut() {
            Option::Some(f) => func(f),
            Option::None => panic!("File is not open"),
        }
    }

    pub fn open_type() -> Type {
        custom_class!(FileObj, create, "File")
    }
}

impl CustomVar for FileObj {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Self::open_type()
    }

    fn get_operator(self: Rc<Self>, op: Operator) -> Variable {
        let func = match op {
            Operator::Enter => Self::open,
            Operator::Exit => Self::close,
            _ => unimplemented!(),
        };
        StdMethod::new_native(self, func).into()
    }

    fn get_attribute(self: Rc<Self>, attr: &str) -> Variable {
        let func = match attr {
            "readLines" => Self::read_lines,
            "read" => Self::read,
            _ => unimplemented!(),
        };
        StdMethod::new_native(self, func).into()
    }
}
