#[macro_export]
macro_rules! hash_map {
    { $($key:expr => $value:expr),+ $(,)? } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
    };
}

#[macro_export]
macro_rules! hash_set {
    { $($value:expr),* $(,)? } => {
        {
            let mut m = ::std::collections::HashSet::new();
            $(
                m.insert($value);
            )+
            m
        }
    };
}

#[macro_export]
macro_rules! name_map {
    () => {
        $crate::name_map::NameMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        {
            let mut m = $crate::name_map::NameMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
    };
}

#[macro_export]
macro_rules! call_op_fn {
    ($name:ident, $ret:ty, $op:ident) => {
        pub fn $name(self, runtime: &mut Runtime) -> Result<$ret, ()> {
            self.call_operator(Operator::$op, vec![], runtime)?;
            Result::Ok(runtime.pop_return().into())
        }
    };
}
