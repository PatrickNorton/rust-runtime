macro_rules! primitive_arithmetic {
    () => {
        fn add(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            let mut result = self.value;
            for arg in args {
                let value = downcast_var::<Self>(arg).unwrap();
                result += value.value;
            }
            runtime.return_1(Rc::new(Self::new(result)).into())
        }

        fn sub(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            let mut result = self.value;
            for arg in args {
                let value = downcast_var::<Self>(arg).unwrap();
                result -= value.value;
            }
            runtime.return_1(Rc::new(Self::new(result)).into())
        }

        fn mul(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            let mut result = self.value;
            for arg in args {
                let value = downcast_var::<Self>(arg).unwrap();
                result *= value.value;
            }
            runtime.return_1(Rc::new(Self::new(result)).into())
        }

        fn div(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            let mut result = self.value;
            for arg in args {
                let value = downcast_var::<Self>(arg).unwrap();
                result /= value.value;
            }
            runtime.return_1(Rc::new(Self::new(result)).into())
        }

        fn modulo(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            debug_assert_eq!(args.len(), 1);
            let other = downcast_var::<Self>(first(args)).unwrap();
            let result = Self::new(self.value % other.value);
            runtime.return_1(Rc::new(result).into())
        }
    };
}

macro_rules! primitive_comparisons {
    () => {
        fn eq(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            for arg in args {
                let value = downcast_var::<Self>(arg).unwrap();
                if value.value != self.value {
                    return runtime.return_1(false.into());
                }
            }
            runtime.return_1(true.into())
        }

        fn ne(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            for arg in args {
                let value = downcast_var::<Self>(arg).unwrap();
                if value.value == self.value {
                    return runtime.return_1(false.into());
                }
            }
            runtime.return_1(true.into())
        }

        fn gt(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            debug_assert_eq!(args.len(), 1);
            let other = downcast_var::<Self>(first(args)).unwrap();
            let result = self.value > other.value;
            runtime.return_1(result.into())
        }

        fn ge(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            debug_assert_eq!(args.len(), 1);
            let other = downcast_var::<Self>(first(args)).unwrap();
            let result = self.value >= other.value;
            runtime.return_1(result.into())
        }

        fn lt(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            debug_assert_eq!(args.len(), 1);
            let other = downcast_var::<Self>(first(args)).unwrap();
            let result = self.value < other.value;
            runtime.return_1(result.into())
        }

        fn le(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            debug_assert_eq!(args.len(), 1);
            let other = downcast_var::<Self>(first(args)).unwrap();
            let result = self.value <= other.value;
            runtime.return_1(result.into())
        }
    };
}

macro_rules! primitive_custom {
    ($name:ident) => {
        impl CustomVar for $name {
            fn set(self: Rc<Self>, _name: Name, _object: Variable) {
                unimplemented!()
            }

            fn get_type(&self) -> Type {
                todo!()
            }

            fn get_operator(self: Rc<Self>, op: Operator) -> Variable {
                StdMethod::new_native(self, Self::op_fn(op)).into()
            }

            fn get_attribute(self: Rc<Self>, name: &str) -> Variable {
                StdMethod::new_native(self, Self::attr_fn(name)).into()
            }
        }
    };
}
