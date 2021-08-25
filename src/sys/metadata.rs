use crate::custom_var::CustomVar;
use crate::first;
use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::sys::os_do_1;
use crate::variable::{FnResult, Variable};
use std::fs::{FileType, Metadata, Permissions};
use std::rc::Rc;

pub fn metadata(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    os_do_1(first(args), runtime, |x| {
        std::fs::metadata(x).map(LangMetadata::new)
    })
}

#[derive(Debug)]
struct LangMetadata {
    value: Metadata,
}

#[derive(Debug)]
struct LangFileType {
    value: FileType,
}

#[derive(Debug)]
struct LangPermissions {
    value: Permissions,
}

impl LangMetadata {
    fn new(value: Metadata) -> Rc<LangMetadata> {
        Rc::new(LangMetadata { value })
    }

    fn repr(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(format!("{:?}", self.value).into())
    }
}

impl LangFileType {
    fn new(value: FileType) -> Rc<LangFileType> {
        Rc::new(LangFileType { value })
    }
}

impl LangPermissions {
    fn new(value: Permissions) -> Rc<LangPermissions> {
        Rc::new(LangPermissions { value })
    }

    fn mode(&self) -> Option<u32> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            Option::Some(self.value.mode())
        }
        #[cfg(not(unix))]
        Option::None
    }
}

impl CustomVar for LangMetadata {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        todo!()
    }

    fn get_operator(self: Rc<Self>, op: Operator) -> Variable {
        match op {
            Operator::Str | Operator::Repr => StdMethod::new_native(self, Self::repr).into(),
            _ => unimplemented!("Metadata.{}", op.name()),
        }
    }

    fn get_attribute(self: Rc<Self>, name: &str) -> Variable {
        match name {
            "isDir" => self.value.is_dir().into(),
            "isFile" => self.value.is_file().into(),
            "length" => self.value.len().into(),
            "fileType" => LangFileType::new(self.value.file_type()).into(),
            "permissions" => LangPermissions::new(self.value.permissions()).into(),
            x => unimplemented!("Metadata.{}", x),
        }
    }
}

impl CustomVar for LangFileType {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        todo!()
    }

    fn get_operator(self: Rc<Self>, _op: Operator) -> Variable {
        todo!()
    }

    fn get_attribute(self: Rc<Self>, name: &str) -> Variable {
        match name {
            "isDir" => self.value.is_dir().into(),
            "isFile" => self.value.is_file().into(),
            "isSymlink" => self.value.is_symlink().into(),
            x => unimplemented!("FileType.{}", x),
        }
    }
}

impl CustomVar for LangPermissions {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        unimplemented!()
    }

    fn get_operator(self: Rc<Self>, _op: Operator) -> Variable {
        unimplemented!()
    }

    fn get_attribute(self: Rc<Self>, name: &str) -> Variable {
        match name {
            "readOnly" => self.value.readonly().into(),
            "mode" => self.mode().map(From::from).into(),
            x => unimplemented!("Permissions.{}", x),
        }
    }
}
