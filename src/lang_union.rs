use crate::custom_var::CustomVar;
use crate::int_var::IntVar;
use crate::lang_union::default_functions::default_methods;
use crate::method::{InnerMethod, StdMethod};
use crate::name::Name;
use crate::name_map::NameMap;
use crate::operator::Operator;
use crate::property::Property;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use crate::{first, looping};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;

pub type UnionMethod = InnerMethod<LangUnion>;
pub type UnionTypeMethod = InnerMethod<Type>;

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
    supers: Vec<u32>,
    variants: Vec<String>,
    variables: HashSet<Arc<str>>,
    methods: NameMap<UnionMethod>,
    static_methods: NameMap<UnionTypeMethod>,
    properties: HashMap<String, Property>,
}

#[derive(Debug, Copy, Clone)]
struct UnionMaker {
    variant_no: usize,
    cls: &'static UnionType,
}

impl LangUnion {
    pub fn new(variant_no: usize, value: Box<Variable>, cls: &'static UnionType) -> LangUnion {
        LangUnion {
            variant_no,
            value,
            cls,
        }
    }

    call_op_fn!(str, StringVar, Str);
    call_op_fn!(repr, StringVar, Repr);
    call_op_fn!(bool, bool, Bool);
    call_op_fn!(int, IntVar, Int);
    call_op_fn!(iter, looping::Iterator, Iter);

    pub fn call_operator(
        self,
        op: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        let inner_method = self.cls.get_method(Name::Operator(op));
        inner_method.call(self, args, runtime)
    }

    pub fn call(self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        self.call_operator(Operator::Call, args.0, args.1)
    }

    pub fn call_op_or_goto(
        self,
        op: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        let inner_method = self.cls.get_method(Name::Operator(op));
        inner_method.call_or_goto(self, args, runtime)
    }

    pub fn call_or_goto(self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        self.call_op_or_goto(Operator::Call, args.0, args.1)
    }

    pub fn index(&self, index: Name, runtime: &mut Runtime) -> Result<Variable, ()> {
        match self.cls.variant_pos(index) {
            Option::Some(true_val) => Result::Ok(
                if self.is_variant(true_val) {
                    Option::Some((*self.value).clone())
                } else {
                    Option::None
                }
                .into(),
            ),
            Option::None => self.index_harder(index, runtime),
        }
    }

    pub fn get_type(&self) -> Type {
        Type::Union(self.cls)
    }

    pub fn variant_name(&self) -> &str {
        self.cls.variants[self.variant_no].as_str()
    }

    pub fn is_variant(&self, variant_no: usize) -> bool {
        self.variant_no == variant_no
    }

    pub fn take_value(self) -> Box<Variable> {
        self.value
    }

    pub fn variant_no(&self) -> usize {
        self.variant_no
    }

    pub fn get_value(&self) -> &Variable {
        &self.value
    }

    fn index_harder(&self, index: Name, runtime: &mut Runtime) -> Result<Variable, ()> {
        match self.cls.get_property(index) {
            Option::Some(val) => {
                val.call_getter(runtime, self.clone().into())?;
                Result::Ok(runtime.pop_return())
            }
            Option::None => {
                let inner_method = self.cls.get_method(index);
                Result::Ok(Box::new(StdMethod::new(self.clone(), inner_method)).into())
            }
        }
    }
}

impl UnionType {
    #[allow(clippy::too_many_arguments)] // Probably should fix this at some point
    pub const fn new(
        name: StringVar,
        file_no: usize,
        supers: Vec<u32>,
        variants: Vec<String>,
        variables: HashSet<Arc<str>>,
        methods: NameMap<UnionMethod>,
        static_methods: NameMap<UnionTypeMethod>,
        properties: HashMap<String, Property>,
    ) -> UnionType {
        UnionType {
            name,
            file_no,
            supers,
            variants,
            variables,
            methods,
            static_methods,
            properties,
        }
    }

    pub fn index(&'static self, name: Name) -> Variable {
        match name {
            Name::Operator(_) => unimplemented!(),
            Name::Attribute(var) => match self.variants.iter().position(|x| *x == var) {
                Option::Some(i) => Rc::new(UnionMaker::new(i, self)).into(),
                Option::None => self.index_attr(var),
            },
        }
    }

    fn index_attr(&'static self, attr: &str) -> Variable {
        let var_attr = Name::Attribute(attr);
        match self.static_methods.get(var_attr) {
            Option::Some(inner_m) => {
                let n = StdMethod::new(Type::Union(self), *inner_m);
                Box::new(n).into()
            }
            _ => unimplemented!("{}.{}", self.name(), attr),
        }
    }

    fn variant_pos(&self, index: Name) -> Option<usize> {
        if let Name::Attribute(name) = index {
            self.variants.iter().position(|x| *x == name)
        } else {
            Option::None
        }
    }

    pub fn name(&self) -> &StringVar {
        &self.name
    }

    pub fn get_property(&self, name: Name) -> Option<&Property> {
        name.do_each(|_| Option::None, |str| self.properties.get(str))
    }

    pub(self) fn get_method(&self, name: Name) -> UnionMethod {
        match self.methods.get(name) {
            Option::Some(t) => *t,
            Option::None => default_methods(name)
                .unwrap_or_else(|| panic!("{}.{} does not exist", self.name, name.as_str())),
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

impl PartialEq for UnionType {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Eq for UnionType {}

impl Hash for UnionType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(self, state);
    }
}

mod default_functions {
    use crate::first;
    use crate::lang_union::{LangUnion, UnionMethod};
    use crate::name::Name;
    use crate::operator::Operator;
    use crate::runtime::Runtime;
    use crate::string_var::StringVar;
    use crate::variable::{FnResult, InnerVar, Variable};

    pub fn default_methods(name: Name) -> Option<UnionMethod> {
        if let Name::Operator(o) = name {
            let result = match o {
                Operator::Repr => default_repr,
                Operator::Str => default_str,
                Operator::Equals => default_eq,
                Operator::Bool => default_bool,
                Operator::In => default_in,
                _ => return Option::None,
            };
            Option::Some(UnionMethod::Native(result))
        } else {
            Option::None
        }
    }

    fn default_repr(this: LangUnion, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let result = format!(
            "{}.{}({})",
            this.cls.name(),
            this.variant_name(),
            this.value.clone().repr(runtime)?
        );
        runtime.return_1(StringVar::from(result).into())
    }

    fn default_str(this: LangUnion, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.call_op(this.into(), Operator::Repr, args)
    }

    fn default_bool(_this: LangUnion, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(true.into())
    }

    fn default_eq(this: LangUnion, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        for arg in args {
            match arg {
                Variable::Normal(InnerVar::Union(other))
                    if (this.cls == other.cls
                        && this.variant_no == other.variant_no
                        && this.value.clone().equals(*other.value.clone(), runtime)?) => {}
                _ => return runtime.return_1(false.into()),
            }
        }
        runtime.return_1(true.into())
    }

    fn default_in(this: LangUnion, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let checked_var = first(args);
        let this_iter = this.iter(runtime)?;
        while let Option::Some(val) = this_iter.clone().next(runtime)?.take_first() {
            if checked_var.clone().equals(val, runtime)? {
                return runtime.return_1(true.into());
            }
        }
        runtime.return_1(false.into())
    }
}

impl From<&'static UnionType> for Type {
    fn from(x: &'static UnionType) -> Self {
        Type::Union(x)
    }
}

impl UnionMaker {
    fn new(variant_no: usize, cls: &'static UnionType) -> UnionMaker {
        UnionMaker { variant_no, cls }
    }

    fn create(&self, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        match args.len() {
            0 => runtime.return_1(
                LangUnion::new(self.variant_no, Box::new(Variable::null()), self.cls).into(),
            ),
            1 => {
                let value = first(args);
                runtime.return_1(LangUnion::new(self.variant_no, Box::new(value), self.cls).into())
            }
            x => panic!(
                "Expected 1 or 0 args, got {}\n{}",
                x,
                runtime.frame_strings()
            ),
        }
    }
}

impl CustomVar for UnionMaker {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        unimplemented!()
    }

    fn get_operator(self: Rc<Self>, op: Operator) -> Variable {
        if op == Operator::Call {
            self.into()
        } else {
            unimplemented!()
        }
    }

    fn get_attribute(self: Rc<Self>, _name: &str) -> Variable {
        unimplemented!()
    }

    fn call(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        self.create(args, runtime)
    }

    fn call_or_goto(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        self.create(args, runtime)
    }
}
