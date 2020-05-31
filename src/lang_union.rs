use crate::int_var::IntVar;
use crate::lang_union::default_functions::default_methods;
use crate::looping;
use crate::method::{InnerMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::property::Property;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ptr;

pub type UnionMethod = InnerMethod<LangUnion>;

#[derive(Debug, Clone)]
pub struct LangUnion {
    variant_no: usize,
    value: Box<Variable>,
    cls: &'static UnionType,
}

#[derive(Debug)]
pub struct UnionType {
    name: StringVar,
    file_no: usize,
    supers: Vec<Type>,
    variants: Vec<StringVar>,
    variables: HashSet<StringVar>,
    methods: HashMap<Name, UnionMethod>,
    static_methods: HashMap<Name, UnionMethod>,
    properties: HashMap<StringVar, Property>,
}

impl LangUnion {
    pub fn new(variant_no: usize, value: Box<Variable>, cls: &'static UnionType) -> LangUnion {
        LangUnion {
            variant_no,
            value,
            cls,
        }
    }

    pub fn str(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.call_operator(Operator::Str, vec![], runtime)?;
        runtime.pop_return().str(runtime)
    }

    pub fn repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.call_operator(Operator::Repr, Vec::new(), runtime)?;
        Result::Ok(runtime.pop_return().into())
    }

    pub fn bool(&self, runtime: &mut Runtime) -> Result<bool, ()> {
        self.call_operator(Operator::Bool, vec![], runtime)?;
        runtime.pop_return().to_bool(runtime)
    }

    pub fn int(&self, runtime: &mut Runtime) -> Result<IntVar, ()> {
        self.call_operator(Operator::Bool, vec![], runtime)?;
        runtime.pop_return().int(runtime)
    }

    pub fn iter(&self, runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        self.call_operator(Operator::Bool, vec![], runtime)?;
        Result::Ok(runtime.pop_return().into())
    }

    pub fn call_operator(
        &self,
        op: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        let inner_method = self.cls.get_method(Name::Operator(op));
        inner_method.call(self, args, runtime)
    }

    pub fn call(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        self.call_operator(Operator::Call, args.0, args.1)
    }

    pub fn call_op_or_goto(
        &self,
        op: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        let inner_method = self.cls.get_method(Name::Operator(op));
        inner_method.call_or_goto(self, args, runtime)
    }

    pub fn call_or_goto(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        self.call_op_or_goto(Operator::Call, args.0, args.1)
    }

    pub fn index(&self, index: Name, runtime: &mut Runtime) -> Result<Variable, ()> {
        match self.cls.variant_pos(&index) {
            Option::Some(_true_val) => todo!("Option"),
            Option::None => self.index_harder(index, runtime),
        }
    }

    pub fn get_type(&self) -> Type {
        Type::Union(self.cls)
    }

    pub fn variant_name(&self) -> &StringVar {
        &self.cls.variants[self.variant_no]
    }

    fn index_harder(&self, index: Name, runtime: &mut Runtime) -> Result<Variable, ()> {
        match self.cls.get_property(&index) {
            Option::Some(val) => {
                val.call_getter(runtime)?;
                Result::Ok(runtime.pop_return())
            }
            Option::None => {
                let inner_method = self.cls.get_method(index);
                Result::Ok(Variable::Method(Box::new(StdMethod::new(
                    self.clone(),
                    inner_method,
                ))))
            }
        }
    }
}

impl UnionType {
    fn variant_pos(&self, index: &Name) -> Option<usize> {
        if let Name::Attribute(name) = index {
            self.variants.iter().position(|x| x == name)
        } else {
            Option::None
        }
    }

    pub fn name(&self) -> &StringVar {
        &self.name
    }

    pub fn get_property(&self, name: &Name) -> Option<&Property> {
        name.do_each_ref(|_| Option::None, |str| self.properties.get(&str))
    }

    pub(self) fn get_method(&self, name: Name) -> UnionMethod {
        match self.methods.get(&name) {
            Option::Some(t) => *t,
            Option::None => default_methods(name),
        }
    }
}

impl PartialEq for LangUnion {
    fn eq(&self, other: &Self) -> bool {
        self.variant_no == other.variant_no
            && self.value == other.value
            && ptr::eq(self.cls, other.cls)
    }
}

impl Eq for LangUnion {}

impl Hash for LangUnion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.variant_no.hash(state);
        self.value.hash(state);
        ptr::hash(self.cls, state);
    }
}

mod default_functions {
    use crate::lang_union::{LangUnion, UnionMethod};
    use crate::name::Name;
    use crate::operator::Operator;
    use crate::runtime::Runtime;
    use crate::variable::{FnResult, Variable};
    use std::mem::replace;

    pub fn default_methods(name: Name) -> UnionMethod {
        if let Name::Operator(o) = name {
            let result = match o {
                Operator::Repr => default_repr,
                Operator::Str => default_str,
                Operator::Equals => default_eq,
                Operator::Bool => default_bool,
                Operator::In => default_in,
                _ => unimplemented!("name {:?} not found", name),
            };
            UnionMethod::Native(result)
        } else {
            panic!("name {:?} not found", name)
        }
    }

    fn default_repr(this: &LangUnion, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let result = format!(
            "{}.{}({})",
            this.cls.name(),
            this.variant_name(),
            this.value.repr(runtime)?
        );
        runtime.return_1(Variable::String(result.into()))
    }

    fn default_str(this: &LangUnion, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.call_op(Variable::Union(this.clone()), Operator::Repr, args)
    }

    fn default_bool(_this: &LangUnion, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Variable::Bool(true))
    }

    fn default_eq(this: &LangUnion, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let this_var = Variable::Union(this.clone());
        for arg in args {
            if this_var != arg {
                return runtime.return_1(Variable::Bool(false));
            }
        }
        runtime.return_1(Variable::Bool(true))
    }

    fn default_in(this: &LangUnion, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let checked_var = replace(&mut args[0], Variable::Null());
        let this_iter = this.iter(runtime)?;
        while let Option::Some(val) = this_iter.clone().next(runtime)? {
            if checked_var.equals(val, runtime)? {
                return runtime.return_1(true.into());
            }
        }
        runtime.return_1(false.into())
    }
}
