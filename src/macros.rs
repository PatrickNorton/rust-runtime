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

macro_rules! call_op_fn {
    ($name:ident, $ret:ty, $op:ident) => {
        pub fn $name(self, runtime: &mut Runtime) -> Result<$ret, ()> {
            self.call_operator(Operator::$op, vec![], runtime)?;
            Result::Ok(runtime.pop_return().into())
        }
    };
}
