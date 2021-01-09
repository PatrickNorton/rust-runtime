use crate::function::NativeFunction;
use crate::runtime::Runtime;
use crate::variable::{FnResult, Variable};
use std::fmt::{self, Debug, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Property {
    Standard(StdProperty),
    Native(NativeProperty),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StdProperty {
    file_no: usize,
    getter: u32,
    setter: u32,
}

#[derive(Copy, Clone)]
pub struct NativeProperty {
    getter: NativeFunction,
    setter: NativeFunction,
}

impl Property {
    pub fn call_getter(&self, runtime: &mut Runtime, this: Variable) -> FnResult {
        let typ = this.get_type();
        let args = vec![this, typ.into()];
        match self {
            Property::Standard(std_prop) => {
                runtime.call_now(0, std_prop.getter as u16, args, std_prop.file_no)
            }
            Property::Native(nat_prop) => runtime.call_native(nat_prop.getter, args),
        }
    }

    pub fn call_setter(&self, runtime: &mut Runtime, this: Variable, val: Variable) -> FnResult {
        let typ = this.get_type();
        let args = vec![this, typ.into(), val];
        match self {
            Property::Standard(std_prop) => {
                runtime.call_now(0, std_prop.setter as u16, args, std_prop.file_no)
            }
            Property::Native(nat_prop) => runtime.call_native(nat_prop.setter, args),
        }
    }
}

impl StdProperty {
    pub fn new(file_no: usize, getter: u32, setter: u32) -> StdProperty {
        StdProperty {
            file_no,
            getter,
            setter,
        }
    }

    pub fn get_file_no(&self) -> usize {
        self.file_no
    }

    pub fn get_getter(&self) -> u32 {
        self.getter
    }

    pub fn get_setter(&self) -> u32 {
        self.setter
    }
}

impl Debug for NativeProperty {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeProperty")
            .field("getter", &format!("fn@{}", self.getter as usize))
            .field("setter", &format!("fn@{}", self.setter as usize));
        fmt::Result::Ok(())
    }
}

impl PartialEq for NativeProperty {
    fn eq(&self, other: &Self) -> bool {
        self.getter as usize == other.getter as usize
            && self.setter as usize == other.setter as usize
    }
}

impl Eq for NativeProperty {}

impl NativeProperty {
    pub fn new(getter: NativeFunction, setter: NativeFunction) -> NativeProperty {
        NativeProperty { getter, setter }
    }

    pub fn get_getter(&self) -> &NativeFunction {
        &self.getter
    }

    pub fn get_setter(&self) -> &NativeFunction {
        &self.setter
    }
}
